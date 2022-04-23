use anyhow::Context as _;
use clap::Parser;
use gband_server::InitialInputs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tap::{Pipe as _, Tap as _};

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Path to the ROM to load
    #[clap(short, long)]
    rom: PathBuf,

    /// Path to the inital inputs file
    #[clap(short, long)]
    inputs: PathBuf,

    /// Address to listen to
    #[clap(short, long, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2345))]
    addr: SocketAddr,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let rom_path = cli.rom;
    let save_data_path = rom_path.clone().tap_mut(|path| {
        path.set_extension("sav");
    });
    let inputs_path = cli.inputs;

    let rom = load_once(&rom_path)
        .with_context(|| format!("Couldn't read ROM {}", rom_path.display()))?;
    let save_data = load_once(&save_data_path)
        .with_context(|| format!("Couldn't read save data {}", save_data_path.display()))?;

    let inputs = std::fs::read_to_string(&inputs_path)
        .with_context(|| format!("Couldn't read initial inputs {}", inputs_path.display()))?;
    let inputs = InitialInputs::parse_str(&inputs)
        .context("Invalid initial inputs")?
        .pipe(Box::new)
        .pipe(Box::leak);

    let ctx = Ctx {
        rom,
        save_data,
        inputs,
    };

    run(cli.addr, ctx)
}

fn load_once(path: &Path) -> anyhow::Result<&'static [u8]> {
    use std::fs::File;
    use std::io::Read as _;

    let mut file = File::open(path)?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    // It's fine to just leak the contents, because this is data loaded only once
    // and expected to live until the end of the program.
    let contents = contents.leak();

    Ok(contents)
}

#[derive(Clone, Copy)]
struct Ctx {
    rom: &'static [u8],
    save_data: &'static [u8],
    inputs: &'static InitialInputs,
}

fn run(addr: SocketAddr, ctx: Ctx) -> anyhow::Result<()> {
    use std::net::TcpListener;
    use std::thread;

    let listener = TcpListener::bind(addr)?;
    println!("Listening on {addr}");

    loop {
        match listener.accept() {
            Ok((socket, addr)) => {
                println!("New client: {addr}");
                thread::spawn(move || match handle_client(socket, ctx) {
                    Ok(()) => println!("{addr} ended gracefully"),
                    Err(e) => println!("{addr} failed: {e:#}"),
                });
            }
            Err(e) => println!("Couldn't get client: {e}"),
        }
    }
}

fn handle_client(stream: std::net::TcpStream, ctx: Ctx) -> anyhow::Result<()> {
    use gband::{Emulator, JoypadState};
    use gband_server::EventType;
    use spin_sleep::LoopHelper;

    let mut emulator = Emulator::new(ctx.rom, Some(ctx.save_data))
        .map_err(anyhow::Error::msg)
        .context("Couldn't parse ROM")?;

    let mut joypad_state = JoypadState::default();
    let mut frame_count = 0;

    for input in &ctx.inputs.0 {
        while frame_count < input.frame {
            run_to_frame(&mut emulator);
            frame_count += 1;
        }

        input
            .buttons
            .iter()
            .copied()
            .map(JoypadState::from)
            .for_each(|button| match input.ty {
                EventType::Pressed => joypad_state.insert(button),
                EventType::Released => joypad_state.remove(button),
            });

        emulator.set_joypad(joypad_state);
    }

    // Execute one last frame with the final joypad state, and then reset joypad state before serial communication
    run_to_frame(&mut emulator);
    emulator.set_joypad(JoypadState::default());

    // set the serial transport
    let active = Arc::new(AtomicBool::new(true));
    let serial_transport = TcpSerialTransport::new(stream, active.clone());
    emulator.set_serial(Box::new(serial_transport));

    let mut loop_helper = LoopHelper::builder().build_with_target_rate(60.0);

    loop {
        loop_helper.loop_start();

        run_to_frame(&mut emulator);

        if !active.load(Ordering::Acquire) {
            break;
        }

        loop_helper.loop_sleep();
    }

    Ok(())
}

fn run_to_frame(emulator: &mut gband::Emulator) {
    while emulator.clock().is_none() {}
}

struct TcpSerialTransport {
    stream: std::net::TcpStream,
    active: Arc<AtomicBool>,
}

impl TcpSerialTransport {
    pub fn new(stream: std::net::TcpStream, active: Arc<AtomicBool>) -> Self {
        let _ = stream.set_nonblocking(true);
        let _ = stream.set_nodelay(true);
        Self { stream, active }
    }
}

impl gband::SerialTransport for TcpSerialTransport {
    fn connect(&mut self) -> bool {
        false
    }

    fn is_connected(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    fn reset(&mut self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
        self.active.store(false, Ordering::Release);
    }

    fn send(&mut self, data: u8) {
        use std::io::Write as _;
        if self.stream.write(&[data]).is_err() {
            self.reset();
        }
    }

    fn recv(&mut self) -> Option<u8> {
        use std::io::Read as _;
        let mut buf = [0];
        match self.stream.read(&mut buf) {
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
            Ok(0) | Err(_) => {
                self.reset();
                None
            }
            Ok(_) => Some(buf[0]),
        }
    }
}
