extern crate rand;
extern crate sdl2;

mod cpu;
mod drivers;

use std::thread;
use std::time::Duration;
use cpu::CPU;
use std::env;

use drivers::{Display, Input, ROM, WindowType, WindowAction, Audio};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const MEMORY_SIZE: usize = 4096;

type VRAM = [[u8; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

fn main() {
    let sleep_duration = Duration::from_millis(2);

    let sdl_context = sdl2::init().unwrap();

    // let args: Vec<String> = env::args().collect();
    let rom_path = "tetris.rom";//&args[1];
    let scale = 5; //&args[2];

    let mut display = Display::from(&sdl_context, scale);
    let audio = Audio::new(&sdl_context);
    let mut input = Input::from(&sdl_context, display.get_window_id(WindowType::Keypad), display.get_window_id(WindowType::Display));
    let rom = ROM::from(rom_path);
    let mut cpu = CPU::new();

    cpu.load(&rom.rom);

    loop {
        if let Some(window_action) = input.poll_window_events() {
            if window_action == WindowAction::Close {
                break;
            }
        }
        let keypad = input.poll();
        let output = cpu.tick(&keypad);
        if output.vram_changed {
            display.draw(output.vram);
        }

        if output.beep {audio.start_beep()} else {audio.stop_beep()}

        thread::sleep(sleep_duration);
    }
}
