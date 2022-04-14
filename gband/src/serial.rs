use alloc::boxed::Box;
use bitflags::bitflags;

use crate::{NullSerialTransport, SerialTransport};

const N_BIT_CYCLES: u8 = 8;
const CPU_CYCLES: u8 = (4194304 / 4 / 8192) as u8;
const CPU_CYCLES_FAST: u8 = (4194304 / 4 / 262144) as u8;

bitflags! {
    struct ControlRegister: u8 {
        const MASTER = 0x01;
        const FAST = 0x02;
        const UNUSED = 0x7C;
        const START = 0x80;
    }
}

impl Default for ControlRegister {
    fn default() -> Self {
        ControlRegister::UNUSED | ControlRegister::FAST
    }
}

pub struct SerialPort {
    buffer: u8,
    control: ControlRegister,

    freq_downscale_cycle: u8,
    bit_cycle: u8,
    receive_latch: u8,

    serial_transport: Box<dyn SerialTransport>,
    skip_send: bool,
}

impl Default for SerialPort {
    fn default() -> Self {
        Self {
            buffer: Default::default(),
            control: Default::default(),

            freq_downscale_cycle: Default::default(),
            bit_cycle: Default::default(),
            receive_latch: Default::default(),

            serial_transport: Box::new(NullSerialTransport),
            skip_send: false,
        }
    }
}

impl SerialPort {
    /// Clock the serial port module.
    /// Returns a bool indicating whether an interrupt is triggered or not
    pub fn clock(&mut self) -> bool {
        self.freq_downscale_cycle += 1;

        let speed = if self.control.contains(ControlRegister::FAST) {
            CPU_CYCLES_FAST
        } else {
            CPU_CYCLES
        };

        if self.freq_downscale_cycle >= speed {
            self.freq_downscale_cycle = 0;

            if self.control.contains(ControlRegister::START) {
                self.run_transfer()
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn set_serial(&mut self, serial: alloc::boxed::Box<dyn SerialTransport>) {
        self.serial_transport = serial
    }

    fn run_transfer(&mut self) -> bool {
        if self.bit_cycle == 0 {
            if !self.serial_transport.is_connected() {
                self.serial_transport.connect();
                self.skip_send = false;
            }

            if self.serial_transport.is_connected() {
                if self.control.contains(ControlRegister::MASTER) {
                    if !self.skip_send {
                        self.serial_transport.send(self.buffer)
                    }

                    match self.serial_transport.recv() {
                        Some(received) => {
                            self.skip_send = false;
                            self.receive_latch = received;
                        }
                        None => {
                            self.skip_send = true;
                            return false;
                        }
                    }
                } else {
                    match self.serial_transport.recv() {
                        Some(received) => {
                            self.receive_latch = received;
                        }
                        None => return false,
                    }

                    self.serial_transport.send(self.buffer)
                }
            } else {
                self.serial_transport.reset();
            }
        }

        // Increment "bits transferred" cycles only if the connection is still active
        if self.serial_transport.is_connected() {
            self.bit_cycle += 1;
        } else {
            self.serial_transport.reset();
            self.bit_cycle = 0;
        }

        if self.bit_cycle == N_BIT_CYCLES {
            self.bit_cycle = 0;
            self.buffer = self.receive_latch;
            self.control.remove(ControlRegister::START);
            true
        } else {
            false
        }
    }

    pub fn set_buffer(&mut self, data: u8) {
        self.buffer = data;
    }

    pub fn get_buffer(&self) -> u8 {
        self.buffer
    }

    pub fn set_control(&mut self, data: u8) {
        self.control = ControlRegister::from_bits_truncate(data) | ControlRegister::UNUSED;
    }

    pub fn get_control(&self) -> u8 {
        self.control.bits()
    }
}
