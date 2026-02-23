use embassy_nrf::gpio::Output;
use embassy_nrf::spim::{self, Spim};
use embedded_hal_async::spi::{self as hal_spi, Operation};

/// SPI device wrapper that implements `embedded_hal_async::spi::SpiDevice`.
///
/// Pairs an Embassy nRF SPIM peripheral with a CS (chip select) GPIO pin.
/// The `transaction()` method asserts CS low, executes all operations, then
/// deasserts CS — matching the contract that `SpiDevice` requires.
pub struct SpiDevice<'a> {
  spi: Spim<'a>,
  cs: Output<'a>,
}

impl<'a> SpiDevice<'a> {
  pub fn new(spi: Spim<'a>, cs: Output<'a>) -> Self {
    Self { spi, cs }
  }
}

impl<'a> hal_spi::ErrorType for SpiDevice<'a> {
  type Error = spim::Error;
}

impl<'a> hal_spi::SpiDevice for SpiDevice<'a> {
  async fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
    self.cs.set_low();

    let result = async {
      for op in operations {
        match op {
          Operation::Read(buf) => self.spi.read(buf).await?,
          Operation::Write(data) => self.spi.write(data).await?,
          Operation::Transfer(read, write) => self.spi.transfer(read, write).await?,
          Operation::TransferInPlace(buf) => self.spi.transfer_in_place(buf).await?,
          Operation::DelayNs(_ns) => {} // not needed for DW3000 communication
        }
      }
      Ok(())
    }
    .await;

    self.cs.set_high();
    result
  }
}
