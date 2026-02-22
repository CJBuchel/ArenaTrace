use std::process::{Command, exit};

const FIRMWARE_TARGET: &str = "thumbv7em-none-eabihf";
const FIRMWARE_PACKAGES: &[&str] = &["tag", "anchor"];
const NATIVE_PACKAGES: &[&str] = &["server"];

fn cargo(args: &[&str]) {
  let status = Command::new("cargo").args(args).status().expect("failed to run cargo");
  if !status.success() {
    exit(status.code().unwrap_or(1));
  }
}

fn build_firmware() {
  for package in FIRMWARE_PACKAGES {
    println!("Building firmware: {package}");
    cargo(&["build", "-p", package, "--target", FIRMWARE_TARGET]);
  }
}

fn build_native() {
  for package in NATIVE_PACKAGES {
    println!("Building: {package}");
    cargo(&["build", "-p", package]);
  }
}

fn main() {
  let task = std::env::args().nth(1);
  match task.as_deref() {
    Some("build") | None => {
      build_firmware();
      build_native();
    }
    Some("build-firmware") => build_firmware(),
    Some("build-server") => build_native(),
    Some(unknown) => {
      eprintln!("Unknown task: {unknown}");
      eprintln!("Available tasks: build, build-firmware, build-server");
      exit(1);
    }
  }
}
