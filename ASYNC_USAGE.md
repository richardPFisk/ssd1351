# Async Usage with Embassy

This document describes how to use the SSD1351 driver with async Rust and Embassy framework.

## Features

Enable the `async` feature to get access to async functionality:

```toml
[dependencies]
ssd1351 = { version = "0.5", features = ["async", "graphics"] }
embassy-time = "0.4"
embedded-hal-async = "1.0"
```

## Basic Usage

### 1. Create an Async Interface

The async version uses `AsyncWriteOnlyDataCommand` trait instead of the blocking `WriteOnlyDataCommand`:

```rust
use ssd1351::{
    async_builder::AsyncBuilder,
    async_interface::AsyncWriteOnlyDataCommand,
    mode::AsyncGraphicsMode,
    properties::{DisplayRotation, DisplaySize},
};

// Your async SPI interface implementation
struct AsyncSPIInterface<SPI> {
    interface: SPIInterface<SPI, Output<'static>, Output<'static>>,
}

impl<SPI> AsyncWriteOnlyDataCommand for AsyncSPIInterface<SPI>
where
    SPI: embedded_hal_async::spi::SpiDevice,
{
    async fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        // Your async SPI implementation
    }
    
    async fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        // Your async SPI implementation  
    }
}
```

### 2. Build and Initialize Display

```rust
// Create async display
let mut display: AsyncGraphicsMode<_> = AsyncBuilder::new()
    .with_size(DisplaySize::Display128x128)
    .with_rotation(DisplayRotation::Rotate0)
    .connect_interface(async_interface);

// Reset display asynchronously (requires async feature)
display.reset_async(&mut rst_pin).await?;

// Initialize display asynchronously
display.init().await?;
```

### 3. Drawing Operations

#### Non-buffered Mode

```rust
// Clear display
display.clear().await?;

// Set individual pixels
display.set_pixel(x, y, color).await?;
```

#### Buffered Mode

```rust
// Enable buffered feature in Cargo.toml
// ssd1351 = { version = "0.5", features = ["async", "graphics", "buffered"] }

// Clear buffer (optionally flush to display)
display.clear(true).await?; // true = also flush to display

// Set pixels in buffer (synchronous - no await needed)
display.set_pixel(x, y, color);

// Flush buffer to display
display.flush().await?;
```

### 4. embedded-graphics Integration

The async version maintains compatibility with embedded-graphics:

```rust
use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::{Rgb565, RgbColor},
    primitives::{Primitive, PrimitiveStyleBuilder, Rectangle},
    Drawable,
};

let rect = Rectangle::new(Point::new(10, 10), Size::new(50, 30))
    .into_styled(
        PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::RED)
            .build(),
    );

// Draw using embedded-graphics (synchronous)
rect.draw(&mut display)?;

// For buffered mode, flush after drawing
#[cfg(feature = "buffered")]
display.flush().await?;
```

## Complete Example

See `examples/esp32c6-async-example.rs` for a full working example with ESP32-C6 and Embassy.

## Key Differences from Sync Version

| Operation | Sync Version | Async Version |
|-----------|--------------|---------------|
| Builder | `Builder::new()` | `AsyncBuilder::new()` |
| Display Type | `GraphicsMode<DI>` | `AsyncGraphicsMode<DI>` |
| Interface Trait | `WriteOnlyDataCommand` | `AsyncWriteOnlyDataCommand` |
| Initialization | `display.init()?` | `display.init().await?` |
| Reset | `display.reset(&mut rst, &mut delay)?` | `display.reset_async(&mut rst).await?` |
| Clear | `display.clear()?` | `display.clear().await?` |
| Set Pixel (unbuffered) | `display.set_pixel(x, y, color)` | `display.set_pixel(x, y, color).await?` |
| Flush (buffered) | `display.flush()` | `display.flush().await?` |

## Performance Considerations

1. **Buffered Mode**: Recommended for better performance. Buffer operations are synchronous, only flush operations are async.

2. **SPI Bus Sharing**: Use Embassy's async mutex for sharing SPI bus between multiple devices:

```rust
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

static SPI_BUS: StaticCell<Mutex<NoopRawMutex, Spi<_>>> = StaticCell::new();
```

3. **Embedded Graphics**: embedded-graphics operations remain synchronous. For buffered mode, call `flush()` after drawing operations.

## Migration Guide

To migrate from sync to async:

1. Add async feature: `features = ["async", "graphics"]`
2. Replace `Builder` with `AsyncBuilder`
3. Replace `GraphicsMode` with `AsyncGraphicsMode`
4. Add `.await` to all display operations
5. Replace reset function with `reset_async()` 
6. Update your SPI interface to implement `AsyncWriteOnlyDataCommand`

The async version maintains API compatibility where possible, making migration straightforward.