// High-level driver for DW3000-family UWB transceivers.
//
// Uses a type-state pattern so the compiler enforces valid state transitions:
//
//   Uninitialized --init()--> Ready --send()--> Sending --wait_sent()--> Ready
//                               |                                         ^
//                               +--receive()--> Receiving --wait_recv()----+
//
// Any active state (Sending/Receiving) can call force_idle() to abort back to Ready.

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::spi::SpiDevice;

use crate::error::Error;
use crate::ll;
use crate::registers;

// ── Type-state markers (zero-size) ──────────────────────────────────────────

pub struct Uninitialized;
pub struct Ready;
pub struct Sending;
pub struct Receiving;

// ── Driver struct ───────────────────────────────────────────────────────────

/// DW3000-family UWB transceiver driver.
///
/// Generic over:
/// - `SPI`: any `embedded_hal_async::spi::SpiDevice` implementation
/// - `STATE`: compile-time state tracking (Uninitialized, Ready, Sending, Receiving)
pub struct DW3000<SPI, STATE> {
  spi: SPI,
  _state: STATE,
}

// Transition helper — moves the SPI peripheral into a new state without copying.
impl<SPI, STATE> DW3000<SPI, STATE> {
  fn into_state<S>(self, state: S) -> DW3000<SPI, S> {
    DW3000 { spi: self.spi, _state: state }
  }
}

// ── Uninitialized ───────────────────────────────────────────────────────────

impl<SPI: SpiDevice> DW3000<SPI, Uninitialized> {
  /// Wrap an SPI device. The chip is assumed to be powered but not yet configured.
  pub fn new(spi: SPI) -> Self {
    DW3000 { spi, _state: Uninitialized }
  }

  /// Initialize the DW3000.
  ///
  /// 1. Sends TXRXOFF fast command to ensure idle state
  /// 2. Reads DEV_ID to verify SPI communication
  /// 3. Logs the device ID via defmt
  ///
  /// Returns `Ready` state on success, or `UnexpectedDeviceId` if DEV_ID reads as 0 or 0xFFFFFFFF
  /// (which indicates SPI wiring or clock issues).
  pub async fn init(mut self, delay: &mut impl DelayNs) -> Result<DW3000<SPI, Ready>, Error<SPI::Error>> {
    // Small delay after power-up to let the chip stabilize
    delay.delay_ms(5).await;

    // Force idle
    ll::fast_command(&mut self.spi, registers::CMD_TXRXOFF).await?;
    delay.delay_ms(1).await;

    // Read device ID to verify SPI link
    let dev_id = ll::read_reg_u32(&mut self.spi, registers::DEV_ID).await?;
    defmt::info!("DW3000 DEV_ID: {:#010X}", dev_id);

    // Sanity check — 0x00000000 or 0xFFFFFFFF means SPI isn't working
    if dev_id == 0x00000000 || dev_id == 0xFFFFFFFF {
      return Err(Error::UnexpectedDeviceId(dev_id));
    }

    // Clear any pending interrupts
    ll::fast_command(&mut self.spi, registers::CMD_CLR_IRQS).await?;

    Ok(self.into_state(Ready))
  }
}

// ── Ready ───────────────────────────────────────────────────────────────────

impl<SPI: SpiDevice> DW3000<SPI, Ready> {
  /// Transmit a frame.
  ///
  /// 1. Writes `data` into TX_BUFFER
  /// 2. Sets TX_FCTRL with the frame length
  /// 3. Issues CMD_TX fast command
  ///
  /// Transitions to `Sending` state. Call `wait_sent()` to block until transmission completes.
  pub async fn send(&mut self, data: &[u8]) -> Result<(), Error<SPI::Error>> {
    // Write payload to TX buffer
    ll::write_reg(&mut self.spi, registers::TX_BUFFER, data).await?;

    // Set frame length in TX_FCTRL (bits 0–9 = frame length including 2-byte FCS added by hardware)
    let frame_len = (data.len() + 2) as u32; // +2 for the FCS the DW3000 appends
    let tx_fctrl = frame_len & 0x3FF;
    ll::write_reg_u32(&mut self.spi, registers::TX_FCTRL, tx_fctrl).await?;

    // Start transmission
    ll::fast_command(&mut self.spi, registers::CMD_TX).await?;

    // Poll for TX complete
    for _ in 0..10_000u32 {
      let status = ll::read_reg_u32(&mut self.spi, registers::SYS_STATUS).await?;
      if status & registers::SYS_STATUS_TXFRS != 0 {
        // Clear the TX done flag
        ll::write_reg_u32(&mut self.spi, registers::SYS_STATUS, registers::SYS_STATUS_TXFRS).await?;
        return Ok(());
      }
    }
    Err(Error::Timeout)
  }

  /// Enable the receiver and wait for a frame.
  ///
  /// Issues CMD_RX, then polls until a frame with good FCS arrives.
  /// Reads the frame from RX_BUFFER into `buf` and returns the number of payload bytes.
  ///
  /// `max_polls`: maximum number of status reads before returning `Timeout`.
  pub async fn recv(&mut self, buf: &mut [u8], max_polls: u32) -> Result<usize, Error<SPI::Error>> {
    ll::fast_command(&mut self.spi, registers::CMD_RX).await?;

    for _ in 0..max_polls {
      let status = ll::read_reg_u32(&mut self.spi, registers::SYS_STATUS).await?;

      if status & registers::SYS_STATUS_RXFCG != 0 {
        // Good frame received — read its length from RX_FINFO
        let finfo = ll::read_reg_u32(&mut self.spi, registers::RX_FINFO).await?;
        let frame_len = (finfo & registers::RX_FINFO_RXFLEN_MASK) as usize;

        // Subtract the 2-byte FCS to get payload length
        let payload_len = if frame_len >= 2 { frame_len - 2 } else { 0 };
        let read_len = payload_len.min(buf.len());

        // Read payload from RX buffer
        ll::read_reg(&mut self.spi, registers::RX_BUFFER, &mut buf[..read_len]).await?;

        // Clear RX status flags
        let clear_mask = registers::SYS_STATUS_RXDFR | registers::SYS_STATUS_RXFCG;
        ll::write_reg_u32(&mut self.spi, registers::SYS_STATUS, clear_mask).await?;

        return Ok(read_len);
      }

      // Check for RX errors — clear and re-enable receiver
      let rx_err = registers::SYS_STATUS_RXFCE | registers::SYS_STATUS_RXPHE;
      if status & rx_err != 0 {
        ll::write_reg_u32(&mut self.spi, registers::SYS_STATUS, rx_err).await?;
        ll::fast_command(&mut self.spi, registers::CMD_RX).await?;
      }
    }

    // Timed out — return to idle
    ll::fast_command(&mut self.spi, registers::CMD_TXRXOFF).await?;
    Err(Error::Timeout)
  }

  /// Force the chip to idle (useful if you need to cancel an operation that was
  /// started outside this driver, or to ensure a clean state).
  pub async fn force_idle(&mut self) -> Result<(), Error<SPI::Error>> {
    ll::fast_command(&mut self.spi, registers::CMD_TXRXOFF).await
  }
}
