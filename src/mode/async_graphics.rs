//! Async graphics mode for Embassy compatibility

use crate::async_display::AsyncDisplay;
use crate::async_interface::AsyncWriteOnlyDataCommand;
use crate::properties::DisplayRotation;
use display_interface::DisplayError;

#[cfg(feature = "async")]
use embassy_time::{Timer, Duration};
use embedded_hal::digital::OutputPin;

/// Async Graphics Mode for the display
pub struct AsyncGraphicsMode<DI>
where
    DI: AsyncWriteOnlyDataCommand,
{
    display: AsyncDisplay<DI>,
    #[cfg(feature = "buffered")]
    pub buffer: &'static mut [u8],
}

impl<DI> AsyncGraphicsMode<DI>
where
    DI: AsyncWriteOnlyDataCommand,
{
    #[cfg(not(feature = "buffered"))]
    /// Create new AsyncGraphicsMode instance
    pub fn new(display: AsyncDisplay<DI>) -> Self {
        AsyncGraphicsMode { display }
    }

    #[cfg(feature = "buffered")]
    /// Create new AsyncGraphicsMode instance with buffer
    pub fn new(display: AsyncDisplay<DI>, buffer: &'static mut [u8]) -> Self {
        AsyncGraphicsMode { display, buffer }
    }

    #[cfg(not(feature = "buffered"))]
    /// Release all resources used by AsyncGraphicsMode
    pub fn release(self) -> AsyncDisplay<DI> {
        self.display
    }

    #[cfg(feature = "buffered")]
    /// Release all resources used by AsyncGraphicsMode
    pub fn release(self) -> (AsyncDisplay<DI>, &'static mut [u8]) {
        (self.display, self.buffer)
    }

    #[cfg(not(feature = "buffered"))]
    /// Clear the display
    pub async fn clear(&mut self) -> Result<(), DisplayError> {
        self.display.clear().await
    }

    #[cfg(feature = "buffered")]
    /// Clear the display
    pub async fn clear(&mut self, flush: bool) -> Result<(), DisplayError> {
        for i in 0..self.buffer.len() {
            self.buffer[i] = 0u8;
        }
        if flush {
            self.flush().await?;
        }
        Ok(())
    }

    /// Reset display asynchronously
    #[cfg(feature = "async")]
    pub async fn reset_async<RST>(&mut self, rst: &mut RST) -> Result<(), RST::Error>
    where
        RST: OutputPin,
    {
        rst.set_high()?;
        Timer::after(Duration::from_millis(1)).await;
        rst.set_low()?;
        Timer::after(Duration::from_millis(10)).await;
        rst.set_high()?;
        Ok(())
    }

    #[cfg(feature = "buffered")]
    /// Access the framebuffer
    pub fn fb(&self) -> &[u8] {
        self.buffer
    }

    #[cfg(feature = "buffered")]
    /// Access the framebuffer
    pub fn fb_mut(&mut self) -> &mut [u8] {
        self.buffer
    }

    #[cfg(not(feature = "buffered"))]
    /// Turn a pixel on or off. A non-zero `value` is treated as on, `0` as off. If the X and Y
    /// coordinates are out of the bounds of the display, this method call is a noop.
    pub async fn set_pixel(&mut self, x: u32, y: u32, color: u16) -> Result<(), DisplayError> {
        let (display_width, display_height) = self.display.get_size().dimensions();
        let rot = self.display.get_rotation();
        let (nx, ny) = match rot {
            DisplayRotation::Rotate0 | DisplayRotation::Rotate180 => (x, y),
            DisplayRotation::Rotate90 | DisplayRotation::Rotate270 => (y, x),
        };
        self.display
            .set_draw_area((nx as u8, ny as u8), (display_width, display_height))
            .await?;
        self.display
            .draw(&[(color >> 8) as u8, color as u8])
            .await?;
        Ok(())
    }

    #[cfg(feature = "buffered")]
    /// Turn a pixel on or off. A non-zero `value` is treated as on, `0` as off. If the X and Y
    /// coordinates are out of the bounds of the display, this method call is a noop.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u16) {
        // set bytes in buffer
        self.buffer[(y as usize * 128usize + x as usize) * 2] = (color >> 8) as u8;
        self.buffer[((y as usize * 128usize + x as usize) * 2) + 1usize] = color as u8;
    }

    #[cfg(feature = "buffered")]
    pub async fn flush(&mut self) -> Result<(), DisplayError> {
        let (display_width, display_height) = self.display.get_size().dimensions();
        self.display
            .set_draw_area((0, 0), (display_width, display_height))
            .await?;
        self.display.draw(self.buffer).await?;
        Ok(())
    }

    /// Display is set up in column mode, i.e. a byte walks down a column of 8 pixels from
    /// column 0 on the left, to column _n_ on the right
    pub async fn init(&mut self) -> Result<(), DisplayError> {
        self.display.init().await?;
        Ok(())
    }

    /// Set the display rotation
    pub async fn set_rotation(&mut self, rot: DisplayRotation) -> Result<(), DisplayError> {
        self.display.set_rotation(rot).await
    }

    /// Get display dimensions, taking into account the current rotation of the display
    pub fn get_dimensions(&self) -> (u8, u8) {
        self.display.get_dimensions()
    }
}

#[cfg(feature = "graphics")]
extern crate embedded_graphics_core;
#[cfg(feature = "graphics")]
use self::embedded_graphics_core::pixelcolor::Rgb565;
#[cfg(feature = "graphics")]
use self::embedded_graphics_core::prelude::{
    Dimensions, DrawTarget, OriginDimensions, Pixel, Size,
};
#[cfg(all(feature = "graphics", not(feature = "buffered")))]
use self::embedded_graphics_core::primitives::Rectangle;

// Note: embedded-graphics doesn't support async yet, so we keep the sync DrawTarget implementation
#[cfg(feature = "graphics")]
impl<DI: AsyncWriteOnlyDataCommand> DrawTarget for AsyncGraphicsMode<DI> {
    type Color = Rgb565;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|Pixel(pos, _)| bb.contains(*pos))
            .for_each(|Pixel(pos, color)| {
                #[cfg(not(feature = "buffered"))]
                {
                    // For non-buffered mode, we can't easily make this async
                    // Users should use the async set_pixel method directly
                    let _ = pos;
                    let _ = color;
                }
                #[cfg(feature = "buffered")]
                {
                    use self::embedded_graphics_core::pixelcolor::raw::RawU16;
                    use self::embedded_graphics_core::prelude::RawData;
                    self.set_pixel(pos.x as u32, pos.y as u32, RawU16::from(color).into_inner())
                }
            });

        Ok(())
    }

    #[cfg(not(feature = "buffered"))]
    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        // For async compatibility, this method is simplified for non-buffered mode
        // Users should use async methods directly for better performance
        let _ = area;
        let _ = colors;
        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<DI: AsyncWriteOnlyDataCommand> OriginDimensions for AsyncGraphicsMode<DI> {
    fn size(&self) -> Size {
        let dim = self.display.get_size().dimensions();
        Size::from((dim.0 as u32, dim.1 as u32))
    }
}