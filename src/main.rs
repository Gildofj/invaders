use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use invaders::{
    frame::{self, new_frame, Drawable, Frame},
    invaders::Invaders,
    player::Player,
    render,
};
use rusty_audio::Audio;
use std::error::Error;
use std::io;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

struct Level {
    number: usize,
}

struct RenderHandleResult {
    frame: Frame,
    is_next_level: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let level = Arc::new(Mutex::new(Level { number: 1 }));

    let mut audio = Audio::new();
    audio.add("explode", "explode.wav");
    audio.add("lose", "lose.wav");
    audio.add("move", "move.wav");
    audio.add("pew", "pew.wav");
    audio.add("startup", "startup.wav");
    audio.add("win", "win.wav");
    audio.play("startup");

    //Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame(1);
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let result: RenderHandleResult = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            if result.is_next_level {
                render::render(&mut stdout, &result.frame, &result.frame, true);
            } else {
                render::render(&mut stdout, &last_frame, &result.frame, false);
            }
            last_frame = result.frame;
        }
    });

    //Game Loop
    let mut player = Player::new(1);
    let mut instant = Instant::now();
    let mut invaders = Invaders::new(1);
    'gameloop: loop {
        // Per frame init
        let mut curr_frame = new_frame(1);
        let delta = instant.elapsed();
        instant = Instant::now();

        if let Ok(data) = level.lock() {
            curr_frame = new_frame(data.number)
        };

        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew")
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }

        // Updates
        if let Ok(data) = level.lock() {
            curr_frame = new_frame(data.number);
            player.update(delta);
            if invaders.update(delta) {
                audio.play("move");
            }
            if player.detect_hits(&mut invaders) {
                audio.play("explode");
            }
        }

        // Draw & render
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        let _ = render_tx.send(RenderHandleResult {
            frame: curr_frame,
            is_next_level: false,
        });
        thread::sleep(Duration::from_millis(1));

        // Win or lose?
        if let Ok(mut data) = level.lock() {
            if invaders.all_killed() {
                audio.play("win");
                data.number += 1;
                player = Player::new(data.number);
                instant = Instant::now();
                invaders = Invaders::new(data.number);
                let _ = render_tx.send(RenderHandleResult {
                    frame: new_frame(data.number),
                    is_next_level: true,
                });
            }
        } else {
            break 'gameloop;
        }

        if invaders.reached_bottom() {
            audio.play("lose");
            break 'gameloop;
        }
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
