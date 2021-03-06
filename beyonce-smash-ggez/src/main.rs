extern crate ggez;
extern crate rand;

use rand::Rng;

use ggez::*;
use ggez::event::*;
use ggez::conf;
use ggez::event;
use ggez::{Context, GameResult};
use ggez::graphics;
use ggez::graphics::{Color, DrawMode, Point2};

use std::{env, path};
use std::time::Duration;

const WINDOW_W: u32 = 400;
const WINDOW_H: u32 = 700;

const PLAYER_X: f32 = 195.0;
const PLAYER_Y: f32 = 20.0;
const PLAYER_WALKING_SPEED: f32 = 2.0;
const HITAREA_W: f32 = 128.0;
const HITAREA_H: f32 = 32.0;
const PLAYER_HOLDING_SPEED: f32 = 0.3;
const PLAYER_HOLDING_TIME_MIN: f32 = 4.0;
const PLAYER_HOLDING_TIME_MAX: f32 = 6.0;

const SMASHABLE_X_LEFT: f32 = 135.0;
const SMASHABLE_X_RIGHT: f32 = 255.0;
const SMASHABLE_SPAWN_FACTOR: f32 = 550.0;
const SMASHABLES_PER_SCREEN: u32 = 13;
const SMASHABLE_W: f32 = 64.0;

struct Smashable {
    x: f32,
    y: f32,
    active: bool,
    sprite: graphics::Image,
}

impl Smashable {
    fn new(ctx: &mut Context) -> Smashable {
        let mut rng = rand::thread_rng();
        let y = rng.gen::<f32>() * SMASHABLE_SPAWN_FACTOR + 100.0; // magic number
        let x: f32;
        let ltr = rng.gen();
        match ltr {
            true => x = SMASHABLE_X_LEFT,
            false => x = SMASHABLE_X_RIGHT,
        }
        let sprite = graphics::Image::new(ctx, "/thing.png").unwrap();

        Smashable {
            x: x,
            y: y,
            active: true,
            sprite: sprite,
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.active {
            let point = graphics::Point2::new(self.x, self.y);
            graphics::draw(ctx, &self.sprite, point, 0.0)?;
        }
        Ok(())
    }
}

struct Player {
    x: f32,
    y: f32,
    sprite: graphics::Image,
    hitarea: graphics::Image,
    h_x: f32,
    h_y: f32,
    h_w: f32,
    h_h: f32,
    holding: f32,
}

impl Player {
    fn new(ctx: &mut Context) -> Player {
        Player {
            x: PLAYER_X,
            y: PLAYER_Y,
            sprite: graphics::Image::new(ctx, "/beyonce.png").unwrap(),
            hitarea: graphics::Image::new(ctx, "/hitarea.png").unwrap(),
            h_x: PLAYER_X,
            h_y: PLAYER_X + HITAREA_H,
            h_w: HITAREA_W,
            h_h: HITAREA_H,
            holding: 0.0,
        }
    }

    pub fn update(&mut self) {
        if self.holding == 0.0 {
            self.y = self.y % WINDOW_H as f32 + PLAYER_WALKING_SPEED;
            self.h_y = self.y + HITAREA_H;
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let dest_point = graphics::Point2::new(self.x, self.y);
        graphics::draw(ctx, &self.sprite, dest_point, 0.0)?;
        if self.holding > PLAYER_HOLDING_TIME_MIN {
            let dest_hitarea = graphics::Point2::new(self.h_x, self.h_y);
            graphics::draw(ctx, &self.hitarea, dest_hitarea, 0.0)?;
        }
        Ok(())
    }

    pub fn hold(&mut self) {
        if self.holding > 0.0 {
            self.holding += PLAYER_HOLDING_SPEED;
            if self.holding > PLAYER_HOLDING_TIME_MAX {
                self.unhold();
            }
        } else {
            self.holding = 0.1;
        }
    }

    pub fn unhold(&mut self) {
        self.holding = 0.0;
    }
}

struct MainState {
    player: Player,
    smashables: Vec<Smashable>,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let mut smashables = vec![];

        for _ in 0..SMASHABLES_PER_SCREEN {
            smashables.push(Smashable::new(ctx));
        }

        let s = MainState {
            player: Player::new(ctx),
            smashables: smashables,
        };
        Ok(s)
    }

    pub fn collision(&mut self) {
        if self.player.holding > PLAYER_HOLDING_TIME_MIN {
            for s in self.smashables.iter_mut() {
                if s.active {
                    if self.player.h_x < s.x + SMASHABLE_W
                        && self.player.h_x + self.player.h_w > s.x
                        && self.player.h_y < s.y + SMASHABLE_W
                        && self.player.h_y + self.player.h_h > s.y
                    {
                        s.active = false;
                    }
                }
            }
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        self.player.update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        for s in self.smashables.iter_mut() {
            s.draw(ctx)?;
        }
        self.player.draw(ctx)?;

        graphics::present(ctx);
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut ggez::Context, keycode: Keycode, _: Mod, _: bool) {
        match keycode {
            Keycode::Space => {
                self.player.hold();
            }
            _ => {}
        }
    }

    fn key_up_event(&mut self, ctx: &mut ggez::Context, keycode: Keycode, _: Mod, _: bool) {
        match keycode {
            Keycode::Space => {
                self.collision();
                self.player.unhold();
            }
            _ => {}
        }
    }
}

pub fn main() {
    let mut cb = ContextBuilder::new("beyonce", "ggez")
        .window_setup(conf::WindowSetup::default().title("beyonce smash"))
        .window_mode(conf::WindowMode::default().dimensions(WINDOW_W, WINDOW_H));

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources/");
        cb = cb.add_resource_path(path);
    } else {
        println!("Not building from cargo?  Ok.");
    }

    let ctx = &mut cb.build().unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    event::run(ctx, state).unwrap();
}
