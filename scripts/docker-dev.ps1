# Run cargo inside the official Rust Docker image.
#
# Why: on this Windows host, Smart App Control (WDAC) blocks execution of the
# unsigned native build-script .exe files cargo produces (os error 4551), and
# no MSVC/mingw linker is available. Building in a Linux container sidesteps
# both. Named volumes cache the toolchain, cargo registry and target dir so
# repeat runs are fast. `--network host` avoids flaky container egress.
#
# Usage:
#   ./scripts/docker-dev.ps1 build --all-targets
#   ./scripts/docker-dev.ps1 test
#   ./scripts/docker-dev.ps1 clippy --all-targets --all-features -- -D warnings
#   ./scripts/docker-dev.ps1 fmt --all --check

param([Parameter(ValueFromRemainingArguments = $true)] [string[]] $CargoArgs)

$repo = Split-Path -Parent $PSScriptRoot

docker run --rm --network host `
  -v "${repo}:/work" -w /work `
  -v tableflow-rustup:/usr/local/rustup `
  -v tableflow-cargo:/usr/local/cargo/registry `
  -v tableflow-target:/tmp/target `
  -e CARGO_TARGET_DIR=/tmp/target `
  rust:latest bash -c "rustup component add rustfmt clippy >/dev/null 2>&1; cargo $($CargoArgs -join ' ')"
