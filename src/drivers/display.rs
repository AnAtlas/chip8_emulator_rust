use sdl2;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels;
use sdl2::rect::Rect;

use crate::{DISPLAY_WIDTH, DISPLAY_HEIGHT, VRAM};

pub struct Display {
    displayCanvas: Canvas<Window>,
    keypadCanvas: Canvas<Window>,
    scale: usize,
}

pub enum WindowType {
    Display,
    Keypad,
}

impl Display {
    pub fn from(sdl_context: &sdl2::Sdl, scale: usize) -> Self {
        let video_subsystem = sdl_context.video().unwrap();
        let display_window = video_subsystem.window("Chip 8 Emulator", (DISPLAY_WIDTH * scale) as u32, (DISPLAY_HEIGHT * scale)as u32).position_centered().opengl().build().unwrap();
        let mut display_canvas = display_window.into_canvas().build().unwrap();
        display_canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        display_canvas.clear();
        display_canvas.present();

        let keypad_window = video_subsystem.window("Keypad", 32 * 5, 32 * 5).position(0, 50).opengl().build().unwrap();
        let mut keypad_canvas = keypad_window.into_canvas().build().unwrap();
        keypad_canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        keypad_canvas.clear();
        keypad_canvas.set_draw_color(pixels::Color::RGB(0, 255, 0));
        let _ = keypad_canvas.draw_rect(Rect::new(4, 4, 10, 10));
        keypad_canvas.present();

        Display{ displayCanvas: display_canvas, keypadCanvas: keypad_canvas, scale}
    }

    pub fn draw(&mut self, pixels: &VRAM) {
        for (y, row) in pixels.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                let x = x * self.scale;
                let y = y * self.scale;

                self.displayCanvas.set_draw_color(color(col));
                let _ = self.displayCanvas.fill_rect(Rect::new(x as i32, y as i32, self.scale as u32, self.scale as u32));
            }
        }
        self.displayCanvas.present();
    }

    pub fn get_window_id(&self, window_type: WindowType) -> u32 {
        match window_type {
            WindowType::Display => self.displayCanvas.window().id(),
            WindowType::Keypad => self.keypadCanvas.window().id(),
        }
    }
}

fn color(value: u8) -> pixels::Color {
    if value == 0 {pixels::Color::RGB(0, 0, 0)}
    else {pixels::Color::RGB(0, 250, 0)}
}