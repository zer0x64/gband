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

        html! {
            <>
            <div class="container">
                <header class="py-3">
                    <div class="row flex-nowrap justify-content-between align-items-center">
                        <div class="col-4 text-center">
                            <a class="header-logo text-dark" href="#">{ "Gband emulator" }</a>
                        </div>
                    </div>
                </header>

                <header class="py-3">
                    <div class="row flex-nowrap justify-content-between align-items-center">
                        <div class="col-4 text-center">
                            <a class="logo text-dark" href="#livedemo">{ "Live demo" }</a>
                        </div>
                    </div>
                </header>
            </div>

            <main class="container">
                <h1 id="introduction">{ "Introduction" }</h1>

                <p>{ "Welcome to the GBand homepage!" }</p>

                <p>{ "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum." }</p>

                <h1 id="livedemo">{ "Live demo" }</h1>

                <div class="row">
                    <div class="col-8">
                    {
                        if let Some(rom) = self.rom.clone() {
                            html! { <Emulator {rom} /> }
                        } else {
                            html! { <p>{ "Choose a ROM and try directly in your browser." }</p> }
                        }
                    }
                    </div>

                    <div class="col">
                        <p>{ "ROM to load: "} <input type="file" {onchange} /></p>

                        <h3>{ "Controls" }</h3>

                        <table class="table table-hover">
                            <thead>
                                <tr>
                                    <th scope="col">{ "Key" }</th>
                                    <th scope="col">{ "Joypad" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <th><kbd>{ "←" }</kbd></th>
                                    <th><kbd>{ "←" }</kbd></th>
                                </tr>
                                <tr>
                                    <th><kbd>{ "→" }</kbd></th>
                                    <th><kbd>{ "→" }</kbd></th>
                                </tr>
                                <tr>
                                    <th><kbd>{ "↑" }</kbd></th>
                                    <th><kbd>{ "↑" }</kbd></th>
                                </tr>
                                <tr>
                                    <th><kbd>{ "↓" }</kbd></th>
                                    <th><kbd>{ "↓" }</kbd></th>
                                </tr>
                                <tr>
                                    <th><kbd>{ "X" }</kbd></th>
                                    <th><kbd>{ "A" }</kbd></th>
                                </tr>
                                <tr>
                                    <th><kbd>{ "Z" }</kbd></th>
                                    <th><kbd>{ "B" }</kbd></th>
                                </tr>
                                <tr>
                                    <th><kbd>{ "S" }</kbd></th>
                                    <th><kbd>{ "START" }</kbd></th>
                                </tr>
                                <tr>
                                    <th><kbd>{ "A" }</kbd></th>
                                    <th><kbd>{ "SELECT" }</kbd></th>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </main>
            </>
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