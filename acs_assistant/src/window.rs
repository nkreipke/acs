use std::process::exit;
use anyhow::{Context, Result};
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, TextureCreator, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::video::{WindowContext, WindowSetShapeError, WindowShapeMode};
use crate::sdl_anyhow_interop::CompatErrorResultTypes;

const SIZE_MULTIPLICAND_ENV: &str = "ACS_WINDOW_SIZE_MUL";

pub struct AssistantWindow {
    enable_window_shaping: bool,
    canvas: WindowCanvas,
    surface: Surface<'static>,
    #[allow(dead_code)] // required for the unsafe_textures feature to retain our texture
    texture_creator: TextureCreator<WindowContext>,
    texture: Texture,
    event_pump: EventPump,
    width: u32,
    height: u32,
    size_multiplicand: u32
}

impl AssistantWindow {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let size_multiplicand = std::env::var(SIZE_MULTIPLICAND_ENV).ok().and_then(|x| x.parse::<u32>().ok()).unwrap_or(1);

        let sdl_context = sdl2::init().compat_err().context("cannot init sdl")?;
        let video_subsystem = sdl_context.video().compat_err()?;
        let wdw = video_subsystem
            .window("Assistant", width * size_multiplicand, height * size_multiplicand)
            .shaped()
            .position_centered()
            .allow_highdpi()
            .build()?;

        unsafe { sdl2::sys::SDL_SetWindowHitTest(wdw.raw(), Some(sdl_window_hit_test_move), std::ptr::null::<sdl2::libc::c_void>() as *mut _); }

        let canvas = wdw.into_canvas().build()?;

        let surface = Surface::new(width, height, PixelFormatEnum::RGBA32).compat_err()?;

        let texture_creator = canvas.texture_creator();

        let texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGBA32, width, height)?;

        Ok(AssistantWindow {
            enable_window_shaping: true,
            canvas,
            surface,
            texture_creator,
            texture,
            event_pump: sdl_context.event_pump().compat_err()?,
            width,
            height,
            size_multiplicand
        })
    }

    pub fn draw(&mut self, data: &[u8]) -> Result<()> {
        self.surface.with_lock_mut(|s| s.copy_from_slice(data));
        self.texture.update(None, data, self.surface.pitch() as usize)?;

        self.canvas.copy(
            &self.texture,
            Rect::new(0, 0, self.width, self.height),
            Rect::new(0,0,self.width * self.size_multiplicand,self.height * self.size_multiplicand)).compat_err()?;
        self.canvas.present();

        if self.enable_window_shaping {
            match self.canvas.window_mut().set_shape(&self.surface, WindowShapeMode::Default) {
                Ok(()) => {},
                Err(WindowSetShapeError::NonShapeableWindow) => {
                    eprintln!("warning: window shaping is unusable on this platform");
                    self.enable_window_shaping = false;
                },
                other => other?
            }
        }

        Ok(())
    }

    pub fn poll_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            //println!("{:?}", event);

            match event {
                Event::Quit { .. } => exit(0),
                _ => {}
                /*Event::AppTerminating { .. } => {}
                Event::AppLowMemory { .. } => {}
                Event::AppWillEnterBackground { .. } => {}
                Event::AppDidEnterBackground { .. } => {}
                Event::AppWillEnterForeground { .. } => {}
                Event::AppDidEnterForeground { .. } => {}
                Event::Display { .. } => {}
                Event::Window { .. } => {}
                Event::KeyDown { .. } => {}
                Event::KeyUp { .. } => {}
                Event::TextEditing { .. } => {}
                Event::TextInput { .. } => {}
                Event::MouseMotion { .. } => {}
                Event::MouseButtonDown { .. } => {}
                Event::MouseButtonUp { .. } => {}
                Event::MouseWheel { .. } => {}
                Event::JoyAxisMotion { .. } => {}
                Event::JoyBallMotion { .. } => {}
                Event::JoyHatMotion { .. } => {}
                Event::JoyButtonDown { .. } => {}
                Event::JoyButtonUp { .. } => {}
                Event::JoyDeviceAdded { .. } => {}
                Event::JoyDeviceRemoved { .. } => {}
                Event::ControllerAxisMotion { .. } => {}
                Event::ControllerButtonDown { .. } => {}
                Event::ControllerButtonUp { .. } => {}
                Event::ControllerDeviceAdded { .. } => {}
                Event::ControllerDeviceRemoved { .. } => {}
                Event::ControllerDeviceRemapped { .. } => {}
                Event::ControllerTouchpadDown { .. } => {}
                Event::ControllerTouchpadMotion { .. } => {}
                Event::ControllerTouchpadUp { .. } => {}
                Event::ControllerSensorUpdated { .. } => {}
                Event::FingerDown { .. } => {}
                Event::FingerUp { .. } => {}
                Event::FingerMotion { .. } => {}
                Event::DollarGesture { .. } => {}
                Event::DollarRecord { .. } => {}
                Event::MultiGesture { .. } => {}
                Event::ClipboardUpdate { .. } => {}
                Event::DropFile { .. } => {}
                Event::DropText { .. } => {}
                Event::DropBegin { .. } => {}
                Event::DropComplete { .. } => {}
                Event::AudioDeviceAdded { .. } => {}
                Event::AudioDeviceRemoved { .. } => {}
                Event::RenderTargetsReset { .. } => {}
                Event::RenderDeviceReset { .. } => {}
                Event::User { .. } => {}
                Event::Unknown { .. } => {}*/
            }
        }
    }
}

extern "C" fn sdl_window_hit_test_move(
    _win: *mut sdl2::sys::SDL_Window,
    _area: *const sdl2::sys::SDL_Point,
    _data: *mut sdl2::libc::c_void,
) -> sdl2::sys::SDL_HitTestResult {
    // Make the whole window draggable
    sdl2::sys::SDL_HitTestResult::SDL_HITTEST_DRAGGABLE
}