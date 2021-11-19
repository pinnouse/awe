//! CHIP8 emulator
//!
//! Written heavily with help from
//! <https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/>

use std::collections::HashMap;
use rand::prelude::*;
use wasm_bindgen::JsValue;
use crate::wasm_bindgen;
use super::Emulator;

const FONTSET_SIZE: usize = 0x50;
const MEMORY_SIZE: usize = 4096;
const GFX_W: usize = 64;
const GFX_H: usize = 32;
const GFX_SIZE: usize = GFX_W * GFX_H;
static CHIP8_FONTSET: [u8; FONTSET_SIZE] = [0; FONTSET_SIZE];

#[wasm_bindgen]
pub struct CHIP8 {
    opcode:         u16,                // Opcode
    memory:         [u8; MEMORY_SIZE],  // 4096 bytes of memory
    V:              [u8; 16],           // 16 registers, V0-VF
    I:              u16,                // Index register for addresses
    pc:             u16,                // Program counter
    gfx:            [u8; GFX_SIZE],     // Graphics memory

    // CHIP8 timers counting down from at 60Hz
    delay_timer:    u8,                 // Game timer
    sound_timer:    u8,                 // Sound timer

    // Stack
    stack:          [u16; 16],          // The stack
    sp:             u16,                // Stack pointer

    // Keyboard
    key:            [u8; 16],           // Key states
    next_key:       i8,                 // Get next key (-2 is waiting, -1 is no key)

    // Draw flag
    draw_flag:      bool,               // Draw or not
}

