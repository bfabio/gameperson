use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub enum JoypadButton {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Select,
    Start
}

pub enum Input {
    Joypad(JoypadButton),
}

impl Input {
    pub fn new(event_pump: &mut EventPump) -> Option<Self> {
        match event_pump.poll_event() {
            Some(Event::KeyDown { keycode: Some(Keycode::Up), .. }) => {
                Some(Input::Joypad(JoypadButton::Up))
            }
            Some(Event::KeyDown { keycode: Some(Keycode::Down), .. }) => {
                Some(Input::Joypad(JoypadButton::Down))
            }
            _ => { None }
        }
    }
}
