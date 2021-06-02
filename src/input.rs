use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

pub enum JoypadButton {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Select,
    Start,
}

pub enum Input {
    Joypad(JoypadButton),
}

impl Input {
    pub fn new(event_pump: &mut EventPump) -> Option<Self> {
        match event_pump.poll_event() {
            Some(Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            }) => Some(Self::Joypad(JoypadButton::Up)),
            Some(Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            }) => Some(Self::Joypad(JoypadButton::Down)),
            Some(Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            }) => Some(Self::Joypad(JoypadButton::Left)),
            Some(Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            }) => Some(Self::Joypad(JoypadButton::Right)),
            Some(Event::KeyDown {
                keycode: Some(Keycode::Return),
                ..
            }) => Some(Self::Joypad(JoypadButton::Start)),
            _ => None,
        }
    }
}
