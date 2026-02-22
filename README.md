# ArenaTrace

UWB-based arena tracking system. Tags worn by players communicate with fixed anchors over Ultra-Wideband radio. Anchor positions are collected by a server and surfaced to a frontend.

## Architecture

```
firmware/tag      — nRF52833 UWB tag (worn by players)
firmware/anchor   — nRF52833 UWB anchor (fixed positions)
server            — Native host server (collects anchor data)
common            — Shared protocol types (no_std, used by all)
xtask             — Build orchestrator (cargo xtask)
```

## Prerequisites

### Rust toolchain

Install Rust via [rustup](https://rustup.rs):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Add the embedded ARM target:

```sh
rustup target add thumbv7em-none-eabihf
```

### Flashing tools (optional, for deploying to hardware)

```sh
cargo install probe-rs-tools
```

## Building

All projects are built via `cargo xtask` from the workspace root. No Make, no external build tools required.

```sh
# Build everything
cargo xtask build

# Build firmware only (tag + anchor)
cargo xtask build-firmware

# Build server only
cargo xtask build-server
```

Individual projects can also be built directly from their directories:

```sh
cd firmware/tag && cargo build
cd firmware/anchor && cargo build
cd server && cargo build
```

## Project structure

```
ArenaTrace/
├── Cargo.toml              # Workspace root — shared deps and profiles
├── .cargo/config.toml      # ARM linker flags, xtask alias
├── common/                 # Shared no_std protocol types
├── firmware/
│   ├── tag/                # Tag firmware (nRF52833)
│   │   ├── memory.x        # nRF52833 memory map
│   │   └── .cargo/         # ARM target override
│   └── anchor/             # Anchor firmware (nRF52833)
│       ├── memory.x        # nRF52833 memory map
│       └── .cargo/         # ARM target override
├── server/                 # Host server
└── xtask/                  # Build orchestrator
```

## Versioning

Version is defined once in the workspace `Cargo.toml` and inherited by all crates. To update the version, change it in one place:

```toml
# Cargo.toml
[workspace.package]
version = "0.2.0"
```

`common::version()` exposes it at runtime for all projects.
