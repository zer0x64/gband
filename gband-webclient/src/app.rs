use crate::emulator::Emulator;
use gloo::file::callbacks::FileReader;
use gloo::file::File;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

pub enum AppMessage {
    RomFile(File),
    RomBytes(Vec<u8>),
}

pub struct App {
    reader: Option<FileReader>,
    rom: Option<Vec<u8>>,
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self {
            reader: None,
            rom: None,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onchange = ctx.link().callback(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let file = File::from(input.files().unwrap().get(0).unwrap());
            AppMessage::RomFile(file)
        });

        if let Some(rom) = self.rom.clone() {
            html! {
                <div>
                    <Emulator {rom} />
                    <input type="file" {onchange} />
                </div>
            }
        } else {
            html! {
                <div>
                    <input type="file" {onchange} />
                </div>
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::RomFile(file) => {
                let link = ctx.link().clone();
                let reader = gloo::file::callbacks::read_as_bytes(&file, move |res| {
                    link.send_message(AppMessage::RomBytes(res.expect("failed to read file")))
                });
                self.reader = Some(reader);
                false
            }
            AppMessage::RomBytes(rom) => {
                self.rom = Some(rom);
                true
            }
        }
    }
}
