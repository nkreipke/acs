use std::process::exit;
use anyhow::{anyhow, Context, Result};
use minifb::{Key, Scale, Window, WindowOptions};

const SIZE_MULTIPLICAND_ENV: &str = "ACS_WINDOW_SIZE_MUL";

pub struct AssistantWindow {
    window: Window,
    width: u32,
    height: u32
}

impl AssistantWindow {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let size_multiplicand = std::env::var(SIZE_MULTIPLICAND_ENV).ok().and_then(|x| x.parse::<u32>().ok()).unwrap_or(1);
        let scale = match size_multiplicand {
            1 => Scale::X1,
            2 => Scale::X2,
            4 => Scale::X4,
            _ => return Err(anyhow!("Unsupported size multiplicand"))
        };

        let mut window = Window::new("Assistant",
            (width * size_multiplicand) as usize,
            (height * size_multiplicand) as usize,
            WindowOptions {
                borderless: true,
                resize: false,
                scale,
                topmost: true,
                transparency: true,
                //none: true,
                .. Default::default()
            }).context("cannot initialize window")?;

        // We do our own rate limiting
        window.limit_update_rate(None);

        Ok(AssistantWindow {
            window,
            width,
            height
        })
    }

    pub fn draw(&mut self, data: &[u32]) -> Result<()> {
        self.window.update_with_buffer(data, self.width as usize, self.height as usize)?;

        Ok(())
    }

    pub fn poll_events(&mut self) {
        if self.window.is_key_down(Key::Escape) {
            exit(0);
        }
    }
}