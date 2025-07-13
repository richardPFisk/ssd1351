//! Async version of command interface for Embassy compatibility

use crate::async_interface::AsyncWriteOnlyDataCommand;
use crate::command::Command;
use display_interface::{DataFormat, DisplayError};

impl Command {
    /// Send command to SSD1351 asynchronously
    pub async fn send_async<DI>(self, iface: &mut DI) -> Result<(), DisplayError>
    where
        DI: AsyncWriteOnlyDataCommand,
    {
        // Transform command into a fixed size array of 7 u8 and the real length for sending
        let (command, data, len) = match self {
            Command::CommandLock(val) => (0xFD, [val, 0, 0, 0, 0, 0], 1),
            Command::DisplayOn(val) => (if val { 0xAF } else { 0xAE }, [0, 0, 0, 0, 0, 0], 0),
            Command::ClockDiv(val) => (0xB3, [val, 0, 0, 0, 0, 0], 1),
            Command::MuxRatio(val) => (0xCA, [val, 0, 0, 0, 0, 0], 1),
            Command::SetRemap(incr, remap, scan) => (
                0xA0,
                [
                    0b00100100 | (incr as u8) | (remap as u8) << 1 | (scan as u8) << 4,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                1,
            ),
            Command::Column(start, end) => (0x15, [start, end, 0, 0, 0, 0], 2),
            Command::Row(start, end) => (0x75, [start, end, 0, 0, 0, 0], 2),
            Command::StartLine(val) => (0xA1, [val, 0, 0, 0, 0, 0], 1),
            Command::DisplayOffset(val) => (0xA2, [val, 0, 0, 0, 0, 0], 1),
            Command::SetGpio(val) => (0xB5, [val, 0, 0, 0, 0, 0], 1),
            Command::FunctionSelect(val) => (0xAB, [val, 0, 0, 0, 0, 0], 1),
            Command::PreCharge(val) => (0xB1, [val, 0, 0, 0, 0, 0], 1),
            Command::Vcomh(val) => (0xBE, [val, 0, 0, 0, 0, 0], 1),
            Command::Invert(val) => (if val { 0xA7 } else { 0xA6 }, [0, 0, 0, 0, 0, 0], 0),
            Command::Contrast(val) => (0xC1, [0xC8, val, 0xC8, 0, 0, 0], 3),
            Command::ContrastCurrent(val) => (0xC7, [val, 0, 0, 0, 0, 0], 1),
            Command::SetVsl => (0xB4, [0xA0, 0xB5, 0x55, 0, 0, 0], 3),
            Command::PreCharge2(val) => (0xB6, [val, 0, 0, 0, 0, 0], 1),
            Command::WriteRam => (0x5C, [0, 0, 0, 0, 0, 0], 0),
        };

        // Send command over the interface
        iface.send_commands(DataFormat::U8(&[command])).await?;

        if len > 0 {
            iface.send_data(DataFormat::U8(&data[0..len])).await?;
        }

        Ok(())
    }
}