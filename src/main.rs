#[cfg(target_arch = "wasm32")]
mod browser;
mod game;
mod sound;

use game::{Game, HEIGHT, WIDTH};
use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Wirespace".into(),
        window_width: WIDTH as i32,
        window_height: HEIGHT as i32,
        window_resizable: false,
        high_dpi: true,
        ..Default::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if std::env::args().any(|arg| arg == "--export-music") {
        let path = "target/wirespace-theme.wav";
        std::fs::create_dir_all("target").expect("create target directory");
        std::fs::write(path, sound::music_wav()).expect("write generated theme");
        println!("Wrote {path}");
        return;
    }
    launch();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    launch();
}

fn launch() {
    macroquad::Window::from_config(window_conf(), run());
}

async fn run() {
    let mut game = Game::new();
    let mut audio = sound::Audio::new().await;
    loop {
        if is_key_pressed(KeyCode::M) {
            audio.toggle();
        }
        #[cfg(target_arch = "wasm32")]
        if browser::take_pause_request() {
            game.pause_for_hidden_tab();
        }
        game.update(get_frame_time());
        audio.sync_music(game.music_active());
        for event in game.take_sounds() {
            audio.play(event);
        }
        set_camera(&Camera2D {
            target: vec2(WIDTH * 0.5, HEIGHT * 0.5),
            zoom: vec2(2.0 / WIDTH, 2.0 / HEIGHT),
            ..Default::default()
        });
        game.render(audio.muted);
        next_frame().await;
    }
}
