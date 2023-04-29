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
    JoypadPress(JoypadButton),
    JoypadRelease(JoypadButton),
}

impl Input {
    pub fn new(event_pump: &mut EventPump) -> Option<Self> {
        match event_pump.poll_event() {
            Some(Event::KeyDown {
                keycode: Some(keycode),
                ..
            }) => match keycode {
                Keycode::Up => Some(Self::JoypadPress(JoypadButton::Up)),
                Keycode::Down => Some(Self::JoypadPress(JoypadButton::Down)),
                Keycode::Left => Some(Self::JoypadPress(JoypadButton::Left)),
                Keycode::Right => Some(Self::JoypadPress(JoypadButton::Right)),
                Keycode::LCtrl => Some(Self::JoypadPress(JoypadButton::A)),
                Keycode::LAlt => Some(Self::JoypadPress(JoypadButton::B)),
                Keycode::Return => Some(Self::JoypadPress(JoypadButton::Start)),
                Keycode::RShift => Some(Self::JoypadPress(JoypadButton::Select)),
                _ => None,
            },

            Some(Event::KeyUp{
                keycode: Some(keycode),
                ..
            }) => match keycode {
                Keycode::Up => Some(Self::JoypadRelease(JoypadButton::Up)),
                Keycode::Down => Some(Self::JoypadRelease(JoypadButton::Down)),
                Keycode::Left => Some(Self::JoypadRelease(JoypadButton::Left)),
                Keycode::Right => Some(Self::JoypadRelease(JoypadButton::Right)),
                Keycode::LCtrl => Some(Self::JoypadRelease(JoypadButton::A)),
                Keycode::LAlt => Some(Self::JoypadRelease(JoypadButton::B)),
                Keycode::Return => Some(Self::JoypadRelease(JoypadButton::Start)),
                Keycode::RShift => Some(Self::JoypadRelease(JoypadButton::Select)),
                _ => None,
            }

            _ => None,
        }
    }
}
