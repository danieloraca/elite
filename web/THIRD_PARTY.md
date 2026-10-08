# Browser loader

`mq_js_bundle.js` is copied from `macroquad` 0.4.16. It contains the JavaScript bridge used by the Rust/WASM game, including code from `miniquad` 0.4.11 and `quad-snd` 0.2.8. These crates are dual licensed under MIT or Apache-2.0. The license texts are in `licenses/`.

The vendored bundle has one local fix: the `quad_net` plugin's `register_plugin` assignment is declared with `var` so it works in strict mode without a browser console error.

Sources:

- https://github.com/not-fl3/macroquad/tree/v0.4.16/js
- https://github.com/not-fl3/miniquad
- https://github.com/not-fl3/quad-snd
