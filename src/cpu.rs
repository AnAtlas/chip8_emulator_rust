use rand::prelude::*;
use crate::{DISPLAY_WIDTH, DISPLAY_HEIGHT, MEMORY_SIZE, VRAM};

const REGISTER_AMOUNT: usize = 16;
const STACK_SIZE: usize = 16;
const OPCODE_SIZE: usize = 2;
const FLAG_REGISTER: usize = 15;

type OpCode = u16;
type Address = u16;
type Register = u8;
type RegisterIndex = usize;
type PC = usize;
type SP = usize;
type KK = u8;
type NNN = u16;
type SpriteSize = u8;

//Actually only 12 bits
#[derive(Copy, Clone)]
struct IndexRegister {
    value: usize,
}

impl IndexRegister {
    pub fn from(value: usize) -> Self {
        IndexRegister{value: value & 0xFFF}
    }

    pub fn overflowing_add(self, rhs: usize) -> (Self, bool) {
        let value = self.value + rhs;
        (IndexRegister{value: value & 0xFFF}, value > 0xFFF)
    }
}

pub struct OutputState<'a> {
    pub vram: &'a VRAM,
    pub vram_changed: bool,
    pub beep: bool,
}

pub(crate) struct CPU {
    pc: PC,
    sp: SP,
    dt: Register,
    st: Register,
    i: IndexRegister,
    registers: [Register; REGISTER_AMOUNT],
    stack: [Address; STACK_SIZE],
    memory: [u8; MEMORY_SIZE],
    vram: VRAM,
    vram_changed: bool,
    waiting_for_key_press: Option<usize>,
    keypad: [bool; 16],
}

