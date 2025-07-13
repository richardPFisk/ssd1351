//! Async interface factory for Embassy compatibility

use crate::async_display::AsyncDisplay;
use crate::async_interface::AsyncWriteOnlyDataCommand;
use crate::mode::async_graphics::AsyncGraphicsMode;
use crate::properties::{DisplayRotation, DisplaySize};

/// Async Builder struct. Driver options and interface are set using its methods.
#[derive(Clone)]
pub struct AsyncBuilder {
    display_size: DisplaySize,
    rotation: DisplayRotation,
}

impl Default for AsyncBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncBuilder {
    /// Create new builder with a default size of 128 x 128 pixels and no rotation.
    pub fn new() -> Self {
        Self {
            display_size: DisplaySize::Display128x128,
            rotation: DisplayRotation::Rotate0,
        }
    }

    /// Set the size of the display. Supported sizes are defined by [DisplaySize].
    pub fn with_size(&self, display_size: DisplaySize) -> Self {
        Self {
            display_size,
            ..*self
        }
    }

    /// Set the rotation of the display to one of four values. Defaults to no rotation.
    pub fn with_rotation(&self, rotation: DisplayRotation) -> Self {
        Self { rotation, ..*self }
    }

    #[cfg(feature = "buffered")]
    /// Finish the builder and use the given interface to communicate with the display
    pub fn connect_interface<DI>(
        &self,
        display_interface: DI,
        buffer: &'static mut [u8],
    ) -> AsyncGraphicsMode<DI>
    where
        DI: AsyncWriteOnlyDataCommand,
    {
        assert_eq!(buffer.len(), self.display_size.num_pixels() * 2);
        let display = AsyncDisplay::new(display_interface, self.display_size, self.rotation);
        AsyncGraphicsMode::new(display, buffer)
    }

    #[cfg(not(feature = "buffered"))]
    /// Finish the builder and use the given interface to communicate with the display
    pub fn connect_interface<DI>(&self, display_interface: DI) -> AsyncGraphicsMode<DI>
    where
        DI: AsyncWriteOnlyDataCommand,
    {
        let display = AsyncDisplay::new(display_interface, self.display_size, self.rotation);
        AsyncGraphicsMode::new(display)
    }

    /// Create an async display instance (without graphics mode)
    pub fn connect_display<DI>(&self, display_interface: DI) -> AsyncDisplay<DI>
    where
        DI: AsyncWriteOnlyDataCommand,
    {
        AsyncDisplay::new(display_interface, self.display_size, self.rotation)
    }
}