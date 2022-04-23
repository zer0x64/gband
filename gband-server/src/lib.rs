use gband::JoypadState;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum Button {
    Start,
    Select,
    B,
    A,
    Down,
    Up,
    Left,
    Right,
}

impl From<Button> for JoypadState {
    fn from(button: Button) -> Self {
        match button {
            Button::Start => JoypadState::START,
            Button::Select => JoypadState::SELECT,
            Button::B => JoypadState::B,
            Button::A => JoypadState::A,
            Button::Down => JoypadState::DOWN,
            Button::Up => JoypadState::UP,
            Button::Left => JoypadState::LEFT,
            Button::Right => JoypadState::RIGHT,
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug)]
pub enum EventType {
    Pressed,
    Released,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Event {
    pub frame: usize,
    pub ty: EventType,
    pub buttons: Vec<Button>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InitialInputs(pub Vec<Event>);

impl InitialInputs {
    pub fn parse_str(ron_repr: &str) -> anyhow::Result<Self> {
        let mut inputs: Self = ron::from_str(ron_repr)?;
        inputs.0.sort_by(|a, b| a.frame.cmp(&b.frame));
        Ok(inputs)
    }
}