impl Emulator for CHIP8 {
    fn e_new() -> Self {
        let mut em = Emulator {
            opcode: 0,
            memory: [0; MEMORY_SIZE],
            V: [0; 16],
            I: 0,
            pc: 0x200, // Set program counter start (512) since the first 512 are stored for interpreter
            gfx: [0; GFX_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
            next_key: -1,
            draw_flag: false,
        };

        // Font starts at 0x50 = 80
        for i in 0..FONTSET_SIZE {
            em.memory[i] = CHIP8_FONTSET[i];
        }

        em
    }

    fn e_load(&mut self, data: Vec<u8>) {
        for i in 0..data.len() {
            self.memory[i + 0x200] = data[i]; // Start loading the data past the interpreter data
        }
    }

    fn e_execute_op(&mut self, opcode: u64) {
        // Decode opcode
        // https://en.wikipedia.org/wiki/CHIP-8#Opcode_table

        let code_type = self.opcode & 0xF000;
        match code_type {
            0x0 => {
                let code_type = self.opcode & 0xFF;
                match code_type {
                    0xE0 => {
                        self.gfx = [0; GFX_SIZE];
                        self.draw_flag = true;
                    },
                    0xEE => {
                        if self.sp == 0 {
                            eprintln!("Bottom of stack, cannot return! at: {0}", self.pc);
                            return;
                        }
                        self.pc = self.stack[self.sp - 1];
                        self.sp -= 1;
                    },
                    _ => { eprintln!("Unknown opcode: {0}", self.opcode); }
                }
            },
            0x1000 => {
                self.pc = self.opcode & 0xFFF;
            },
            0x2000 => {
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = self.opcode & 0xFFF;
            },
            0x3000 => {
                let reg = (self.opcode & 0xF00) >> 16;
                if self.V[reg] == self.opcode & 0xFF {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0x4000 => {
                let reg = (self.opcode & 0xF00) >> 16;
                if self.V[reg] != self.opcode & 0xFF {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0x5000 => {
                let reg_x = (self.opcode & 0xF00) >> 16;
                let reg_y = (self.opcode & 0xF0) >> 8;
                if self.V[reg_x] == self.V[reg_y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0x6000 => {
                let reg = (self.opcode & 0xF00) >> 16;
                self.V[reg] = self.opcode & 0xFF;
                self.pc += 2;
            },
            0x7000 => {
                let reg = (self.opcode & 0xF00) >> 16;
                self.V[reg] += self.opcode & 0xFF;
                self.pc += 2;
            },
            0x8000 => {
                let code_type = self.opcode & 0xF;
                let reg_x = (self.opcode & 0xF00) >> 16;
                let reg_y = (self.opcode & 0xF0) >> 8;
                match code_type {
                    0x0 => { self.V[reg_x] = self.V[reg_y]; self.pc += 2; },
                    0x1 => { self.V[reg_x] |= self.V[reg_y]; self.pc += 2; },
                    0x2 => { self.V[reg_x] &= self.V[reg_y]; self.pc += 2; },
                    0x3 => { self.V[reg_x] ^= self.V[reg_y]; self.pc += 2; },
                    0x4 => { self.V[reg_x] += self.V[reg_y]; self.pc += 2; },
                    0x5 => { self.V[reg_x] -= self.V[reg_y]; self.pc += 2; },
                    0x6 => { self.V[0xF] = self.V[reg_x] & 0b1; self.V[reg_x] >>= 1; self.pc += 2; },
                    0x7 => { self.V[reg_x] = self.V[reg_y] - self.V[reg_x]; self.pc += 2; },
                    0xE => { self.V[0xF] = self.V[reg_x] & 0b10000000; self.V[reg_x] <<= 1; self.pc += 2; },
                    _ => { eprintln!("Unknown opcode: {0}", self.opcode); },
                }
            },
            0x9000 => {
                let reg_x = (self.opcode & 0xF00) >> 16;
                let reg_y = (self.opcode & 0xF0) >> 8;
                if self.V[reg_x] != self.V[reg_y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0xA000 => { // Sets I to the address NNN.
                self.i = self.opcode & 0xFFF;
                self.pc += 2;
            },
            0xB000 => {
                self.pc = self.V[0] as u16 + (self.opcode & 0xFFF);
            },
            0xC000 => {
                let reg = (self.opcode & 0xF00) >> 16;
                self.V[reg] = rand::random::<u8>() & self.opcode & 0xFF;
                self.pc += 2;
            },
            0xD000 => {
                let reg_x = (self.opcode & 0xF00) >> 16;
                let reg_y = (self.opcode & 0xF0) >> 8;
                let height = self.opcode & 0xF;
                let start_position = self.V[reg_y] * GFX_W + self.V[reg_x];
                self.V[0xF] = 0;
                for i in 0..height { // Paint row of 8 pixels at a time
                    self.V[0xF] |= {
                        let pos_gfx = start_position + (i * GFX_W);
                        let pos_mem = self.I + i;
                        let mut x = self.memory[pos_mem];
                        let mut r = 0;
                        for j in 0..8 {
                            r |= (self.gfx[pos_gfx + 7 - j]) ^ (x % 2);
                            self.gfx[pos_gfx + 7 - j] = x % 2;
                            x >>= 1;
                        }
                        r
                    };
                }
                self.pc += 2;
            },
            0xE000 => {
                let code_type = self.opcode & 0xFF;
                let reg = (self.opcode & 0xF00) >> 16;
                let key = self.key[self.V[reg]];
                match code_type {
                    0x9E => {
                        if key {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    },
                    0xA1 => {
                        if !key {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                    },
                    _ => { eprintln!("Unknown opcode: {0}", self.opcode); }
                }
            },
            0xF000 => {
                let code_type = self.opcode & 0xFF;
                let reg = (self.opcode & 0xF00) >> 16;
                match code_type {
                    0x07 => { self.V[reg] = self.delay_timer; self.pc += 2; },
                    0x0A => { self.next_key = -2; },
                    0x15 => { self.delay_timer = self.V[reg]; self.pc += 2; },
                    0x18 => { self.sound_timer = self.V[reg]; self.pc += 2; },
                    0x1E => { self.I += self.V[reg]; self.pc += 2; },
                    0x29 => { self.I = self.V[reg]; self.pc += 2; },
                    0x33 => {
                        self.memory[self.I] = self.V[reg] / 100;
                        self.memory[self.I + 1] = (self.V[reg] / 10) % 10;
                        self.memory[self.I + 2] = self.V[reg] % 10;
                        self.pc += 2;
                    },
                    0x55 => {
                        let mem = &mut self.memory[self.I..self.I + reg + 1];
                        mem[..reg + 1].clone_from_slice(self.V[..reg + 1]);
                        self.pc += 2;
                    },
                    0x65 => {
                        let registers = &mut self.V[..];
                        registers[..reg + 1].clone_from_slice(self.memory[self.I..self.I + reg + 1]);
                        self.pc += 2;
                    },
                    _ => { eprintln!("Unknown opcode: {0}", self.opcode); }
                }
            },
            _ => { eprintln!("Unknown opcode: {0}", self.opcode); },
        }
    }

    fn e_update(&mut self) {
        if self.next_key == -2 {
            todo!("Block and get next key");
            self.pc += 2;
        } else if self.next_key >= 0 {
            self.key[self.next_key] = 1;
        }

        // Set opcode stored in big endian
        self.opcode = self.memory[self.pc] << 8 | self.memory[self.pc + 1];

        self.e_execute_op(self.opcode as u64);

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }
    }

    fn e_set_metadata(&mut self, metadata: HashMap<String, JsValue>) { /* Nothing is required to set */ }

    fn e_draw(&mut self) {
        if !self.draw_flag {
            return;
        }
        todo!();

        self.draw_flag = false;
    }

    fn e_set_input(&mut self) {
        todo!()
    }

    fn e_reset(&mut self) {
        self.memory = [0; MEMORY_SIZE];
        self.pc = 0x200;
        self.stack = [0; 16];
        self.V = [0; 16];
        self.key = [0; 16];
        self.next_key = -1;
        self.sp = 0;
    }
}

#[wasm_bindgen]
impl CHIP8 {
    /// Initializes a new CHIP8 emulator object
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { CHIP8::e_new() }

    /// Loads a CHIP8 ROM file into memory
    #[wasm_bindgen]
    pub fn load(&mut self, data: Vec<u8>) { self.e_load(data) }

    /// Update loop for the emulator
    #[wasm_bindgen]
    pub fn update(&mut self) { self.e_update() }

    /// Drawing the monochromatic display for the emulator
    #[wasm_bindgen]
    pub fn draw(&mut self) { self.e_draw() }

    /// Function for setting arbitrary metadata for the system
    #[wasm_bindgen]
    pub fn set_metadata(&mut self, metadata: HashMap<String, JsValue>) { self.e_set_metadata(metadata) }

    /// Sets the hex keyboard input
    #[wasm_bindgen]
    pub fn set_input(&mut self) { self.e_set_input() }

    /// Resets the memory of the CHIP8 emulator
    #[wasm_bindgen]
    pub fn reset(&mut self) { self.e_reset() }
}
