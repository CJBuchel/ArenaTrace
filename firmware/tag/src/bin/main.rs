#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_nrf::bind_interrupts;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::peripherals;
use embassy_nrf::spim;
use embassy_nrf::spim::{Config, Frequency};
use embassy_time::{Duration, Ticker};
use tag::spi::SpiDevice;

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
  let p = embassy_nrf::init(Default::default());

  // SPI config
  let mut config = Config::default();
  config.frequency = Frequency::M8;

  let spi = spim::Spim::new(
    p.SPI3, Irqs, p.P0_16, // SCK
    p.P0_21, // MISO
    p.P0_29, // MOSI
    config,
  );

  let cs = Output::new(p.P0_17, Level::High, OutputDrive::Standard);

  let mut device = SpiDevice::new(spi, cs);

  let mut ticker = Ticker::every(Duration::from_hz(1));

  loop {
    ticker.next().await;

    let tx = [0xAA, 0x55, 0xAA, 0x55];
    let mut rx = [0u8; 4];

    device.transfer(&tx, &mut rx).await.unwrap();
  }
}
