#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_dir"

cargo build --release --target wasm32-unknown-unknown --locked
mkdir -p dist
cp web/index.html web/styles.css web/app.js web/favicon.svg web/mq_js_bundle.js dist/
cp web/THIRD_PARTY.md dist/
cp -R web/licenses dist/
cp target/wasm32-unknown-unknown/release/elite.wasm dist/elite.wasm

echo "Web build ready in $project_dir/dist"