#[derive(Debug, PartialEq)]
enum PcChange {
    Increment,
    Skip,
    Jump(PC),
}

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidOpcode(OpCode),
    StackOverflow,
    StackUnderflow,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            pc: 0x200,
            sp: 0,
            dt: 0,
            st: 0,
            i: IndexRegister::from(0),
            registers: [0; REGISTER_AMOUNT],
            stack: [0; STACK_SIZE],
            memory: [0; MEMORY_SIZE],
            vram: [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            vram_changed: false,
            waiting_for_key_press: None,
            keypad: [false; 16],
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = 0x200 + i;
            if addr < self.memory.len() {
                self.memory[addr] = byte;
            } else {
                break;
            }
        }
    }

    fn read_opcode(&self) -> OpCode {
        (self.memory[self.pc] as OpCode) << 8 | (self.memory[self.pc + 1] as OpCode)
    }

    pub fn tick(&mut self, keypad: &[bool; 16]) -> OutputState {
        self.keypad = *keypad;
        self.vram_changed = false;

        if self.waiting_for_key_press.is_some() {
            for i in 0..keypad.len() {
                if keypad[i] {
                    self.registers[self.waiting_for_key_press.unwrap()] = i as u8;
                    self.waiting_for_key_press = None;
                }
            }
        } else {
            if self.dt > 0 {
                self.dt -= 1;
            }
            if self.dt > 0 {
                self.dt -= 1;
            }
            let pc_change = self.run_opcode(self.read_opcode());

            match pc_change {
                Ok(PcChange::Increment) => self.pc += OPCODE_SIZE,
                Ok(PcChange::Skip) => self.pc += OPCODE_SIZE * 2,
                Ok(PcChange::Jump(address)) => self.pc = address,
                Err(Error::InvalidOpcode(opcode)) => println!("Invalid Opcode {:?}", opcode),
                Err(error) => println!("ERROR {:?}", error),
            };
        }
        OutputState {
            vram: &self.vram,
            vram_changed: self.vram_changed,
            beep: self.dt > 0,
        }
    }

    fn run_opcode(&mut self, opcode: OpCode) -> Result<PcChange, Error>{
        let c = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let n = (opcode & 0x000F) as u8;
        let nnn = opcode & 0x0FFF;
        let kk = (opcode & 0x00FF) as u8;

        match (c, x, y, n) {
            (0x0, 0x0, 0xE, 0x0) => self.clear_screen(),
            (0x0, 0x0, 0xE, 0xE) => self.return_from_subroutine(),
            // (  0,   _,   _,   _) => self.execute_machine_language_subroutine_at_address(nnn),
            (0x1,   _,   _,   _) => self.jump_to_address_nnn(nnn),
            (0x2,   _,   _,   _) => self.execute_subroutine_at_address_nnn(nnn),
            (0x3,   _,   _,   _) => self.skip_next_op_if_reg_x_equals_kk(x as RegisterIndex, kk),
            (0x4,   _,   _,   _) => self.skip_next_op_if_reg_x_not_equals_kk(x as RegisterIndex, kk),
            (0x5,   _,   _, 0x0) => self.skip_next_op_if_reg_x_equals_reg_y(x as RegisterIndex, y as RegisterIndex),
            (0x6,   _,   _,   _) => self.set_register_x_to_kk(x as RegisterIndex, kk),
            (0x7,   _,   _,   _) => self.add_kk_to_register_x(x as RegisterIndex, kk),
            (0x8,   _,   _, 0x0) => self.set_register_x_to_register_y(x as RegisterIndex, y as RegisterIndex),
            (0x8,   _,   _, 0x1) => self.set_register_x_to_register_x_or_register_y(x as RegisterIndex, y as RegisterIndex),
            (0x8,   _,   _, 0x2) => self.set_register_x_to_register_x_and_register_y(x as RegisterIndex, y as RegisterIndex),
            (0x8,   _,   _, 0x3) => self.set_register_x_to_register_x_xor_register_y(x as RegisterIndex, y as RegisterIndex),
            (0x8,   _,   _, 0x4) => self.set_register_x_to_register_x_add_register_y(x as RegisterIndex, y as RegisterIndex),
            (0x8,   _,   _, 0x5) => self.set_register_x_to_register_x_sub_register_y(x as RegisterIndex, y as RegisterIndex),
            (0x8,   _,   _, 0x6) => self.shift_register_x_right(x as RegisterIndex),
            (0x8,   _,   _, 0x7) => self.set_register_x_to_register_y_sub_register_x(x as RegisterIndex, y as RegisterIndex),
            (0x8,   _,   _, 0xE) => self.shift_register_x_left(x as RegisterIndex),
            (0x9,   _,   _, 0x0) => self.skip_next_op_if_reg_x_not_equals_reg_y(x as RegisterIndex, y as RegisterIndex),
            (0xA,   _,   _,   _) => self.set_register_i_to_nnn(nnn),
            (0xB,   _,   _,   _) => self.jump_to_address_nnn_plus_reg_0(nnn),
            (0xC,   _,   _,   _) => self.set_register_x_to_random_byte_plus_kk(x as RegisterIndex, kk),
            (0xD,   _,   _,   _) => self.display_sprite(x as RegisterIndex, y as RegisterIndex, n),
            (0xE,   _, 0x9, 0xE) => self.skip_next_op_if_reg_x_key_is_pressed(x as RegisterIndex),
            (0xE,   _, 0xA, 0x1) => self.skip_next_op_if_reg_x_key_is_not_pressed(x as RegisterIndex),
            (0xF,   _, 0x0, 0x7) => self.set_register_x_to_timer_register(x as RegisterIndex),
            (0xF,   _, 0x0, 0xA) => self.set_register_x_to_next_pressed_key(x as RegisterIndex),
            (0xF,   _, 0x1, 0x5) => self.set_delay_timer_to_register_x(x as RegisterIndex),
            (0xF,   _, 0x1, 0x8) => self.set_sound_timer_to_register_x(x as RegisterIndex),
            (0xF,   _, 0x1, 0xE) => self.set_register_i_to_register_i_add_register_x(x as RegisterIndex),
            (0xF,   _, 0x2, 0x9) => self.set_register_i_to_address_of_sprite_at_register_x(x as RegisterIndex),
            (0xF,   _, 0x3, 0x3) => self.set_memory_at_i_to_decimal_value_of_register_x(x as RegisterIndex),
            (0xF,   _, 0x5, 0x5) => self.set_memory_at_i_to_registers(x as RegisterIndex),
            (0xF,   _, 0x6, 0x5) => self.set_registers_to_memory_at_i(x as RegisterIndex),
            _ => {
                Err(Error::InvalidOpcode(opcode))
            }
        }
    }

    fn set_registers_to_memory_at_i(&mut self, x: RegisterIndex) -> Result<PcChange, Error> {
        for i in 0..x + 1 {
            self.registers[i] = self.memory[self.i.value + i];
        }
        Ok(PcChange::Increment)
    }

    fn set_memory_at_i_to_registers(&mut self, x: RegisterIndex) -> Result<PcChange, Error> {
        for i in 0..x + 1 {
            self.memory[self.i.value + i] = self.registers[i];
        }
        Ok(PcChange::Increment)
    }

    fn set_memory_at_i_to_decimal_value_of_register_x(&mut self, x: RegisterIndex) -> Result<PcChange, Error> {
        self.memory[self.i.value] = self.registers[x] / 100;
        self.memory[self.i.value + 1] = (self.registers[x] % 100) / 10;
        self.memory[self.i.value + 2] = self.registers[x] %10;
        Ok(PcChange::Increment)
    }

    fn set_register_i_to_address_of_sprite_at_register_x(&mut self, x: RegisterIndex) -> Result<PcChange, Error> {
        self.i = IndexRegister::from((self.registers[x] as usize) * 5);
        Ok(PcChange::Increment)
    }

    fn set_register_i_to_register_i_add_register_x(&mut self, x: RegisterIndex) -> Result<PcChange, Error> {
        let (value, overflow) = self.i.overflowing_add(self.registers[x] as usize);
        self.i = value;
        if overflow {
            self.registers[FLAG_REGISTER] = 1;
        }
        else {
            self.registers[FLAG_REGISTER] = 0;
        }
        Ok(PcChange::Increment)
    }

    fn set_sound_timer_to_register_x(&mut self, x: RegisterIndex) -> Result<PcChange, Error> {
        self.st = self.registers[x];
        Ok(PcChange::Increment)
    }

    fn set_delay_timer_to_register_x(&mut self, x: RegisterIndex) -> Result<PcChange, Error> {
        self.dt = self.registers[x];
        Ok(PcChange::Increment)
    }

    fn set_register_x_to_next_pressed_key(&mut self, x: RegisterIndex) -> Result<PcChange, Error> {
        self.waiting_for_key_press = Some(x as usize);
        Ok(PcChange::Increment)
    }

    fn set_register_x_to_timer_register(&mut self, x: RegisterIndex) -> Result<PcChange, Error> {
        self.registers[x] = self.dt;
        Ok(PcChange::Increment)
    }

    fn skip_next_op_if_reg_x_key_is_not_pressed(&self, x: RegisterIndex) -> Result<PcChange, Error> {
        if self.keypad[self.registers[x] as usize] {
            Ok(PcChange::Increment)
        } else {
            Ok(PcChange::Skip)
        }
    }

    fn skip_next_op_if_reg_x_key_is_pressed(&self, x: RegisterIndex) -> Result<PcChange, Error> {
        if self.keypad[self.registers[x] as usize] {
            Ok(PcChange::Skip)
        } else {
            Ok(PcChange::Increment)
        }
    }

    fn display_sprite(&mut self, x: RegisterIndex, y: RegisterIndex, n: SpriteSize) -> Result<PcChange, Error> {
        self.registers[FLAG_REGISTER] = 0x00;
        for byte in 0..n as usize{
            let y = (self.registers[y] as usize + byte) % DISPLAY_HEIGHT;
            for bit in 0..8 {
                let x = (self.registers[x] as usize + bit) % DISPLAY_WIDTH;
                let color = (self.memory[self.i.value + byte] >> (7 - bit)) & 1;
                self.registers[FLAG_REGISTER] |= color & self.vram[y][x];
                self.vram[y][x] ^= color;
            }
        }
        self.vram_changed = true;
        Ok(PcChange::Increment)
    }

    fn set_register_x_to_random_byte_plus_kk(&mut self, x: RegisterIndex, kk: KK) -> Result<PcChange, Error> {
        self.registers[x] = rand::thread_rng().gen::<u8>() & kk;
        Ok(PcChange::Increment)
    }

    fn jump_to_address_nnn_plus_reg_0(&self, nnn: NNN) -> Result<PcChange, Error> {
        Ok(PcChange::Jump((nnn + (self.registers[0] as NNN)) as PC))
    }

    fn set_register_i_to_nnn(&mut self, nnn: NNN) -> Result<PcChange, Error> {
        self.i.value = nnn as usize;
        Ok(PcChange::Increment)
    }

    fn skip_next_op_if_reg_x_not_equals_reg_y(&self, x: RegisterIndex, y: RegisterIndex) -> Result<PcChange, Error> {
        if self.registers[x] != self.registers[y]  { Ok(PcChange::Skip) } else {Ok(PcChange::Increment)}
    }

    fn shift_register_x_left(&mut self, x:RegisterIndex) -> Result<PcChange, Error> {
        if self.registers[x] & 0x80 != 0 {
            self.registers[FLAG_REGISTER] = 1;
        } else {
            self.registers[FLAG_REGISTER] = 0;
        }
        self.registers[x] <<= 1;
        Ok(PcChange::Increment)
    }

    //Set VF NOT borrow
    fn set_register_x_to_register_y_sub_register_x(&mut self, x: RegisterIndex, y: RegisterIndex) -> Result<PcChange, Error> {
        if self.registers[y] > self.registers[x] {
            self.registers[FLAG_REGISTER] = 1;
        }
        else {
            self.registers[FLAG_REGISTER] = 0;
        }
        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
        Ok(PcChange::Increment)
    }

    fn shift_register_x_right(&mut self, x:RegisterIndex) -> Result<PcChange, Error> {
        if self.registers[x] & 0x1 != 0 {
            self.registers[FLAG_REGISTER] = 1;
        } else {
            self.registers[FLAG_REGISTER] = 0;
        }
        self.registers[x] >>= 1;
        Ok(PcChange::Increment)
    }

    fn set_register_x_to_register_x_sub_register_y(&mut self, x: RegisterIndex, y: RegisterIndex) -> Result<PcChange, Error> {
        if self.registers[x] > self.registers[y] {
            self.registers[FLAG_REGISTER] = 1;
        }
        else {
            self.registers[FLAG_REGISTER] = 0;
        }
        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
        Ok(PcChange::Increment)
    }

    fn set_register_x_to_register_x_add_register_y(&mut self, x: RegisterIndex, y: RegisterIndex) -> Result<PcChange, Error> {
        let (val, overflow) = self.registers[x].overflowing_add(self.registers[y]);
        self.registers[x] = val;
        if overflow {
            self.registers[FLAG_REGISTER] = 1;
        }
        else {
            self.registers[FLAG_REGISTER] = 0;
        }
        Ok(PcChange::Increment)
    }

    fn set_register_x_to_register_x_xor_register_y(&mut self, x: RegisterIndex, y: RegisterIndex) -> Result<PcChange, Error> {
        self.registers[x] ^= self.registers[y];
        Ok(PcChange::Increment)
    }

    fn set_register_x_to_register_x_and_register_y(&mut self, x: RegisterIndex, y: RegisterIndex) -> Result<PcChange, Error> {
        self.registers[x] &= self.registers[y];
        Ok(PcChange::Increment)
    }

    fn set_register_x_to_register_x_or_register_y(&mut self, x: RegisterIndex, y: RegisterIndex) -> Result<PcChange, Error> {
        self.registers[x] |= self.registers[y];
        Ok(PcChange::Increment)
    }

    fn set_register_x_to_register_y(&mut self, x: RegisterIndex, y: RegisterIndex) -> Result<PcChange, Error> {
        self.registers[x] = self.registers[y];
        Ok(PcChange::Increment)
    }

    fn add_kk_to_register_x(&mut self, x: RegisterIndex, kk: KK) -> Result<PcChange,Error> {
        let (value, _overflow) = self.registers[x].overflowing_add(kk);
        self.registers[x] = value;
        Ok(PcChange::Increment)
    }

    fn set_register_x_to_kk(&mut self, x: RegisterIndex, kk: KK) -> Result<PcChange, Error> {
        self.registers[x] = kk;
        Ok(PcChange::Increment)
    }

    fn clear_screen(&mut self) -> Result<PcChange, Error> {
        for row in self.vram.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = 0;
            }
        }
        self.vram_changed = true;
        Ok(PcChange::Increment)
    }

    fn return_from_subroutine(&mut self) -> Result<PcChange, Error> {
        if self.sp == 0 {
            return Err(Error::StackUnderflow);
        }
        self.sp -= 1;
        Ok(PcChange::Jump(self.stack[self.sp] as PC))
    }

    fn jump_to_address_nnn(&mut self, address: NNN) -> Result<PcChange, Error> {
        Ok(PcChange::Jump(address as PC))
    }

    fn execute_subroutine_at_address_nnn(&mut self, address: NNN) -> Result<PcChange, Error> {
        if self.sp > self.stack.len() {
            return Err(Error::StackOverflow);
        }
        self.stack[self.sp] = (self.pc + OPCODE_SIZE) as Address;
        self.sp += 1;
        Ok(PcChange::Jump(address as PC))
    }

    fn skip_next_op_if_reg_x_equals_kk(&self, x: RegisterIndex, kk: Register) -> Result<PcChange, Error> {
        match self.registers[x] == kk {
            true => Ok(PcChange::Skip),
            false => Ok(PcChange::Increment),
        }
    }

    fn skip_next_op_if_reg_x_not_equals_kk(&self, x: RegisterIndex, kk: Register) -> Result<PcChange, Error> {
        match self.registers[x] == kk {
            true => Ok(PcChange::Increment),
            false => Ok(PcChange::Skip),
        }
    }

    fn skip_next_op_if_reg_x_equals_reg_y(&self, x: RegisterIndex, y: RegisterIndex) -> Result<PcChange, Error> {
        if self.registers[x] == self.registers[y]  { Ok(PcChange::Skip) } else {Ok(PcChange::Increment)}
    }
}

#[cfg(test)]
#[path = "./cpu_test.rs"]
mod cpu_test;