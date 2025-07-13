use hal::spi::{Mode, Phase, Polarity};

pub const SSD1351_SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};

pub use display_interface_spi::SPIInterface;

// Async exports
#[cfg(feature = "async")]
pub use crate::async_builder::AsyncBuilder;
#[cfg(feature = "async")]
pub use crate::async_display::AsyncDisplay;
#[cfg(feature = "async")]
pub use crate::async_interface::AsyncWriteOnlyDataCommand;
#[cfg(feature = "async")]
pub use crate::mode::AsyncGraphicsMode;
