use crate::debugger::DebuggerOpt;
use gband::{Emulator, JoypadState};
use spin_sleep::LoopHelper;
use std::{
    sync::{atomic::AtomicBool, mpsc, Arc},
    thread::JoinHandle,
};

pub enum EmulatorInput {
    Input(JoypadState),
    RequestSaveData(mpsc::Sender<Option<Vec<u8>>>),
    DebuggerInput(DebuggerOpt),
    Stop,
}

pub struct EmulatorState {
    pub emulator: Emulator,

    pub paused: Arc<AtomicBool>,
    pub breakpoints: Vec<u16>,

    queue: Arc<wgpu::Queue>,
    texture: wgpu::Texture,

    input_receiver: mpsc::Receiver<EmulatorInput>,
    loop_helper: LoopHelper,
}

impl EmulatorState {
    pub fn run(&mut self) {
        loop {
            if self.paused.load(std::sync::atomic::Ordering::Relaxed) {
                // Block on input if paused
                if let Ok(input) = self.input_receiver.recv() {
                    if self.handle_inputs(input) {
                        // Stop the thread if a stop is requested
                        break;
                    }
                }
            } else {
                // Don't block if not paused
                if let Ok(input) = self.input_receiver.try_recv() {
                    if self.handle_inputs(input) {
                        // Stop the thread if a stop is requested
                        break;
                    }
                }

                // Emulation not paused, continue to run
                self.loop_helper.loop_start();

                // Get a frame from the emulation and write it to the texture
                let frame = loop {
                    if let Some(f) = self.emulator.clock() {
                        break f;
                    }
                };

                self.update_frame(frame.as_slice());

                self.loop_helper.loop_sleep();
            }
        }
    }

    pub fn update_frame(&self, frame: &[u8]) {
        let emulator_width = gband::FRAME_WIDTH as u32;
        let emulator_height = gband::FRAME_HEIGHT as u32;

        // Update texture
        let texture_size = wgpu::Extent3d {
            width: emulator_width,
            height: emulator_height,
            depth_or_array_layers: 1,
        };

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            frame,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * emulator_width),
                rows_per_image: std::num::NonZeroU32::new(emulator_height),
            },
            texture_size,
        );
    }

    fn handle_inputs(&mut self, input: EmulatorInput) -> bool {
        match input {
            EmulatorInput::Input(x) => self.emulator.set_joypad(x),
            EmulatorInput::RequestSaveData(sender) => {
                let save = match self.emulator.get_save_data() {
                    Some(save) => Some(save.to_vec()),
                    _ => None,
                };
                let _ = sender.send(save);
            }
            EmulatorInput::DebuggerInput(x) => self.handle_debugger_inputs(x),
            EmulatorInput::Stop => {
                return true;
            }
        }

        false
    }
}

pub fn start(
    emulator: Emulator,
    queue: Arc<wgpu::Queue>,
    texture: wgpu::Texture,
    paused: Arc<AtomicBool>,
) -> (JoinHandle<()>, mpsc::Sender<EmulatorInput>) {
    let (input_sender, input_receiver) = mpsc::channel::<EmulatorInput>();

    let loop_helper = LoopHelper::builder()
        .report_interval_s(0.5)
        .build_with_target_rate(59.727500569606);

    let mut emulator_state = EmulatorState {
        emulator,
        queue,
        texture,

        paused,
        breakpoints: Vec::new(),

        input_receiver,
        loop_helper,
    };

    let join_handle = std::thread::spawn(move || emulator_state.run());

    (join_handle, input_sender)
}
