/// Errors that can occur when communicating with a DW3000-family chip.
#[derive(Debug)]
pub enum Error<SPI> {
  /// SPI bus error (wraps the platform-specific SPI error type).
  Spi(SPI),

  /// DEV_ID register returned an unexpected value during init.
  UnexpectedDeviceId(u32),

  /// A transmit or receive operation timed out (SYS_STATUS never set the expected flag).
  Timeout,
}

// Manual defmt::Format impl — we can't derive it because the SPI error type
// may not implement Format. We log SPI errors as "SPI error" and use Debug
// for the rest.
impl<SPI: core::fmt::Debug> defmt::Format for Error<SPI> {
  fn format(&self, f: defmt::Formatter) {
    match self {
      Error::Spi(_) => defmt::write!(f, "SPI bus error"),
      Error::UnexpectedDeviceId(id) => defmt::write!(f, "Unexpected DEV_ID: {:#010X}", id),
      Error::Timeout => defmt::write!(f, "Timeout"),
    }
  }
}
