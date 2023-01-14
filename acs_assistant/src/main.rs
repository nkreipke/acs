#![windows_subsystem="windows"] // hide the console window under Windows

mod sdl_anyhow_interop;
mod window;

use std::collections::{HashMap, VecDeque};
use std::collections::hash_map::Entry;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use anyhow::{anyhow, Context, Result};
use acs::AcsFile;
use crate::window::AssistantWindow;

const ANIM_PLAYLIST: &[&str] = &["SHOW"]; // default order of animations
const ANIM_IDLE: &str = "HIDE"; // what to use if our queue is empty

const POLL_INTERVAL: Duration = Duration::from_millis(300);

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();

    let acs_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        // Find the single acs file in our working directory
        let path = std::fs::read_dir(".")
            .context("cannot enumerate directory")?
            .filter_map(|result| result.ok())
            .map(|entry| entry.path())
            .find(|path| path.extension() == Some(OsStr::new("acs")));

        match path {
            Some(path) => path,
            None => return Err(anyhow!("No ACS file specified and none found in the working directory"))
        }
    };

    // Parse the ACS file
    let acs = AcsFile::open_path(acs_path)?;
    let (width, height) = acs.char_size();

    let mut window = AssistantWindow::new(width as u32, height as u32)?;

    let mut image_cache = HashMap::new();
    let mut animations = HashMap::new();

    let mut animation_queue = ANIM_PLAYLIST.iter().map(|s| s.to_string()).collect::<VecDeque<_>>();
    let mut frame_time = Instant::now();

    // Read all animations
    for animation in acs.animations() {
        let animation = animation?;
        animation_queue.push_back(animation.name().to_string());

        animations.insert(animation.name().to_string(), animation);
    }

    loop {
        let current_animation = animations.get(animation_queue.pop_front().as_deref().unwrap_or(ANIM_IDLE)).ok_or(anyhow!("missing animation"))?;

        for frame in current_animation.frames()? {
            if let Some(frame_image) = frame.images()?.next() {
                let image_index = frame_image.image_index();

                // Lazy load (decompress) our image
                let image = match image_cache.entry(image_index) {
                    Entry::Occupied(entry) => entry.into_mut(),
                    Entry::Vacant(entry) => {
                        let image = acs.image(image_index)?;

                        let mut data = vec![];
                        image.read_argb(&mut data);

                        entry.insert(data.into_boxed_slice())
                    }
                };

                window.draw(image)?;
            }

            frame_time += frame.duration();

            // Wait for the next frame
            loop {
                window.poll_events();

                let now = Instant::now();
                if now >= frame_time {
                    break;
                }

                let wait = (frame_time - now).min(POLL_INTERVAL);
                std::thread::sleep(wait);
            }
        }
    }
}
