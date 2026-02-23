// DW3000-family register definitions.
//
// Each register is identified by a 5-bit register file ID (base address 0x00–0x1F)
// and a 7-bit sub-address (offset within that register file, 0x00–0x7F).
// Sizes are in bytes.
//
// Reference: DW3000 User Manual, Section 8 — Register Set.

/// A register descriptor: base ID, sub-address, and byte length.
#[derive(Clone, Copy)]
pub struct Register {
  pub id: u8,
  pub sub: u8,
  pub len: usize,
}

// ── General device info ──────────────────────────────────────────────────────

/// Device identifier (read-only). Returns chip model / version / lot.
pub const DEV_ID: Register = Register { id: 0x00, sub: 0x00, len: 4 };

/// Extended unique identifier (EUI-64), 8 bytes.
pub const EUI_64: Register = Register { id: 0x00, sub: 0x04, len: 8 };

// ── System configuration ─────────────────────────────────────────────────────

/// System configuration register.
pub const SYS_CFG: Register = Register { id: 0x00, sub: 0x10, len: 4 };

// ── TX control ───────────────────────────────────────────────────────────────

/// Transmit frame control — sets payload length, data rate, PRF, preamble length.
pub const TX_FCTRL: Register = Register { id: 0x00, sub: 0x24, len: 4 };

// ── Status ───────────────────────────────────────────────────────────────────

/// System status register — flags for TX done, RX done, errors, etc.
pub const SYS_STATUS: Register = Register { id: 0x00, sub: 0x44, len: 4 };

// SYS_STATUS bit masks
pub const SYS_STATUS_TXFRS: u32 = 1 << 7; // TX frame sent
pub const SYS_STATUS_RXDFR: u32 = 1 << 13; // RX data frame ready
pub const SYS_STATUS_RXFCG: u32 = 1 << 14; // RX FCS good
pub const SYS_STATUS_RXFCE: u32 = 1 << 15; // RX FCS error
pub const SYS_STATUS_RXPHE: u32 = 1 << 12; // RX PHY header error
pub const SYS_STATUS_RXPTO: u32 = 1 << 21; // RX preamble detection timeout
pub const SYS_STATUS_RXSFDTO: u32 = 1 << 26; // RX SFD timeout

// ── RX info ──────────────────────────────────────────────────────────────────

/// Receive frame info — frame length, ranging flag, etc.
pub const RX_FINFO: Register = Register { id: 0x00, sub: 0x4C, len: 4 };

/// Mask for the frame length field within RX_FINFO (bits 0–9, 10-bit value).
pub const RX_FINFO_RXFLEN_MASK: u32 = 0x3FF;

// ── Data buffers ─────────────────────────────────────────────────────────────

/// Transmit data buffer (write-only). Up to 1024 bytes.
pub const TX_BUFFER: Register = Register { id: 0x14, sub: 0x00, len: 1024 };

/// Receive data buffer (read-only). Up to 1024 bytes.
pub const RX_BUFFER: Register = Register { id: 0x12, sub: 0x00, len: 1024 };

// ── Fast commands ────────────────────────────────────────────────────────────
//
// Fast commands are single-byte SPI transactions that trigger immediate actions.
// Format: [1][CMD4:0][1]  (bit 7 = 1, bits 6:2 = command code, bit 0 = 1)

pub const CMD_TX: u8 = 0x01; // Start transmission
pub const CMD_RX: u8 = 0x02; // Enable receiver
pub const CMD_TXRXOFF: u8 = 0x03; // Abort TX/RX, return to idle
pub const CMD_DRX: u8 = 0x04; // Double-buffered RX mode (delayed)
pub const CMD_CLR_IRQS: u8 = 0x0E; // Clear all interrupt flags
