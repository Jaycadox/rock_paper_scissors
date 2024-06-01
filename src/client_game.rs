use std::{collections::HashMap, time::Instant};

use egui_macroquad::macroquad::color::WHITE;
use egui_macroquad::macroquad::input::{is_key_pressed, mouse_position, KeyCode};
use egui_macroquad::macroquad::texture::draw_texture_ex;
use egui_macroquad::{
    egui,
    macroquad::{
        math::Vec2,
        prelude::ImageFormat,
        texture::{DrawTextureParams, Image, Texture2D},
        window::{screen_height, screen_width},
    },
};
use rand::{
    distributions::{Distribution, Standard},
    thread_rng, Rng,
};

use crate::{main_menu::MainMenuState, GameState, BUNDLE};

#[derive(Clone, PartialEq)]
enum Type {
    Rock,
    Paper,
    Scissor,
}

impl Distribution<Type> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Type {
        match rng.gen_range(0..=2) {
            0 => Type::Rock,
            1 => Type::Paper,
            _ => Type::Scissor,
        }
    }
}

#[derive(Clone)]
struct Player {
    team: Type,
    position: Vec2,
    attacking: Option<usize>,
    last_switched: Instant,
}

impl Player {
    fn new(team: Type, position: Vec2) -> Self {
        Self {
            team,
            position,
            attacking: None,
            last_switched: Instant::now(),
        }
    }
}

pub struct ClientGameState {
    players: Vec<Player>,
    rock: Texture2D,
    paper: Texture2D,
    scissor: Texture2D,
    //sound: Sound,
}

const COOLDOWN: u128 = 150;

