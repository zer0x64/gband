use gband::JoypadState;
use gloo::timers::callback::Interval;
use yew::prelude::*;

pub enum EmulatorMessage {
    Tick,
    KeyUp(web_sys::KeyboardEvent),
    KeyDown(web_sys::KeyboardEvent),
}

#[derive(PartialEq, Properties)]
pub struct EmulatorProps {
    pub rom: Vec<u8>,
}

pub struct Emulator {
    emu: gband::Emulator,
    canvas: NodeRef,
    joypad: JoypadState,

    #[cfg(feature = "gamepad")]
    gamepad_events: Option<gilrs::Gilrs>,

    _interval: Interval,
}

impl Component for Emulator {
    type Message = EmulatorMessage;
    type Properties = EmulatorProps;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let emu = gband::Emulator::new(&props.rom, None).unwrap();

        let interval = {
            let link = ctx.link().clone();
            Interval::new(1000 / 60, move || link.send_message(EmulatorMessage::Tick))
        };

        #[cfg(feature = "gamepad")]
        let gamepad_events = match gilrs::Gilrs::new() {
            Ok(g) => Some(g),
            Err(_e) => None,
        };

        Self {
            emu,
            canvas: NodeRef::default(),
            joypad: JoypadState::default(),

            #[cfg(feature = "gamepad")]
            gamepad_events,

            _interval: interval,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onkeydown = ctx.link().callback(EmulatorMessage::KeyDown);
        let onkeyup = ctx.link().callback(EmulatorMessage::KeyUp);

        html! {
            <div {onkeydown} {onkeyup} tabIndex="0">
                <canvas ref={self.canvas.clone()} width="160" height="144" style="width:100%;
                    image-rendering: pixelated;
                    image-rendering: crisp-edges;
                    image-rendering: -moz-crisp-edges;
                    image-rendering: -o-crisp-edges;
                    image-rendering: -webkit-crisp-edges;
                    -ms-interpolation-mode: nearest-neighbor;"
                ></canvas>
            </div>
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EmulatorMessage::Tick => {
                self.tick();
                true
            }
            EmulatorMessage::KeyUp(e) => {
                if let Some(input) = h_key_event_to_joypad(e) {
                    self.joypad.remove(input);
                    self.emu.set_joypad(self.joypad);
                }
                false
            }
            EmulatorMessage::KeyDown(e) => {
                if let Some(input) = h_key_event_to_joypad(e) {
                    self.joypad.insert(input);
                    self.emu.set_joypad(self.joypad);
                }
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let props = ctx.props();
        self.emu = gband::Emulator::new(&props.rom, None).unwrap();
        false
    }
}

impl Emulator {
    fn tick(&mut self) {
        use wasm_bindgen::{Clamped, JsCast};
        use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

        #[cfg(feature = "gamepad")]
        if let Some(gilrs) = &mut self.gamepad_events {
            if let Some(gilrs::Event {
                id: _id,
                event,
                time: _time,
            }) = gilrs.next_event()
            {
                match gilrs_to_gband_input(event) {
                    Some(JoypadStateChange::Pressed(input)) => {
                        self.joypad.insert(input);
                        self.emu.set_joypad(self.joypad);
                    }
                    Some(JoypadStateChange::Released(input)) => {
                        self.joypad.remove(input);
                        self.emu.set_joypad(self.joypad);
                    }
                    None => {}
                }
            }
        }

        let frame = loop {
            if let Some(frame) = self.emu.clock() {
                break frame;
            }
        };

        // Get canvas 2d context
        let context = self
            .canvas
            .cast::<HtmlCanvasElement>()
            .unwrap()
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        // Draw image data to the canvas
        let image_data =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(frame.as_ref()), 160, 144).unwrap();

        context.put_image_data(&image_data, 0.0, 0.0).unwrap();
    }
}

fn h_key_event_to_joypad(e: KeyboardEvent) -> Option<JoypadState> {
    match e.key_code() {
        0x58 => Some(JoypadState::A),
        0x5a => Some(JoypadState::B),
        0x41 => Some(JoypadState::SELECT),
        0x53 => Some(JoypadState::START),
        0x28 => Some(JoypadState::DOWN),
        0x25 => Some(JoypadState::LEFT),
        0x27 => Some(JoypadState::RIGHT),
        0x26 => Some(JoypadState::UP),
        _ => None,
    }
}

#[cfg(feature = "gamepad")]
enum JoypadStateChange {
    Pressed(JoypadState),
    Released(JoypadState),
}

#[cfg(feature = "gamepad")]
fn gilrs_to_gband_input(event: gilrs::EventType) -> Option<JoypadStateChange> {
    match event {
        gilrs::EventType::AxisChanged(axis, value, _) => match axis {
            gilrs::Axis::LeftStickX | gilrs::Axis::DPadX => {
                if value > 0.0 {
                    if value > 0.4 {
                        Some(JoypadStateChange::Pressed(JoypadState::RIGHT))
                    } else {
                        Some(JoypadStateChange::Released(JoypadState::RIGHT))
                    }
                } else if value < -0.4 {
                    Some(JoypadStateChange::Pressed(JoypadState::LEFT))
                } else {
                    Some(JoypadStateChange::Released(JoypadState::LEFT))
                }
            }
            gilrs::Axis::LeftStickY | gilrs::Axis::DPadY => {
                if value > 0.0 {
                    if value > 0.4 {
                        Some(JoypadStateChange::Pressed(JoypadState::UP))
                    } else {
                        Some(JoypadStateChange::Released(JoypadState::UP))
                    }
                } else if value < -0.4 {
                    Some(JoypadStateChange::Pressed(JoypadState::DOWN))
                } else {
                    Some(JoypadStateChange::Released(JoypadState::DOWN))
                }
            }
            _ => None,
        },
        gilrs::EventType::ButtonPressed(b, _) => {
            gilrs_button_to_gband_input(b).map(JoypadStateChange::Pressed)
        }
        gilrs::EventType::ButtonReleased(b, _) => {
            gilrs_button_to_gband_input(b).map(JoypadStateChange::Released)
        }
        _ => None,
    }
}

#[cfg(feature = "gamepad")]
fn gilrs_button_to_gband_input(keycode: gilrs::Button) -> Option<JoypadState> {
    match keycode {
        gilrs::Button::East => Some(JoypadState::A),
        gilrs::Button::South => Some(JoypadState::B),
        gilrs::Button::Start => Some(JoypadState::START),
        gilrs::Button::Select => Some(JoypadState::SELECT),
        gilrs::Button::DPadDown => Some(JoypadState::DOWN),
        gilrs::Button::DPadLeft => Some(JoypadState::LEFT),
        gilrs::Button::DPadRight => Some(JoypadState::RIGHT),
        gilrs::Button::DPadUp => Some(JoypadState::UP),
        _ => None,
    }
}
