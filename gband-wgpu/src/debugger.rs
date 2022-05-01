use crate::emulation_thread::{EmulatorInput, EmulatorState};
use crate::State;

use std::io::{stdin, stdout, Write};

use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(no_version)]
/// GBAND debugger commands, inspired by GDB
pub enum DebuggerOpt {
    #[structopt(visible_alias = "c", no_version)]
    /// Resume execution of the emulator
    Continue,

    #[structopt(visible_alias = "b", no_version)]
    /// Set breakpoint at the specified location
    Break {
        #[structopt(parse(try_from_str = parse_hex_addr))]
        /// Address to break on. The breakpoint will be placed at the nearest instruction of the currently loaded bank
        addr: u16,
    },

    #[structopt(visible_alias = "del", no_version)]
    /// Remove a breakpoint with the specified index, or all breakpoints if no index is passed.
    Delete {
        /// Index of the breakpoint to remove.
        index: Option<usize>,
    },

    #[structopt(visible_alias = "s", no_version)]
    /// Execute one CPU instruction
    Step,

    #[structopt(visible_alias = "i", no_version)]
    /// Print various information
    Info(DebuggerInfoOpt),

    #[structopt(visible_alias = "disas", no_version)]
    /// Print the disassembly from the currently loaded program banks.
    Disassemble {
        #[structopt(parse(try_from_str = parse_hex_addr))]
        /// Reference address to search. If missing, the reference will be the current instruction
        search_addr: Option<u16>,
    },

