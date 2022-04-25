use bitflags::bitflags;

#[derive(Default)]
pub struct TimerRegisters {
    div: u16,
    counter: u8,
    modulo: u8,
    control: TimerControl,

    tac_falling_edge_latch: bool,
    interrupt_cycle_countdown: u8,
}

impl TimerRegisters {
    pub fn clock(&mut self) -> bool {
        // Accuracy shennanigans
        // If TIMA is written on this specific cycle, the interrupt is not fired.
        let result = self.interrupt_cycle_countdown == 2 && self.counter == 0;

        match self.interrupt_cycle_countdown {
            // This is the cycle the interrupt is thrown
            2 => {
                self.interrupt_cycle_countdown = 1;

                if self.counter == 0 {
                    self.counter = self.modulo;
                }
            }
            // This is the cycle the counter get reset to the modulo
            // Note that the counter itself gets reset in the emulator the previous cycle,
            //      however this is because we emulate the cycles sequentially, not concurrently,
            //      so the actual electrical circuit is hard to represent.
            1 => {
                self.interrupt_cycle_countdown = 0;
            }
            _ => {}
        }

        // This is incremented 4 times per CPU clock.
        // Since we don't emulate the CPU sub-cycle, we can approximate it this way.
        self.set_div(self.div.wrapping_add(4));

        result
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            // DIV register. Resets the timer to 0
            0xFF04 => self.reset_div(),

            // TIMA counter
            0xFF05 => {
                if self.interrupt_cycle_countdown != 1 {
                    // On this specific cycle, the value gets overwritten by TMA, so the write is ignored
                    self.counter = data;
                }
            }

            // TMA modulo
            0xFF06 => {
                self.modulo = data;
                if self.interrupt_cycle_countdown == 1 {
                    self.counter = data;
                }
            }

            // Control
            0xFF07 => self.control.bits = data & 0x07,
            _ => {}
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            // DIV register. Reads the upper 8 bits
            0xFF04 => (self.div >> 8) as u8,

            // TIMA counter
            0xFF05 => self.counter,

            // TMA modulo
            0xFF06 => self.modulo,

            // Control
            0xFF07 => (self.control | TimerControl::UNUSED).bits,
            _ => 0xFF,
        }
    }

    pub fn reset_div(&mut self) {
        self.set_div(0);
    }

    fn set_div(&mut self, div: u16) {
        self.div = div;

        // Select the bit the timer is listening on
        let mask = self.control.get_mask();

        // This comes from the electrical circuit of the timer.
        // We cache the last result of "is this bit on AND is timer enabled", and then see if it fell since last clock.
        // Note that this method cause some bugs on the actual GB hardware because there can be false positive when the timer is reset.
        let latch = self.control.contains(TimerControl::ENABLED) && self.div & mask == mask;

        if !latch & self.tac_falling_edge_latch {
            // Interrupt is delayed one cycle, during which TIMA is 0.
            // We use 2 because there's another behavior to emulate another cycle later.
            if self.counter == 0xFF {
                self.interrupt_cycle_countdown = 2;
            };

            self.counter = self.counter.wrapping_add(1);
        }

        self.tac_falling_edge_latch = latch;
    }
}

bitflags! {
    #[derive(Default)]
    pub struct TimerControl: u8 {
        const CLOCK_1024 = 0x00;
        const CLOCK_16 = 0x01;
        const CLOCK_64 = 0x02;
        const CLOCK_256 = 0x03;
        const ENABLED = 0x04;
        const UNUSED = 0xf8;
    }
}

impl TimerControl {
    pub fn get_mask(&self) -> u16 {
        match *self & TimerControl::CLOCK_256 {
            TimerControl::CLOCK_1024 => 1 << 9,
            TimerControl::CLOCK_16 => 1 << 3,
            TimerControl::CLOCK_64 => 1 << 5,
            TimerControl::CLOCK_256 => 1 << 7,
            _ => unreachable!(),
        }
    }
}
