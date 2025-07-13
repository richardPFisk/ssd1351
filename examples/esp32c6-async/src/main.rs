#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::{Rgb565, RgbColor},
    primitives::{Primitive, PrimitiveStyleBuilder, Rectangle},
    Drawable,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_alloc as _;
use esp_hal::{
    gpio::{Io, Level, Output},
    spi::{master::{Spi, Config as SpiConfig}, Mode as SpiMode},
    timer::timg::TimerGroup,
};
use esp_hal::time::Rate;
use embedded_hal::spi::SpiDevice;
use embedded_hal::digital::OutputPin;
use fugit::RateExtU32;
use esp_println::println;

// Import our async SSD1351 modules
use ssd1351::{
    async_builder::AsyncBuilder,
    async_interface::AsyncWriteOnlyDataCommand,
    mode::AsyncGraphicsMode,
    properties::{DisplayRotation, DisplaySize},
};
use display_interface_spi::SPIInterface;
use display_interface::{DataFormat, DisplayError};

// Create an async-compatible SPI interface wrapper
pub struct AsyncSPIInterface<SPI, DC> {
    interface: SPIInterface<SPI, DC>,
}

impl<SPI, DC> AsyncSPIInterface<SPI, DC> {
    pub fn new(spi: SPI, dc: DC) -> Self {
        let interface = SPIInterface::new(spi, dc);
        Self { interface }
    }
}

impl<SPI, DC> AsyncWriteOnlyDataCommand for AsyncSPIInterface<SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    async fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        // For now, use the blocking implementation
        // In a real async implementation, you'd use async SPI here
        display_interface::WriteOnlyDataCommand::send_commands(&mut self.interface, cmd)
    }
    
    async fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        // For now, use the blocking implementation
        // In a real async implementation, you'd use async SPI here
        display_interface::WriteOnlyDataCommand::send_data(&mut self.interface, buf)
    }
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    println!("Starting ESP32-C6 async SSD1351 example!");
    
    // Initialize the system
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    // Initialize the allocator
    esp_alloc::heap_allocator!(size: 72 * 1024);
    
    // Initialize embassy time driver
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    
    let io = Io::new(peripherals.IO_MUX);
    
    // Configure SPI pins for SSD1351 using your exact pin configuration  
    let clk = peripherals.GPIO18;  // SCK
    let mosi = peripherals.GPIO19; // MOSI
    let cs = Output::new(peripherals.GPIO3, Level::Low, Default::default());
    let dc = Output::new(peripherals.GPIO4, Level::Low, Default::default());
    let mut rst = Output::new(peripherals.GPIO14, Level::Low, Default::default());
    
    println!("Configuring SPI...");
    
    // Create SPI interface using the correct ESP-HAL API
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(4)).with_mode(SpiMode::_0),
    )
    .unwrap()
    .with_sck(clk)
    .with_mosi(mosi);
        
    let spi_device = ExclusiveDevice::new(spi, cs, Delay);
    
    println!("Creating display interface...");
    
    // Create async display interface
    let async_interface = AsyncSPIInterface::new(spi_device, dc);
    
    // Build async display
    let mut display: AsyncGraphicsMode<_> = AsyncBuilder::new()
        .with_size(DisplaySize::Display128x128)
        .with_rotation(DisplayRotation::Rotate0)
        .connect_interface(async_interface);
    
    // Debug: Check display dimensions
    let (width, height) = display.get_dimensions();
    println!("Display dimensions: {}x{}", width, height);
    
    println!("Resetting display...");
    // Reset display
    display.reset_async(&mut rst).await.unwrap();
    
    println!("Initializing display...");
    // Initialize display asynchronously
    display.init().await.unwrap();
    
    println!("Clearing display...");
    // Clear display asynchronously
    display.clear().await.unwrap();
    
    println!("Drawing 4 colored quadrants to fill the entire screen...");
    
    // Draw pixels directly to ensure we fill the entire 128x128 screen
    // This approach guarantees every pixel is set correctly
    
    for y in 0..128 {
        for x in 0..128 {
            let color = if x < 64 && y < 64 {
                0xF800 // Top-left: RED (RGB565: 11111 000000 00000)
            } else if x >= 64 && y < 64 {
                0x07E0 // Top-right: GREEN (RGB565: 00000 111111 00000)
            } else if x < 64 && y >= 64 {
                0x001F // Bottom-left: BLUE (RGB565: 00000 000000 11111)
            } else {
                0xFFFF // Bottom-right: WHITE (RGB565: 11111 111111 11111)
            };
            
            display.set_pixel(x as u32, y as u32, color).await.unwrap();
        }
        
        // Add progress updates every 16 lines to see the drawing progress
        if y % 16 == 0 {
            println!("Drawing line {}/128", y);
            Timer::after_millis(10).await; // Small delay to see progress
        }
    }
    
    println!("Finished drawing all 16,384 pixels!");
    
    println!("Display initialization complete! Starting border animation...");
    
    // Animation loop - moving black dot around the center borders between quadrants
    let mut frame = 0u32;
    loop {
        let cycle_length = 256; // Total animation cycle length
        let pos = frame % cycle_length;
        
        // Calculate position along the border between quadrants
        let (x, y) = if pos < 64 {
            // Top border: move from center-left to center-right
            (32 + pos, 63) // Horizontal line at y=63 (border between top and bottom)
        } else if pos < 128 {
            // Right border: move from center-top to center-bottom  
            (63, 32 + (pos - 64)) // Vertical line at x=63 (border between left and right)
        } else if pos < 192 {
            // Bottom border: move from center-right to center-left
            (32 + (63 - (pos - 128)), 64) // Horizontal line at y=64
        } else {
            // Left border: move from center-bottom to center-top
            (64, 32 + (63 - (pos - 192))) // Vertical line at x=64
        };
        
        // Clear previous position
        let prev_cycle = (frame.wrapping_sub(1)) % cycle_length;
        let (prev_x, prev_y) = if prev_cycle < 64 {
            (32 + prev_cycle, 63)
        } else if prev_cycle < 128 {
            (63, 32 + (prev_cycle - 64))
        } else if prev_cycle < 192 {
            (32 + (63 - (prev_cycle - 128)), 64)
        } else {
            (64, 32 + (63 - (prev_cycle - 192)))
        };
        
        // Restore the original color at previous position
        let prev_color = if prev_x < 64 && prev_y < 64 {
            0xF800 // Red
        } else if prev_x >= 64 && prev_y < 64 {
            0x07E0 // Green
        } else if prev_x < 64 && prev_y >= 64 {
            0x001F // Blue
        } else {
            0xFFFF // White
        };
        display.set_pixel(prev_x, prev_y, prev_color).await.unwrap();
        
        // Draw black dot at new position
        display.set_pixel(x, y, 0x0000).await.unwrap(); // Black dot
        
        frame = frame.wrapping_add(1);
        Timer::after_millis(20).await;
        
        // Print progress every 2 seconds
        if frame % 100 == 0 {
            println!("Animation frame: {}", frame);
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop {}
}