use embassy_nrf::gpio::Output;
use embassy_nrf::spim::Spim;

/// Generic SPI device wrapper
pub struct SpiDevice<'a> {
  spi: Spim<'a>,
  cs: Output<'a>,
}

impl<'a> SpiDevice<'a> {
  pub fn new(spi: Spim<'a>, cs: Output<'a>) -> Self {
    Self { spi, cs }
  }

  /// Write only
  pub async fn write(&mut self, bytes: &[u8]) -> Result<(), ()> {
    self.cs.set_low();
    let result = self.spi.write(bytes).await;
    self.cs.set_high();

    result.map_err(|_| ())
  }

  /// Read only
  pub async fn read(&mut self, buf: &mut [u8]) -> Result<(), ()> {
    self.cs.set_low();
    let result = self.spi.read(buf).await;
    self.cs.set_high();

    result.map_err(|_| ())
  }

  /// Full duplex transfer
  pub async fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<(), ()> {
    self.cs.set_low();
    let result = self.spi.transfer(rx, tx).await;
    self.cs.set_high();

    result.map_err(|_| ())
  }
}
