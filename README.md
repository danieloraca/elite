# Wirespace

A playable Rust tribute to wireframe space trading games of the 1980s. Fly to a rotating station, trade between six systems, and defend your ship from pirates. The names and generated geometry are original.

The music and sound effects are original retro synths generated from Rust code. Music starts with the game; **M** mutes or unmutes it. Leaving the browser tab pauses the game and silences audio; press **P** to resume.

To export a WAV of the original theme for listening without opening a game window, run `cargo run --release -- --export-music`. The file is written to `target/wirespace-theme.wav`.

## Play in a browser

Build the Rust game as WebAssembly and collect the complete static site:

```sh
./scripts/build-web.sh
python3 -m http.server 8080 --directory dist
```

Open `http://localhost:8080`, click the game to focus it, then press Enter. The machine serving the files does not run the game logic; the browser does. Keep `index.html`, `elite.wasm`, `mq_js_bundle.js`, `app.js`, and `styles.css` together when deploying.

## Run on the Raspberry Pi

The Pi runs a small Rust HTTP server as `wirespace.service` on port **3141**. The game itself runs in each player's browser. Like Dario's deploy flow, the browser build is made locally and the server is built on the Pi; the Pi needs Rust installed for deployment.

From this checkout, with SSH and sudo access to the Pi:

```sh
sh scripts/deploy-pi.sh
```

The default SSH destination is `danutz@192.168.0.25`. Set `PI_HOST=user@host` to use another address. The service unit assumes the Pi account and install path are `danutz` and `/home/danutz/Development/elite`; adjust `deploy/wirespace.service` and `deploy/install.sh` together if those differ.

After deployment, open `http://192.168.0.25:3141/` on the local network. The deploy script checks that port 3141 is available, enables the service at boot, and verifies the page and WASM content type. To inspect it on the Pi:

```sh
systemctl status wirespace.service
journalctl -u wirespace.service -n 50 --no-pager
```

For a local server check after `./scripts/build-web.sh`, run `cargo run --release -p wirespace-server` from the project root and open `http://127.0.0.1:3141/`.

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
- M: mute or unmute music and effects

## Station controls

- Up / Down: select commodity
- B / V: buy / sell one tonne
- F: buy one fuel unit for 25 credits
- U: undock

You start facing the station. Use W to approach, S to slow down, and E when the docking indicator appears. Buy goods at one station and sell where the price is higher. Pirates carry an 80 credit bounty. Press R after defeat to return to the title screen.
# elite
