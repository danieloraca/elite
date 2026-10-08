# Wirespace

A playable Rust tribute to wireframe space trading games of the 1980s. Fly to a rotating station, trade between six systems, and defend your ship from pirates. The names and generated geometry are original.

## Play in a browser

Build the Rust game as WebAssembly and collect the complete static site:

```sh
./scripts/build-web.sh
python3 -m http.server 8080 --directory dist
```

Open `http://localhost:8080`, click the game to focus it, then press Enter. The machine serving the files does not run the game logic; the browser does. Keep `index.html`, `elite.wasm`, `mq_js_bundle.js`, `app.js`, and `styles.css` together when deploying.

To host it on a Raspberry Pi, build on your development machine and copy the contents of `dist/` to a directory served by nginx, Caddy, or another static HTTP server on the Pi. Serve `.wasm` as `application/wasm`. The Pi does not need a Rust toolchain for hosting.

## Play as a desktop app

```sh
cargo run --release
```

## Flight controls

- Enter or Space: start from the title screen
- W / S: increase / decrease throttle
- Up / Down: pitch
- Left / Right: roll
- A / D: yaw
- Space: fire laser
- E: dock when aligned, close, and below 30 speed
- Tab: select the next hyperspace destination
- J: jump (uses one fuel unit)
- P: pause or resume

## Station controls

- Up / Down: select commodity
- B / V: buy / sell one tonne
- F: buy one fuel unit for 25 credits
- U: undock

You start facing the station. Use W to approach, S to slow down, and E when the docking indicator appears. Buy goods at one station and sell where the price is higher. Pirates carry an 80 credit bounty. Press R after defeat to return to the title screen.
# elite
