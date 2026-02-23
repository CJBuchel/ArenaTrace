// Low-level SPI access for DW3000-family chips.
//
// Implements the DW3000 SPI protocol:
//   - Full addressed transactions (2-byte header + data) for register read/write
//   - Fast commands (1-byte) for immediate actions like TX/RX start
//
// Reference: DW3000 Datasheet, SPI Transaction Formatting.

use embedded_hal_async::spi::SpiDevice;

use crate::error::Error;
use crate::registers::Register;

/// Build a 2-byte full-addressed SPI header.
///
/// Byte 0: `[W/R][1][ADDR4:0][SUB_MSB]`
/// Byte 1: `[SUB5:0][M1=0][M0=0]`
///
/// - `write`: true for write, false for read
/// - `reg`: the register descriptor (contains ID and sub-address)
fn build_header(write: bool, reg: Register) -> [u8; 2] {
  let b0 = ((write as u8) << 7) // bit 7: R/W
    | (1 << 6)                   // bit 6: always 1 for full-addressed
    | ((reg.id & 0x1F) << 1)     // bits 5:1: 5-bit base address
    | ((reg.sub >> 6) & 0x01); // bit 0: MSB of 7-bit sub-address

  let b1 = (reg.sub & 0x3F) << 2; // bits 7:2: remaining 6 bits of sub-address
  // bits 1:0: M1,M0 = 0,0 (standard mode)

  [b0, b1]
}

/// Read bytes from a register.
///
/// Sends a 2-byte read header, then clocks out `buf.len()` bytes of data from the device.
pub async fn read_reg<SPI: SpiDevice>(spi: &mut SPI, reg: Register, buf: &mut [u8]) -> Result<(), Error<SPI::Error>> {
  let header = build_header(false, reg);

  // SpiDevice::transaction handles CS assert/deassert.
  // We write the header, then read the data — two separate operations in one CS frame.
  spi
    .transaction(&mut [
      embedded_hal_async::spi::Operation::Write(&header),
      embedded_hal_async::spi::Operation::Read(buf),
    ])
    .await
    .map_err(Error::Spi)
}

/// Write bytes to a register.
///
/// Sends a 2-byte write header followed by the data bytes, all in one CS frame.
pub async fn write_reg<SPI: SpiDevice>(spi: &mut SPI, reg: Register, data: &[u8]) -> Result<(), Error<SPI::Error>> {
  let header = build_header(true, reg);

  spi
    .transaction(&mut [
      embedded_hal_async::spi::Operation::Write(&header),
      embedded_hal_async::spi::Operation::Write(data),
    ])
    .await
    .map_err(Error::Spi)
}

/// Issue a fast command.
///
/// Fast commands are single-byte SPI transactions: `[1][CMD4:0][1]`
/// They trigger immediate actions (start TX, start RX, idle, etc.)
pub async fn fast_command<SPI: SpiDevice>(spi: &mut SPI, cmd: u8) -> Result<(), Error<SPI::Error>> {
  let byte = (1 << 7) | ((cmd & 0x1F) << 1) | 1;

  spi.transaction(&mut [embedded_hal_async::spi::Operation::Write(&[byte])]).await.map_err(Error::Spi)
}

/// Read a 32-bit register value (little-endian, as the DW3000 uses LE byte order).
pub async fn read_reg_u32<SPI: SpiDevice>(spi: &mut SPI, reg: Register) -> Result<u32, Error<SPI::Error>> {
  let mut buf = [0u8; 4];
  read_reg(spi, reg, &mut buf).await?;
  Ok(u32::from_le_bytes(buf))
}

/// Write a 32-bit value to a register (little-endian).
pub async fn write_reg_u32<SPI: SpiDevice>(spi: &mut SPI, reg: Register, val: u32) -> Result<(), Error<SPI::Error>> {
  let bytes = val.to_le_bytes();
  write_reg(spi, reg, &bytes).await
}
