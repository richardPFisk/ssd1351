//! True async SPI interface for SSD1351 using embedded_hal_async

use display_interface::{DataFormat, DisplayError};
use embedded_hal_async::spi::SpiDevice;
use embedded_hal::digital::OutputPin;

use crate::async_interface::AsyncWriteOnlyDataCommand;

/// True async SPI interface that uses embedded_hal_async::spi::SpiDevice
pub struct AsyncSPIInterface<SPI, DC> {
    spi: SPI,
    dc: DC,
}

impl<SPI, DC> AsyncSPIInterface<SPI, DC> {
    pub fn new(spi: SPI, dc: DC) -> Self {
        Self { spi, dc }
    }
}

impl<SPI, DC> AsyncWriteOnlyDataCommand for AsyncSPIInterface<SPI, DC>
where
    SPI: SpiDevice,
    DC: OutputPin,
{
    async fn send_commands(&mut self, cmd: DataFormat<'_>) -> Result<(), DisplayError> {
        // Set DC low for command mode
        self.dc.set_low().map_err(|_| DisplayError::DCError)?;
        
        match cmd {
            DataFormat::U8(data) => {
                self.spi.write(data).await.map_err(|_| DisplayError::BusWriteError)?;
            }
            DataFormat::U16(data) => {
                // Convert u16 data to bytes and send
                let mut buffer = [0u8; 2];
                for &word in data {
                    buffer[0] = (word >> 8) as u8;
                    buffer[1] = (word & 0xFF) as u8;
                    self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
            DataFormat::U16BE(data) => {
                // Big-endian u16 data
                let mut buffer = [0u8; 2];
                for &word in data {
                    buffer[0] = (word >> 8) as u8;
                    buffer[1] = (word & 0xFF) as u8;
                    self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
            DataFormat::U16LE(data) => {
                // Little-endian u16 data
                let mut buffer = [0u8; 2];
                for &word in data {
                    buffer[0] = (word & 0xFF) as u8;
                    buffer[1] = (word >> 8) as u8;
                    self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
            DataFormat::U8Iter(iter) => {
                // Handle iterator efficiently by collecting into chunks
                let mut buffer = [0u8; 64]; // Small buffer for chunked writes
                let mut count = 0;
                
                for byte in iter {
                    buffer[count] = *byte;
                    count += 1;
                    
                    if count == buffer.len() {
                        self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                        count = 0;
                    }
                }
                
                // Send remaining bytes
                if count > 0 {
                    self.spi.write(&buffer[..count]).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
            DataFormat::U16BEIter(iter) => {
                let mut buffer = [0u8; 128]; // Buffer for chunked writes (64 u16 values)
                let mut count = 0;
                
                for word in iter {
                    buffer[count] = (*word >> 8) as u8;
                    buffer[count + 1] = (*word & 0xFF) as u8;
                    count += 2;
                    
                    if count == buffer.len() {
                        self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                        count = 0;
                    }
                }
                
                // Send remaining bytes
                if count > 0 {
                    self.spi.write(&buffer[..count]).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
            DataFormat::U16LEIter(iter) => {
                let mut buffer = [0u8; 128]; // Buffer for chunked writes
                let mut count = 0;
                
                for word in iter {
                    buffer[count] = (*word & 0xFF) as u8;
                    buffer[count + 1] = (*word >> 8) as u8;
                    count += 2;
                    
                    if count == buffer.len() {
                        self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                        count = 0;
                    }
                }
                
                // Send remaining bytes
                if count > 0 {
                    self.spi.write(&buffer[..count]).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
        }
        
        Ok(())
    }
    
    async fn send_data(&mut self, buf: DataFormat<'_>) -> Result<(), DisplayError> {
        // Set DC high for data mode
        self.dc.set_high().map_err(|_| DisplayError::DCError)?;
        
        // Reuse the same logic as send_commands but with DC high
        match buf {
            DataFormat::U8(data) => {
                self.spi.write(data).await.map_err(|_| DisplayError::BusWriteError)?;
            }
            DataFormat::U16(data) => {
                let mut buffer = [0u8; 2];
                for &word in data {
                    buffer[0] = (word >> 8) as u8;
                    buffer[1] = (word & 0xFF) as u8;
                    self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
            DataFormat::U16BE(data) => {
                let mut buffer = [0u8; 2];
                for &word in data {
                    buffer[0] = (word >> 8) as u8;
                    buffer[1] = (word & 0xFF) as u8;
                    self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
            DataFormat::U16LE(data) => {
                let mut buffer = [0u8; 2];
                for &word in data {
                    buffer[0] = (word & 0xFF) as u8;
                    buffer[1] = (word >> 8) as u8;
                    self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
            DataFormat::U8Iter(iter) => {
                let mut buffer = [0u8; 64];
                let mut count = 0;
                
                for byte in iter {
                    buffer[count] = *byte;
                    count += 1;
                    
                    if count == buffer.len() {
                        self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                        count = 0;
                    }
                }
                
                if count > 0 {
                    self.spi.write(&buffer[..count]).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
            DataFormat::U16BEIter(iter) => {
                let mut buffer = [0u8; 128];
                let mut count = 0;
                
                for word in iter {
                    buffer[count] = (*word >> 8) as u8;
                    buffer[count + 1] = (*word & 0xFF) as u8;
                    count += 2;
                    
                    if count == buffer.len() {
                        self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                        count = 0;
                    }
                }
                
                if count > 0 {
                    self.spi.write(&buffer[..count]).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
            DataFormat::U16LEIter(iter) => {
                let mut buffer = [0u8; 128];
                let mut count = 0;
                
                for word in iter {
                    buffer[count] = (*word & 0xFF) as u8;
                    buffer[count + 1] = (*word >> 8) as u8;
                    count += 2;
                    
                    if count == buffer.len() {
                        self.spi.write(&buffer).await.map_err(|_| DisplayError::BusWriteError)?;
                        count = 0;
                    }
                }
                
                if count > 0 {
                    self.spi.write(&buffer[..count]).await.map_err(|_| DisplayError::BusWriteError)?;
                }
            }
        }
        
        Ok(())
    }
}