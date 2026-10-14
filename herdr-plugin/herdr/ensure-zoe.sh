#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
source_root=$(cd "$root/.." && pwd)
command -v cargo >/dev/null 2>&1 || { echo "cargo is required to build the fork's zoe" >&2; exit 1; }
[ -f "$source_root/Cargo.toml" ] && [ -d "$source_root/src/provider/pi" ] && [ -d "$source_root/src/provider/rusty" ] || {
  echo "install leo-labs-ai/zoetrope/herdr-plugin from the harness-providers branch" >&2
  exit 1
}
cargo build --release --manifest-path "$source_root/Cargo.toml" --bin zoe
mkdir -p "$root/bin"
cp "$source_root/target/release/zoe" "$root/bin/zoe"
echo "Built fork zoe at $root/bin/zoe"
