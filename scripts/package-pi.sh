#!/bin/sh
set -eu

task_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
sh "$task_root/scripts/build-web.sh"
tar -czf "$task_root/target/wirespace-pi.tar.gz" \
    -C "$task_root" Cargo.toml Cargo.lock build.rs src server dist deploy
printf 'Pi deployment archive: %s/target/wirespace-pi.tar.gz\n' "$task_root"