impl ClientGameState {
    pub async fn new(player_count: usize) -> Self {
        let mut players = Vec::with_capacity(player_count);
        let screen_width = screen_width();
        let screen_height = screen_height();

        for _ in 0..player_count {
            players.push(Player::new(
                thread_rng().gen::<Type>(),
                Vec2::new(
                    thread_rng().gen_range(0.0..screen_width),
                    thread_rng().gen_range(0.0..screen_height),
                ),
            ));
        }

        let rock = Texture2D::from_image(&Image::from_file_with_format(
            BUNDLE.get("/rock.png").unwrap(),
            Some(ImageFormat::Png),
        ));

        let paper = Texture2D::from_image(&Image::from_file_with_format(
            BUNDLE.get("/paper.png").unwrap(),
            Some(ImageFormat::Png),
        ));

        let scissor = Texture2D::from_image(&Image::from_file_with_format(
            BUNDLE.get("/scissor.png").unwrap(),
            Some(ImageFormat::Png),
        ));

        //let sound = load_sound_from_bytes(BUNDLE.get("/hitHurt.ogg").unwrap())
        //    .await
        //    .unwrap();

        Self {
            players,
            rock,
            paper,
            scissor,
            //sound,
        }
    }
    pub async fn tick(mut self, dt: f32) -> GameState {
        let old_players = self.players.clone();
        let mut change_team_map = HashMap::<usize, Type>::new();
        for player in self.players.iter_mut() {
            // Random movement
            let shift_x = thread_rng().gen_range(-1.0..1.0);
            let shift_y = thread_rng().gen_range(-1.0..1.0);
            player.position.x += shift_x;
            player.position.y += shift_y;

            if player.position.x < 0.0 {
                player.position.x = 0.0;
            }

            if player.position.x > screen_width() {
                player.position.x = screen_width();
            }

            if player.position.y < 0.0 {
                player.position.y = 0.0;
            }

            if player.position.y > screen_height() {
                player.position.y = screen_height();
            }

            // Attacking
            let Some(idx) = player.attacking else {
                // Find closest player of different team
                let mut closest: Option<(usize, f32)> = None;
                for (i, old_player) in old_players.iter().enumerate() {
                    if let Some((_, old_record_dist)) = closest {
                        let dist = player.position.distance(old_player.position).abs();
                        if dist < old_record_dist
                            && old_player.team != player.team
                            && Instant::now()
                                .duration_since(old_player.last_switched)
                                .as_millis()
                                > COOLDOWN
                        {
                            closest = Some((i, dist));
                        }
                    } else {
                        closest = Some((i, player.position.distance(old_player.position).abs()));
                    }
                }
                if let Some((idx, _)) = closest {
                    player.attacking = Some(idx);
                } else {
                    println!("failed");
                }
                continue;
            };

            let Some(attacked) = old_players.get(idx) else {
                player.attacking = None;
                continue;
            };

            if attacked.team == player.team {
                player.attacking = None;
                continue;
            }

            let to_victim = attacked.position - player.position;
            if to_victim.length().abs() > 20.0 {
                let to_victim = to_victim.normalize() * 30.0 * dt;
                player.position += to_victim;

                if player.position.distance_squared(attacked.position).abs() < (30.0 * 30.0)
                    && Instant::now()
                        .duration_since(attacked.last_switched)
                        .as_millis()
                        > COOLDOWN
                {
                    change_team_map.insert(idx, player.team.clone());
                    player.attacking = None;
                }
            }

            // Push self back from players of same team that are too close
            for old_player in &old_players {
                if old_player.team != player.team {
                    continue;
                }

                if player.position.distance(old_player.position).abs() < 40.0 {
                    let to_player = player.position - old_player.position;
                    let mut to_player = to_player.normalize();
                    if !to_player.x.is_normal() || !to_player.y.is_normal() {
                        to_player = Vec2::new(1.0, 1.0);
                    }
                    player.position += to_player * dt * 20.0;
                }
            }
        }
        for (i, player) in self.players.iter_mut().enumerate() {
            // Change team
            if let Some(team) = change_team_map.get(&i) {
                // I'm unsure how to make this actually sound good, lol.
                //play_sound_once(self.sound);

                player.team = team.clone();
                player.last_switched = Instant::now();
                player.attacking = None;
            }
        }

        // Listen to input for manual player spawns
        let (mx, my) = mouse_position();
        let mouse = Vec2::new(mx, my);

        if is_key_pressed(KeyCode::R) {
            self.players.push(Player::new(Type::Rock, mouse));
        }

        if is_key_pressed(KeyCode::P) {
            self.players.push(Player::new(Type::Paper, mouse));
        }

        if is_key_pressed(KeyCode::S) {
            self.players.push(Player::new(Type::Scissor, mouse));
        }

        let mut new_game_state = None;
        self.render().await;
        egui_macroquad::ui(|ctx| {
            egui::Window::new("In-game").show(ctx, |ui| {
                let (mut rock_count, mut paper_count, mut scissor_count) = (0, 0, 0);
                for player in &self.players {
                    match player.team {
                        Type::Rock => rock_count += 1,
                        Type::Paper => paper_count += 1,
                        Type::Scissor => scissor_count += 1,
                    };
                }
                ui.label(format!("Rock: {rock_count}"));
                ui.label(format!("Paper: {paper_count}"));
                ui.label(format!("Scissor: {scissor_count}"));

                if ui.button("Disconnect").clicked() {
                    new_game_state = Some(GameState::MainMenu(MainMenuState::new()));
                }
            });
        });
        egui_macroquad::draw();

        if let Some(new_game_state) = new_game_state {
            new_game_state
        } else {
            GameState::InGame(self)
        }
    }

    async fn render(&self) {
        for player in &self.players {
            let position = &player.position;
            let icon = match player.team {
                Type::Rock => self.rock,
                Type::Paper => self.paper,
                Type::Scissor => self.scissor,
            };

            centered_text_at(icon, position.x, position.y, 30.0);
        }
    }
}

fn centered_text_at(text: Texture2D, x: f32, y: f32, size: f32) {
    let opts = DrawTextureParams {
        dest_size: Some(Vec2::new(size, size)),
        ..Default::default()
    };
    draw_texture_ex(text, x, y, WHITE, opts);
}
