use sdl2;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::Sdl;

pub struct Input {
    events: sdl2::EventPump,
    keypad_window_id: u32,
    display_window_id: u32,
}

#[derive(PartialEq)]
pub enum WindowAction{
    Close,
}

impl Input {
    pub fn from(sdl_context: &sdl2::Sdl, keypad_window_id: u32, display_window_id: u32) -> Self {
        Input{events:sdl_context.event_pump().unwrap(), keypad_window_id, display_window_id}
    }

    pub fn poll_window_events(&mut self) -> Option<WindowAction> {
        for event in self.events.poll_iter() {
            return match event {
                Event::Window { timestamp: _timestamp, window_id, win_event} => self.handle_window_event(window_id, win_event),
                Event::Quit { .. } => { Some(WindowAction::Close) }
                _ => { None }
            }
        }
        None
    }

    fn handle_window_event(&self, window_id: u32, win_event: WindowEvent) -> Option<WindowAction> {
        if win_event == WindowEvent::Close {
            return Some(WindowAction::Close);
        }
        None
    }

    pub fn poll(&mut self) -> [bool; 16] {
        let keys: Vec<Keycode> = self.events.keyboard_state().pressed_scancodes().filter_map(Keycode::from_scancode).collect();

        let mut chip8_keys = [false; 16];

        for key in keys {
            let index = match key {
                Keycode::Num1 => Some(0x1),
                Keycode::Num2 => Some(0x2),
                Keycode::Num3 => Some(0x3),
                Keycode::Num4 => Some(0x4),
                Keycode::Num5 => Some(0x5),
                Keycode::Num6 => Some(0x6),
                Keycode::Num7 => Some(0x7),
                Keycode::Num8 => Some(0x8),
                Keycode::Num9 => Some(0x9),
                Keycode::Num0 => Some(0x0),
                Keycode::A => Some(0xA),
                Keycode::B => Some(0xB),
                Keycode::C => Some(0xC),
                Keycode::D => Some(0xD),
                Keycode::E => Some(0xE),
                Keycode::F => Some(0xF),
                _ => None,
            };

            if let Some(i) = index {
                chip8_keys[i] = true;
            }
        }
        chip8_keys
    }
}