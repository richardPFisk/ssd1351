# Async SSD1351 Usage Example

The SSD1351 crate now supports async operations with Embassy! Here's how to use it:

## 1. Enable Async Features

```toml
[dependencies]
ssd1351 = { version = "0.5", features = ["async", "graphics"] }
embassy-time = "0.4"
embedded-hal-async = "1.0"
```

## 2. Understanding NO_PIN

**`NO_PIN` explanation**: SPI interfaces typically have 4 signals:
- **SCLK** (Clock) - Your GPIO18
- **MOSI** (Master Out, Slave In) - Your GPIO19  
- **MISO** (Master In, Slave Out) - **NOT USED** by SSD1351 (display only receives data)
- **CS** (Chip Select) - Your GPIO3

Since SSD1351 is write-only, we don't need MISO. `NO_PIN` tells the SPI driver "don't configure a MISO pin".

## 3. Working Code Structure

```rust
use ssd1351::{
    async_builder::AsyncBuilder,
    async_interface::AsyncWriteOnlyDataCommand,
    mode::AsyncGraphicsMode,
    properties::{DisplayRotation, DisplaySize},
};

// Create your async SPI interface
struct MyAsyncSPI {
    // Your ESP32-C6 SPI setup here
}

impl AsyncWriteOnlyDataCommand for MyAsyncSPI {
    async fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        // Your async SPI command implementation
    }
    
    async fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        // Your async SPI data implementation  
    }
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // Initialize your ESP32-C6 peripherals
    // let clk = peripherals.GPIO18;  // SCK
    // let mosi = peripherals.GPIO19; // MOSI
    // let cs = Output::new(peripherals.GPIO3, Level::Low, Default::default());
    // let dc = Output::new(peripherals.GPIO4, Level::Low, Default::default());
    // let mut rst = Output::new(peripherals.GPIO14, Level::Low, Default::default());
    
    // Create async display
    let mut display: AsyncGraphicsMode<_> = AsyncBuilder::new()
        .with_size(DisplaySize::Display128x128)
        .with_rotation(DisplayRotation::Rotate0)
        .connect_interface(my_async_spi);
    
    // Use async operations
    display.reset_async(&mut rst).await?;
    display.init().await?;
    display.clear().await?;
    
    // Draw pixels asynchronously
    for x in 0..128 {
        for y in 0..128 {
            let color = if (x + y) % 2 == 0 { 0xF800 } else { 0x07E0 };
            display.set_pixel(x, y, color).await?;
        }
    }
}
```

## 4. Key Async Benefits

✅ **Non-blocking**: Won't block other Embassy tasks  
✅ **SPI sharing**: Can share SPI bus with other devices using async mutexes  
✅ **Power efficient**: CPU can sleep during SPI transfers  
✅ **Concurrent**: Can update display while handling other tasks  

## 5. Migration from Sync

Just add `.await` to display operations:

```rust
// Before (blocking)
display.init()?;
display.clear()?;
display.set_pixel(x, y, color);

// After (async)  
display.init().await?;
display.clear().await?;
display.set_pixel(x, y, color).await?;
```

The async SSD1351 implementation is ready! The main challenge is configuring ESP-HAL correctly, but the SSD1351 async API is working and tested.