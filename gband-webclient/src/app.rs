use crate::emulator::Emulator;
use gloo::file::callbacks::FileReader;
use gloo::file::File;
use yew::prelude::*;

const DESCRIPTION: &str =  "You can be one of the first to experience the thrill our the new experience we crafted for you.
                            You can play in your browser below, or use one of the download links to download it your PC.
                            You can also try out the demo for the lauch title: Super Myco Boi™";

pub enum AppMessage {
    RomFile(File),
    RomSample(String),
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
        use web_sys::{HtmlInputElement, HtmlSelectElement};

        let file_onchange = ctx.link().callback(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let file = File::from(input.files().unwrap().get(0).unwrap());
            AppMessage::RomFile(file)
        });

        let sample_onchange = ctx.link().callback(|e: Event| {
            let input: HtmlSelectElement = e.target_unchecked_into();
            let sample_name = input.value();
            AppMessage::RomSample(sample_name)
        });

        html! {
            <body>
            <div class="container">
                <header class="py-3">
                    <div class="row flex-nowrap justify-content-between align-items-center">
                        <div class="col-4 text-center">
                            <h1 class="logo text-dark">{ "GBAND" }</h1>
                        </div>
                    </div>
                </header>
            </div>

            <main class="container">
                <h1 id="introduction">{ "Introduction" }</h1>

                <p>{ "GBAND, the new revolution in multplayer gaming. " }</p>

                <h4>{ "A Mycoverse Exclusive™" }</h4>

                <p>{ DESCRIPTION }</p>

                <h4>{ "Play as Myco Boi with your friends!" }</h4>

                <p>{ "When launching GBAND, you can connect to the game servers using this command:" }</p>
                <code>{ "./gband -c \"http://gband.ctf:8080\" path/to/rom" }</code>
                <p>{ "You can then adventure through the magical mushroom forest and talk to the elder mushroom mans to connect with other players." }</p>
                <p>{ "Note that for now, the multiplayer servers are only able to handle Super Myco Boi™ using the native version of GBAND. It won't work using the web version." }</p>

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
                        <table class="table">
                            <thead>
                                <tr>
                                    <th scope="col"></th>
                                    <th scope="col">{ "Select a ROM" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr>
                                    <th>{ "Your own" }</th>
                                    <th><input type="file" onchange={file_onchange} /></th>
                                </tr>
                                <tr>
                                    <th>{ "Sample" }</th>
                                    <th>
                                        <select name="samples" id="samples" onchange={sample_onchange}>
                                            <option value="none" selected=true>{ "Choose from list" }</option>
                                            <option value="super-myco-boi.gbc">{ "super-myco-boi.gbc" }</option>
                                            <option value="desertboy.gb">{ "desertboy.gb" }</option>
                                            <option value="flappyboy.gb">{ "flappyboy.gb" }</option>
                                            <option value="ucity.gbc">{ "ucity.gbc" }</option>
                                            <option value="RenegadeRush.gb">{ "RenegadeRush.gb" }</option>
                                        </select>
                                    </th>
                                </tr>
                            </tbody>
                        </table>

                        <h3>{ "Controls" }</h3>
                        <h4>{ "You can also use a controller! (native version only)" }</h4>
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

                <h1 id="livedemo">{ "Downloads" }</h1>

                <div class="row">
                    <div class="col">
                        <h3>{ "GBAND Emulator" }</h3>
                        <table class="table">
                            <tbody>
                                <tr>
                                    <th><a class="btn btn-primary" href="https://dl.nsec/gband-windows.zip" role="button">{ "Windows x64" }</a></th>
                                    <th><a class="btn btn-primary" href="https://dl.nsec/gband-linux.zip" role="button">{ "Linux x64" }</a></th>
                                </tr>
                            </tbody>
                        </table>

                        <h3>{ "Try our new game's demo!" }</h3>
                        <p><a class="btn btn-danger" href="https://dl.nsec/super-myco-boi.gbc" role="button">{ "Super Myco Boi!" }</a></p>
                    </div>

                    <div class="col">
                        <h3>{ "Sample ROMs" }</h3>
                        <table class="table table-hover">
                            <tbody>
                                <tr>
                                    <th><a class="btn btn-secondary" href="/roms/desertboy.gb" role="button">{ "desertboy.gb" }</a></th>
                                    <th><a class="btn btn-link" href="https://ekimekim.itch.io/desert-bus-for-gameboy" role="button">{ "Source" }</a></th>
                                </tr>
                                <tr>
                                    <th><a class="btn btn-secondary" href="/roms/flappyboy.gb" role="button">{ "flappyboy.gb" }</a></th>
                                    <th><a class="btn btn-link" href="https://github.com/bitnenfer/flappy-boy-asm" role="button">{ "Source" }</a></th>
                                </tr>
                                <tr>
                                    <th><a class="btn btn-secondary" href="/roms/ucity.gbc" role="button">{ "ucity.gbc" }</a></th>
                                    <th><a class="btn btn-link" href="https://github.com/AntonioND/ucity" role="button">{ "Source" }</a></th>
                                </tr>
                                <tr>
                                    <th><a class="btn btn-secondary" href="/roms/RenegadeRush.gb" role="button">{ "RenegadeRush.gb" }</a></th>
                                    <th><a class="btn btn-link" href="https://quinnp.itch.io/renegade-rush" role="button">{ "Source" }</a></th>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>

                <h1> { "System Requirements" } </h1>
                <table class="table">
                    <tbody>
                        <tr>
                            <th> {"Linux"} </th>
                            <th> {"A modern system and support for Vulkan or OpenGL 3"} </th>
                        </tr>
                        <tr>
                            <th> {"Windows"} </th>
                            <th> {"A modern system and support for DirectX 11, DirectX 12 or Vulkan."} </th>
                        </tr>
                    </tbody>
                </table>
                <a href="https://github.com/gfx-rs/wgpu"> {"For more compatibility information, please refer to the WGPU compatibily matrix."} </a>
            </main>

            <footer class="container">
                <p>{ "© 2022 Ouyaya" }</p>
                <p>
                    <a href="#">{ "Back to top" }</a>
                </p>
            </footer>
            </body>
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
            AppMessage::RomSample(sample_name) => {
                if sample_name == "none" {
                    return false;
                }

                ctx.link().send_future(async move {
                    let rom = fetch_rom(&sample_name).await;
                    AppMessage::RomBytes(rom)
                });

                false
            }
            AppMessage::RomBytes(rom) => {
                self.rom = Some(rom);
                true
            }
        }
    }
}

async fn fetch_rom(sample_name: &str) -> Vec<u8> {
    use gloo::net::http::Request;

    let url = format!("/roms/{}", sample_name);
    let resp = Request::get(&url).send().await.unwrap();
    let rom = resp.binary().await.unwrap();

    rom
}
