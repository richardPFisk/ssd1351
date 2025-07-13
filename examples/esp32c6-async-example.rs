//! Async SSD1351 example for ESP32-C6 using Embassy
//! 
//! This example demonstrates how to use the SSD1351 OLED display driver
//! with async Embassy on ESP32-C6. It shows:
//! - Async display initialization
//! - Async pixel drawing
//! - Async graphics operations
//! - SPI bus sharing with async mutex

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Timer;
use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::{Rgb565, RgbColor},
    primitives::{Primitive, PrimitiveStyleBuilder, Rectangle},
    Drawable,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    gpio::{Io, Level, Output},
    prelude::*,
    spi::{master::Spi, SpiMode},
    timer::timg::TimerGroup,
};
use esp_println::println;
use static_cell::StaticCell;

// Import the async SSD1351 modules
use ssd1351::{
    async_builder::AsyncBuilder,
    async_interface::AsyncWriteOnlyDataCommand,
    mode::AsyncGraphicsMode,
    prelude::*,
    properties::{DisplayRotation, DisplaySize},
};
use display_interface_spi::SPIInterface;

// Static allocations for async usage
static EXECUTOR: StaticCell<embassy_executor::Executor> = StaticCell::new();

// Async SPI interface adapter for Embassy
pub struct AsyncSPIInterface<SPI> {
    interface: SPIInterface<SPI, Output<'static>, Output<'static>>,
}

impl<SPI> AsyncSPIInterface<SPI> {
    pub fn new(
        spi: SPI,
        dc: Output<'static>,
        cs: Output<'static>,
    ) -> Self {
        let interface = SPIInterface::new(spi, dc, cs);
        Self { interface }
    }
}

impl<SPI> AsyncWriteOnlyDataCommand for AsyncSPIInterface<SPI>
where
    SPI: embedded_hal::spi::SpiDevice + Send,
{
    async fn send_commands(&mut self, cmd: display_interface::DataFormat<'_>) -> Result<(), display_interface::DisplayError> {
        // In a real implementation, you'd use async SPI here
        // For now, we'll use the blocking implementation as a fallback
        use display_interface::WriteOnlyDataCommand;
        self.interface.send_commands(cmd)
    }
    
    async fn send_data(&mut self, buf: display_interface::DataFormat<'_>) -> Result<(), display_interface::DisplayError> {
        // In a real implementation, you'd use async SPI here
        // For now, we'll use the blocking implementation as a fallback
        use display_interface::WriteOnlyDataCommand;
        self.interface.send_data(buf)
    }
}

#[main]
async fn main(_spawner: Spawner) {
    println!("Starting async SSD1351 example!");
    
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    
    // Initialize embassy time driver
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    
    // Configure SPI pins for SSD1351
    let sclk = io.pins.gpio6;
    let mosi = io.pins.gpio7; 
    let cs = Output::new(io.pins.gpio10, Level::High);
    let dc = Output::new(io.pins.gpio5, Level::Low);
    let mut rst = Output::new(io.pins.gpio4, Level::High);
    
    // Create SPI interface
    let spi = Spi::new(peripherals.SPI2, 4.MHz(), SpiMode::Mode0)
        .with_pins(sclk, mosi, esp_hal::gpio::NO_PIN, esp_hal::gpio::NO_PIN);
        
    let spi_device = ExclusiveDevice::new(spi, cs, embassy_time::Delay);
    
    // Create async display interface
    let async_interface = AsyncSPIInterface::new(spi_device, dc, cs);
    
    // Build async display
    let mut display: AsyncGraphicsMode<_> = AsyncBuilder::new()
        .with_size(DisplaySize::Display128x128)
        .with_rotation(DisplayRotation::Rotate0)
        .connect_interface(async_interface);
    
    println!("Resetting display...");
    // Reset display asynchronously
    #[cfg(feature = "async")]
    display.reset_async(&mut rst).await.unwrap();
    
    println!("Initializing display...");
    // Initialize display asynchronously
    display.init().await.unwrap();
    
    println!("Clearing display...");
    // Clear display asynchronously
    #[cfg(not(feature = "buffered"))]
    display.clear().await.unwrap();
    #[cfg(feature = "buffered")]
    display.clear(true).await.unwrap();
    
    println!("Drawing pixels...");
    // Draw some pixels asynchronously
    for x in 0..64 {
        for y in 0..64 {
            let color = if (x + y) % 2 == 0 { 0xF800 } else { 0x07E0 }; // Red or Green
            #[cfg(not(feature = "buffered"))]
            display.set_pixel(x, y, color).await.unwrap();
            #[cfg(feature = "buffered")]
            display.set_pixel(x, y, color);
        }
    }
    
    #[cfg(feature = "buffered")]
    {
        println!("Flushing buffer...");
        display.flush().await.unwrap();
    }
    
    // Draw colored rectangles using embedded-graphics
    println!("Drawing rectangles...");
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
    
    red_rect.draw(&mut display).unwrap();
    green_rect.draw(&mut display).unwrap();
    blue_rect.draw(&mut display).unwrap();
    
    #[cfg(feature = "buffered")]
    {
        println!("Final flush...");
        display.flush().await.unwrap();
    }
    
    println!("Display initialization complete!");
    
    // Animation loop
    let mut frame = 0u32;
    loop {
        // Draw a moving pixel
        let x = (frame % 128) as u32;
        let y = 64;
        
        #[cfg(not(feature = "buffered"))]
        display.set_pixel(x, y, 0xFFFF).await.unwrap(); // White pixel
        #[cfg(feature = "buffered")]
        {
            display.set_pixel(x, y, 0xFFFF); // White pixel
            display.flush().await.unwrap();
        }
        
        frame = frame.wrapping_add(1);
        Timer::after_millis(50).await;
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic: {}", info);
    loop {}
}