use std::collections::HashMap;
use std::io::Cursor;
use std::time::Instant;

use client_game::ClientGameState;
use egui_macroquad::macroquad;
use egui_macroquad::macroquad::prelude::*;
use egui_macroquad::macroquad::window::clear_background;

mod client_game;
mod main_menu;
use lazy_static::lazy_static;
use main_menu::MainMenuState;

enum GameState {
    MainMenu(MainMenuState),
    InGame(ClientGameState),
}

lazy_static! {
    static ref BUNDLE: HashMap<String, Vec<u8>> = {
        let mut m = HashMap::new();
        let bundle = include_bytes!("bundle.pfa");

        let mut reader = pfa::reader::PfaReader::new(Cursor::new(bundle)).unwrap();
        reader.traverse_files("/", |file| {
            println!("[Bundle] file={}", file.get_path());
            m.insert(file.get_path().to_string(), file.get_contents().to_vec());
        });

        m
    };
}

#[tokio::main]
async fn main() {
    macroquad::Window::new("Rock-Paper-Scissors", async move {
        let mut game_state = GameState::MainMenu(MainMenuState::new());
        let mut last_frame = Instant::now();
        loop {
            let this_frame = Instant::now();
            clear_background(BLACK);
            match game_state {
                GameState::MainMenu(main_menu) => {
                    game_state = main_menu.tick().await;
                }
                GameState::InGame(game) => {
                    game_state = game
                        .tick(this_frame.duration_since(last_frame).as_secs_f32())
                        .await;
                }
            };
            last_frame = this_frame;
            next_frame().await;
        }
    });
}
