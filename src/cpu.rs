use system::System;
use opcodes::Opcode;

extern crate rand;

const SIZE_STACK: usize = 16;
const NUM_REG: usize = 18;

// Special registers
const REG_V0: u8 = 0;
const REG_VF: u8 = 15;
const REG_DT: u8 = 16;
const REG_ST: u8 = 17;

pub struct Cpu {
    regs:  [u8; NUM_REG],
    stack: Vec<usize>,
    sys: System,

    idx: u16,
    pc: usize,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs:  [0; NUM_REG],
            stack: vec![0; SIZE_STACK],
            sys: System::new(),
            idx: 0,
            pc: 0,
        }
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch();

    }

    fn fetch(&mut self) -> u16 {
        // Get the two bytes pointed at by the program counter
        let b0 = self.sys.get_mem(self.pc);
        let b1 = self.sys.get_mem(self.pc + 1);

        // Increment the program counter to the next instruction (2 bytes)
        self.pc += 2;

        // Combine the two bytes to a 16-bit opcode
        ((b0 as u16) << 8) | (b1 as u16)
    }

    fn decode(&self, opcode: u16) -> Opcode {
        match (opcode & 0xF000) >> 12 {
            0x0 => match opcode & 0x0FFF {
                0x0E0 => Opcode::CLS,
                0x0EE => Opcode::RET,
                addr  => Opcode::SYS(addr),
            },
            0x1 => Opcode::JP(addr(opcode)),
            0x2 => Opcode::CALL(addr(opcode)),
            0x3 => Opcode::SE(regx(opcode), byte(opcode)),
            0x4 => Opcode::SNE(regx(opcode), byte(opcode)),
            0x5 => Opcode::SE(regx(opcode), regy(opcode)),
            0x6 => Opcode::LD(regx(opcode), byte(opcode)),
            0x7 => Opcode::ADD(regx(opcode), byte(opcode)),
            0x8 => match nibble(opcode) {
                0x0 => Opcode::LD(regx(opcode), regy(opcode)),
                0x1 => Opcode::OR(regx(opcode), regy(opcode)),
                0x2 => Opcode::AND(regx(opcode), regy(opcode)),
                0x3 => Opcode::XOR(regx(opcode), regy(opcode)),
                0x4 => Opcode::ADD(regx(opcode), regy(opcode)),
                0x5 => Opcode::SUB(regx(opcode), regy(opcode)),
                0x6 => Opcode::SHR(regx(opcode)),
                0x7 => Opcode::SUBN(regx(opcode), regy(opcode)),
                0xE => Opcode::SHL(regx(opcode), regy(opcode)),
                _   => unimplemented!(),
            },
            0x9 => Opcode::SNE(regx(opcode), regy(opcode)),
            0xA => Opcode::LDI(addr(opcode)),
            0xB => Opcode::JP(addr(opcode) + (self.get_r(REG_V0) as u16)),
            0xC => Opcode::RND(regx(opcode), byte(opcode)),
            0xD => Opcode::DRW(regx(opcode), regy(opcode), nibble(opcode)),
            0xE => match byte(opcode) {
                0x9E => Opcode::SKP(regx(opcode)),
                0xA1 => Opcode::SKNP(regx(opcode)),
                _ => unimplemented!(),
            },
            0xF => match byte(opcode) {
                0x07 => Opcode::LD(regx(opcode), REG_DT),
                0x0A => unimplemented!(), // TODO implement wait for key press
                0x15 => Opcode::LD(REG_DT, regx(opcode)),
                0x18 => Opcode::LD(REG_ST, regx(opcode)),
                0x1E => Opcode::ADDI(regx(opcode)),
                0x29 => unimplemented!(), // TODO sprite location
                0x33 => unimplemented!(), // TODO BCD representation
                0x55 => unimplemented!(), // TODO store registers
                0x64 => unimplemented!(), // TODO load registers
                _    => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    fn execute(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::CLS => unimplemented!(),
            Opcode::RET => {
                match self.stack.pop() {
                    Some(addr) => self.pc = addr,
                    None => panic!("Attempt to return with empty stack"),
                }
            },
            Opcode::SYS(addr) => unimplemented!(), // See http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#0nnn
            Opcode::JP(addr) => self.pc = addr as usize,
            Opcode::CALL(addr) => {
                self.stack.push(self.pc);
                self.pc = addr as usize;
            },
            Opcode::SE(x, y) => {
                if x == y {
                    self.pc += 2;
                }
            },
            Opcode::SNE(x, y) => {
                if x != y {
                    self.pc += 2;
                }
            },
            Opcode::LD(rx, byte) => self.set_r(rx, byte),
            Opcode::ADD(rx, byte) => {
                // If the result is greater than 255 set the carry flag in REG_VF
                let res = self.get_r(rx) + byte;
                let vf = if res > 255 { 1 } else { 0 };
                self.set_r(REG_VF, vf);
                self.set_r(rx, res as u8);
            },
            Opcode::OR(rx, ry)  => {
                let res = self.get_r(rx) | self.get_r(ry);
                self.set_r(rx, res);
            },
            Opcode::AND(rx, ry) => {
                let res = self.get_r(rx) & self.get_r(ry);
                self.set_r(rx, res);
            },
            Opcode::XOR(rx, ry) => 
            {
                let res = self.get_r(rx) ^ self.get_r(ry);
                self.set_r(rx, res);
            },
            Opcode::SUB(rx, ry) => {
                let vf = if self.get_r(rx) > self.get_r(ry) { 1 } else { 0 };
                let res = self.get_r(rx) - self.get_r(ry);
                self.set_r(REG_VF, vf);
                self.set_r(rx, res);
            },
            Opcode::SHR(rx) => {
                // If least-significant bit is 1, set carry flag
                let vf = if (self.get_r(rx) & 0x1) == 0x1 { 1 } else { 0 };
                let res = self.get_r(rx) >> 1;
                self.set_r(REG_VF, vf);
                self.set_r(rx, res);
            },
            Opcode::SUBN(rx, ry) => {
                let vf = if self.get_r(rx) < self.get_r(ry) { 1 } else { 0 };
                let res = self.get_r(ry) - self.get_r(rx);
                self.set_r(REG_VF, vf);
                self.set_r(rx, res);
            },
            Opcode::SHL(rx, ry) => {
                // If most-significant bit is 1, set carry flag
                let vf = if (self.get_r(rx) & 0xC0) == 0xC0 { 1 } else { 0 };
                let res = self.get_r(rx) << 1;
                self.set_r(REG_VF, vf);
                self.set_r(rx, res);
            },
            Opcode::LDI(addr) => self.idx = addr as u16,
            Opcode::RND(rx, byte) => self.set_r(rx, rand::random::<u8>() & byte),
            Opcode::DRW(rx, ry, nibble) => unimplemented!(), // TODO draw a sprite
            Opcode::SKP(rx) => {
                if self.sys.key_pressed(rx) {
                    self.pc += 2;
                }
            },
            Opcode::SKNP(rx) => {
                if !self.sys.key_pressed(rx) {
                    self.pc += 2;
                }
            },
            Opcode::ADDI(rx) => {
                let res = self.get_r(rx) as u16 + self.idx;
                self.idx = res;
            },
        }
    }

    fn update(&mut self) {
        // Increment pc
        // Decrement timers
    }

    fn get_r(&self, r: u8) -> u8 {
        self.regs[r as usize]
    }

    fn set_r(&mut self, r: u8, n: u8) {
        self.regs[r as usize] = n;
    }
}

// Opcode decode helper functions

fn addr(opcode: u16) -> u16 {
    opcode & 0x0FFF
}

fn byte(opcode: u16) -> u8 {
    (opcode & 0x00FF) as u8
}

fn regx(opcode: u16) -> u8 {
    ((opcode & 0x0F00) >> 8) as u8
}

fn regy(opcode: u16) -> u8 {
    ((opcode & 0x00F0) >> 4) as u8
}

fn nibble(opcode: u16) -> u8 {
    (opcode & 0x000F) as u8
}

