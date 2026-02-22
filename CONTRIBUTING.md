# Contributing

## Setup

1. Install Rust via [rustup](https://rustup.rs)
2. Add the embedded ARM target:
   ```sh
   rustup target add thumbv7em-none-eabihf
   ```
3. Clone the repo and build:
   ```sh
   git clone <repo>
   cd ArenaTrace
   cargo xtask build
   ```

That's it. No additional tools are required to build.

## Build system

This project uses `cargo xtask` as the build orchestrator. It is a plain Rust binary inside the workspace — nothing to install.

```sh
cargo xtask build            # build everything
cargo xtask build-firmware   # firmware only (tag + anchor)
cargo xtask build-server     # server only
```

`cargo build --workspace` does **not** work because the firmware targets ARM (`thumbv7em-none-eabihf`) while the server targets the host. `cargo xtask` handles this by invoking each package with the correct `--target` flag.

## Workspace layout

| Crate | Purpose | Target |
|---|---|---|
| `common` | Shared protocol types | `no_std`, any |
| `firmware/tag` | Tag firmware | `thumbv7em-none-eabihf` |
| `firmware/anchor` | Anchor firmware | `thumbv7em-none-eabihf` |
| `server` | Host server | host |
| `xtask` | Build orchestrator | host |

## Adding a new firmware target

1. Create the crate under `firmware/<name>/`
2. Add a `memory.x` with the chip's flash/RAM layout
3. Add a `.cargo/config.toml` setting the correct target triple
4. Add a `build.rs` that emits `cargo:rustc-link-search`
5. Add the crate to `members` in the workspace `Cargo.toml`
6. Add the package to `FIRMWARE_PACKAGES` in `xtask/src/main.rs` (and update `FIRMWARE_TARGET` if it's a different architecture)

## Versioning

Version is defined once in `Cargo.toml` under `[workspace.package]`. All crates inherit it via `version.workspace = true`. Bump it there and nowhere else.
