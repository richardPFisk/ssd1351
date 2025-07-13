//! Async display interface traits for Embassy compatibility

use display_interface::{DataFormat, DisplayError};

/// Async version of WriteOnlyDataCommand trait for non-blocking communication
/// 
/// This trait should be implemented by your async SPI interface to enable
/// non-blocking communication with the SSD1351 display.
pub trait AsyncWriteOnlyDataCommand {
    /// Send commands to the display
    async fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError>;
    
    /// Send data to the display
    async fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError>;
}

// Provide a blanket implementation for compatibility with existing sync interfaces
impl<T> AsyncWriteOnlyDataCommand for T
where
    T: display_interface::WriteOnlyDataCommand,
{
    async fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        display_interface::WriteOnlyDataCommand::send_commands(self, cmd)
    }
    
    async fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        display_interface::WriteOnlyDataCommand::send_data(self, buf)
    }
}