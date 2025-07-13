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
    
    println!("Resetting display...");
    // Reset display
    display.reset_async(&mut rst).await.unwrap();
    
    println!("Initializing display...");
    // Initialize display asynchronously
    display.init().await.unwrap();
    
    println!("Clearing display...");
    // Clear display asynchronously
    display.clear().await.unwrap();
    
    println!("Drawing test pattern...");
    
    // Draw some test pixels
    for x in 0..64 {
        for y in 0..64 {
            let color = if (x + y) % 2 == 0 { 0xF800 } else { 0x07E0 }; // Red or Green checkerboard
            display.set_pixel(x, y, color).await.unwrap();
        }
        // Add a small delay to see the drawing progress
        if x % 8 == 0 {
            Timer::after_millis(10).await;
        }
    }
    
    println!("Drawing colored rectangles...");
    
    // Draw colored rectangles using embedded-graphics
    let red_rect = Rectangle::new(Point::new(70, 10), Size::new(50, 30))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::RED)
                .build(),
        );
    
    let green_rect = Rectangle::new(Point::new(70, 50), Size::new(50, 30))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::GREEN)
                .build(),
        );
    
    let blue_rect = Rectangle::new(Point::new(70, 90), Size::new(50, 30))
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLUE)
                .build(),
        );
    
    // Draw rectangles (embedded-graphics calls are still sync)
    red_rect.draw(&mut display).unwrap();
    green_rect.draw(&mut display).unwrap();
    blue_rect.draw(&mut display).unwrap();
    
    println!("Display initialization complete! Starting animation...");
    
    // Animation loop - moving white pixel
    let mut frame = 0u32;
    loop {
        // Draw a moving white pixel across the top
        let x = (frame % 128) as u32;
        let y = 5;
        
        // Clear previous position (draw black pixel)
        if x > 0 {
            display.set_pixel(x - 1, y, 0x0000).await.unwrap(); // Black
        } else {
            display.set_pixel(127, y, 0x0000).await.unwrap(); // Clear last position when wrapping
        }
        
        // Draw white pixel at new position
        display.set_pixel(x, y, 0xFFFF).await.unwrap(); // White
        
        frame = frame.wrapping_add(1);
        Timer::after_millis(50).await;
        
        // Print progress every second
        if frame % 20 == 0 {
            println!("Animation frame: {}", frame);
        }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop {}
}