    #[structopt(visible_alias = "x", no_version)]
    /// Print an hex dump of the CPU memory
    Hexdump {
        #[structopt(parse(try_from_str = parse_hex_addr))]
        /// Beginning of the range to print. Minimum is 0x0000
        start_addr: u16,
        #[structopt(parse(try_from_str = parse_hex_addr))]
        /// End of the range to print. Maximum is 0xFFFF
        end_addr: u16,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(no_version)]
pub enum DebuggerInfoOpt {
    #[structopt(visible_alias = "b", no_version)]
    /// Display the breakpoints currently set
    Break,
    #[structopt(visible_alias = "r", no_version)]
    /// Display registers, or a specific register if specified
    Reg { register: Option<String> },
}

fn parse_hex_addr(src: &str) -> Result<u16, std::num::ParseIntError> {
    let src = src.trim_start_matches("0x");
    u16::from_str_radix(src, 16)
}

impl EmulatorState {
    pub fn handle_debugger_inputs(&mut self, input: DebuggerOpt) {
        match input {
            DebuggerOpt::Continue => self
                .paused
                .store(false, std::sync::atomic::Ordering::Relaxed),
            DebuggerOpt::Step => {
                let current_pc = self.emulator.cpu().pc;
                while {
                    if let Some(step_frame) = self.emulator.clock() {
                        self.update_frame(step_frame.as_slice());
                    }
                    self.emulator.cpu().cycles > 1 || self.emulator.cpu().pc == current_pc
                } {}

                self.disassemble(None);
                self.print_registers(None);
            }
            DebuggerOpt::Break { addr } => {
                let disassembly = self.emulator.disassemble(0, 0);
                let closest_addr = disassembly
                    .iter()
                    .min_by_key(|&(_, x, _)| (x.wrapping_sub(addr)))
                    .unwrap()
                    .1;

                self.breakpoints.push(closest_addr);
                println!("Added breakpoint at {:04x}", closest_addr);
            }
            DebuggerOpt::Delete { index } => {
                if let Some(index) = index {
                    let removed = self.breakpoints.remove(index);
                    println!("Removed breakpoint {}: {:04x}", index, removed);
                } else {
                    self.breakpoints.clear();
                    println!("Cleared all breakpoints");
                }
            }
            DebuggerOpt::Disassemble { search_addr } => self.disassemble(search_addr),
            DebuggerOpt::Hexdump {
                start_addr,
                end_addr,
            } => {
                // Align start address to get 16 values
                let start_addr = start_addr - (start_addr & 0xf);
                let data = self.emulator.mem_dump(start_addr, end_addr);

                for (i, chunk) in data.chunks(16).enumerate() {
                    let addr = start_addr + (16 * i as u16);

                    let bytes = chunk
                        .iter()
                        .map(|c| format!("{:02x}", c))
                        .collect::<Vec<_>>()
                        .join(" ");

                    let ascii = chunk
                        .iter()
                        .map(|num| {
                            if *num >= 32 && *num <= 126 {
                                (*num as char).to_string()
                            } else {
                                '.'.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("");

                    println!("{:04x}: {:47} {}", addr, bytes, ascii);
                }
            }
            DebuggerOpt::Info(info) => match info {
                DebuggerInfoOpt::Break => {
                    for (index, addr) in self.breakpoints.iter().enumerate() {
                        println!("Breakpoint {}: {:04x}", index, addr);
                    }
                }
                DebuggerInfoOpt::Reg { register } => self.print_registers(register),
            },
        }
    }

    fn disassemble(&mut self, search_addr: Option<u16>) {
        let disassembly = self.emulator.disassemble(0, 0);
        let cpu = self.emulator.cpu();

        let center_addr = if let Some(search_addr) = search_addr {
            search_addr
        } else {
            cpu.pc
        };

        for (bank, addr, disas) in &disassembly {
            if (*addr as usize) > (center_addr as usize) - 20
                && (*addr as usize) < (center_addr as usize) + 20
            {
                let prefix = if *addr == cpu.pc || *addr == (cpu.pc - 1) {
                    ">"
                } else {
                    " "
                };

                println!("{} {:02x}:{:04x}: {}", prefix, bank, addr, disas);
            }
        }
    }

    fn print_registers(&self, register: Option<String>) {
        let cpu = self.emulator.cpu();

        if let Some(register) = register {
            match register.as_str() {
                "a" => println!("a: {:02x}", cpu.a),
                "f" => println!("f: {:02x}", cpu.f),
                "b" => println!("b: {:02x}", cpu.b),
                "c" => println!("c: {:02x}", cpu.c),
                "d" => println!("d: {:02x}", cpu.d),
                "e" => println!("e: {:02x}", cpu.e),
                "h" => println!("h: {:02x}", cpu.h),
                "l" => println!("l: {:02x}", cpu.l),
                "af" => println!("af: {:02x}{:02x}", cpu.a, cpu.f),
                "bc" => println!("bc: {:02x}{:02x}", cpu.b, cpu.c),
                "de" => println!("de: {:02x}{:02x}", cpu.d, cpu.e),
                "hl" => println!("hl: {:02x}{:02x}", cpu.h, cpu.l),
                "sp" => println!("sp: {:04x}", cpu.sp),
                "pc" => println!("pc: {:04x}", cpu.pc),
                reg => println!("Unknown register: {}", reg),
            }
        } else {
            println!(
                "af: {:02x}{:02x}  bc: {:02x}{:02x}  de: {:02x}{:02x}",
                cpu.a, cpu.f, cpu.b, cpu.c, cpu.d, cpu.e
            );
            println!(
                "hl: {:02x}{:02x}  sp: {:04x}  pc: {:04x}",
                cpu.h, cpu.l, cpu.sp, cpu.pc
            );
        }
    }
}

impl State {
    pub fn debugger_prompt(&mut self) {
        print!("debugger> ");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let tokens = input.split_ascii_whitespace();
        let clap = DebuggerOpt::clap()
            .global_setting(AppSettings::NoBinaryName)
            .global_setting(AppSettings::DisableVersion)
            .global_setting(AppSettings::VersionlessSubcommands)
            .get_matches_from_safe(tokens);

        match clap {
            Ok(clap) => {
                let opt = DebuggerOpt::from_clap(&clap);
                self.emulator_input
                    .send(EmulatorInput::DebuggerInput(opt))
                    .expect("Emulation thread crashed!");
            }
            Err(e) => println!("{}", e.message),
        }
    }
}
