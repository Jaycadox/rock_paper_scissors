use egui_macroquad::egui::{self, RichText, Slider};
use egui_macroquad::macroquad::prelude::*;

use crate::client_game::ClientGameState;
use crate::GameState;

pub struct MainMenuState {
    population: usize,
}

impl MainMenuState {
    pub fn new() -> Self {
        Self { population: 50 }
    }

    pub async fn tick(mut self) -> GameState {
        let mut new_gamestate = None;
        let mut load_game = false;
        egui_macroquad::ui(|ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::dark_canvas(&ctx.style()))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Rock-Paper-Scissors").size(32.0));
                    });

                    let window_pos_x = (screen_width() - 200.0) / 2.0;
                    let window_pos_y = (screen_height() - 200.0) / 2.0;

                    egui::Window::new("Play")
                        .fixed_pos((window_pos_x, window_pos_y))
                        .fixed_size((200.0, 200.0))
                        .collapsible(false)
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("Population");
                                ui.add(Slider::new(&mut self.population, 1..=10000));
                            });
                            ui.separator();
                            if ui.button("Begin simulation").clicked() {
                                load_game = true;
                            }
                        });
                });
        });
        egui_macroquad::draw();

        if load_game {
            new_gamestate = Some(GameState::InGame(
                ClientGameState::new(self.population).await,
            ));
        }

        if let Some(new_gamestate) = new_gamestate {
            new_gamestate
        } else {
            GameState::MainMenu(self)
        }
    }
}
