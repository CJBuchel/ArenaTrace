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
use embassy_time::{Duration, Ticker, Timer};
use tag::spi::SpiDevice;

bind_interrupts!(struct Irqs {
    SPIM3 => spim::InterruptHandler<peripherals::SPI3>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
  let p = embassy_nrf::init(Default::default());

  // SPI config — DW3000 supports up to 38 MHz, 8 MHz is safe for startup
  let mut config = Config::default();
  config.frequency = Frequency::M8;

  let spi = spim::Spim::new(
    p.SPI3, Irqs, p.P0_16, // SCK
    p.P0_21, // MISO
    p.P0_29, // MOSI
    config,
  );

  let cs = Output::new(p.P0_17, Level::High, OutputDrive::Standard);
  let spi_device = SpiDevice::new(spi, cs);

  // Initialize DW3000 — reads DEV_ID to verify SPI link
  let mut delay = embassy_time::Delay;
  let dw = dw3000::DW3000::new(spi_device);
  let mut dw = match dw.init(&mut delay).await {
    Ok(dw) => dw,
    Err(e) => {
      defmt::error!("DW3000 init failed: {}", e);
      loop {
        Timer::after(Duration::from_secs(1)).await;
      }
    }
  };

  defmt::info!("DW3000 initialized, starting 60Hz TX loop");

  let mut ticker = Ticker::every(Duration::from_hz(60));
  let mut seq: u8 = 0;

  loop {
    ticker.next().await;

    // Build a small blink payload: [sequence_number]
    let payload = [seq];
    seq = seq.wrapping_add(1);

    if let Err(e) = dw.send(&payload).await {
      defmt::warn!("TX failed: {}", e);
    }
  }
}
