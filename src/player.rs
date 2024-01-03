use crate::{frame::Drawable, invaders::Invaders, shot::Shot, NUM_COLS, NUM_ROWS};
use std::time::Duration;

pub struct Player {
    x: usize,
    y: usize,
    shots: Vec<Shot>,
    game_level: usize,
}

impl Player {
    pub fn new(level: usize) -> Self {
        Self {
            x: (NUM_COLS * level) / 2,
            y: (NUM_ROWS * level) - 1,
            shots: Vec::new(),
            game_level: level,
        }
    }

    pub fn move_left(&mut self) {
        if self.x > 0 {
            self.x -= 1;
        }
    }

    pub fn move_right(&mut self) {
        let num_cols = NUM_COLS * self.game_level;
        if self.x < num_cols - 1 {
            self.x += 1;
        }
    }

    pub fn shoot(&mut self) -> bool {
        if self.shots.len() < 2 {
            self.shots.push(Shot::new(self.x, self.y - 1));
            true
        } else {
            false
        }
    }

    pub fn update(&mut self, delta: Duration) {
        for shot in self.shots.iter_mut() {
            shot.update(delta);
        }
        self.shots.retain(|shot| !shot.dead());
    }

    pub fn detect_hits(&mut self, invaders: &mut Invaders) -> bool {
        let mut hit_something = false;
        for shot in self.shots.iter_mut() {
            if (!shot.exploding) && (invaders.kill_invader_at(shot.x, shot.y)) {
                hit_something = true;
                shot.explode();
            }
        }
        hit_something
    }
}

impl Drawable for Player {
    fn draw(&self, frame: &mut crate::frame::Frame) {
        frame[self.x][self.y] = "A";
        for shot in self.shots.iter() {
            shot.draw(frame);
        }
    }
}
