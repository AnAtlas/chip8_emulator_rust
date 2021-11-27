use crate::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::cpu::{Error, FLAG_REGISTER, PcChange};
use super::CPU;

#[test]
fn test_initial_state() {
    let cpu = CPU::new();
    assert_eq!(cpu.pc, 0x200);
    assert_eq!(cpu.sp, 0);
    assert_eq!(cpu.stack, [0; 16]);
}

#[test]
fn test_clear_screen() {
    let mut cpu = CPU::new();
    cpu.vram = [[128; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
    let change = cpu.run_opcode(0x00E0);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.vram, [[0; DISPLAY_WIDTH]; DISPLAY_HEIGHT]);
}

#[test]
fn test_return_from_subroutine() {
    let mut cpu = CPU::new();
    let change = cpu.run_opcode(0x00EE);
    assert!(change.is_err());
    assert_eq!(change.unwrap_err(), Error::StackUnderflow);
    cpu.stack[0] = 0x100;
    cpu.sp = 1;
    let change = cpu.run_opcode(0x00EE);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Jump(0x100));
    assert_eq!(cpu.sp, 0);
}

#[test]
fn test_jump_to_address_nnn() {
    let mut cpu = CPU::new();
    let change = cpu.run_opcode(0x1234);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Jump(0x234));
}

#[test]
fn test_execute_subroutine_at_address_nnn() {
    let mut cpu = CPU::new();
    let change = cpu.run_opcode(0x2234);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Jump(0x234));
    assert_eq!(cpu.sp, 1);
    assert_eq!(cpu.stack[0], 0x202);
}

#[test]
fn test_skip_next_op_if_reg_x_equals_kk() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x34;
    let change = cpu.run_opcode(0x3134);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    let change = cpu.run_opcode(0x3234);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Skip);
}

#[test]
fn test_skip_next_op_if_reg_x_not_equals_kk() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x34;
    let change = cpu.run_opcode(0x4134);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Skip);
    let change = cpu.run_opcode(0x4234);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
}

#[test]
fn test_skip_next_op_if_reg_x_equals_reg_y() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x34;
    cpu.registers[3] = 0x34;
    let change = cpu.run_opcode(0x5230);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Skip);
    let change = cpu.run_opcode(0x5130);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
}

#[test]
fn test_set_register_x_to_kk() {
    let mut cpu = CPU::new();
    let change = cpu.run_opcode(0x6135);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[1], 0x35);
}

#[test]
fn test_add_kk_to_register_x() {
    let mut cpu = CPU::new();
    let change = cpu.run_opcode(0x7135);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[1], 0x35);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x00);
    let change = cpu.run_opcode(0x71FF);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[1], 0x34);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x00);
}

#[test]
fn test_set_register_x_to_register_y() {
    let mut cpu = CPU::new();
    cpu.registers[3] = 0x11;
    let change = cpu.run_opcode(0x8030);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[0], 0x11);
}

#[test]
fn test_set_register_x_to_register_x_or_register_y() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x88;
    cpu.registers[3] = 0x11;
    let change = cpu.run_opcode(0x8231);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[2], 0x99);
}

#[test]
fn test_set_register_x_to_register_x_and_register_y() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x8F;
    cpu.registers[3] = 0x11;
    let change = cpu.run_opcode(0x8232);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[2], 0x01);
}

#[test]
fn test_set_register_x_to_register_x_xor_register_y() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x8F;
    cpu.registers[3] = 0x11;
    let change = cpu.run_opcode(0x8233);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[2], 0x9E);
}

#[test]
fn test_set_register_x_to_register_x_add_register_y() {
    let mut cpu = CPU::new();
    cpu.registers[5] = 0xFE;
    cpu.registers[6] = 0x01;
    let change = cpu.run_opcode(0x8564);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[5], 0xFF);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x00);
    let change = cpu.run_opcode(0x8564);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[5], 0x00);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x01);
}

#[test]
fn test_set_register_x_to_register_x_sub_register_y() {
    let mut cpu = CPU::new();
    cpu.registers[5] = 0x02;
    cpu.registers[6] = 0x01;
    let change = cpu.run_opcode(0x8565);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[5], 0x01);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x01);
    let change = cpu.run_opcode(0x8565);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[5], 0x00);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x00);
    let change = cpu.run_opcode(0x8565);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[5], 0xFF);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x00);
}

#[test]
fn test_shift_register_x_right() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x81;
    let change = cpu.run_opcode(0x8206);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[2], 0x40);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x01);
    let change = cpu.run_opcode(0x8206);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[2], 0x20);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x00);
}

#[test]
fn test_set_register_x_to_register_y_sub_register_x() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x05;
    cpu.registers[3] = 0x07;
    let change = cpu.run_opcode(0x8237);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[2], 0x02);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x01);
    cpu.registers[2] = 0x05;
    cpu.registers[3] = 0x03;
    let change = cpu.run_opcode(0x8237);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[2], 0xFE);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x00);
}

#[test]
fn test_shift_register_x_left() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x81;
    let change = cpu.run_opcode(0x820E);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[2], 0x02);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x01);
    let change = cpu.run_opcode(0x820E);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.registers[2], 0x04);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x00);
}

#[test]
fn test_skip_next_op_if_reg_x_not_equals_reg_y() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x34;
    cpu.registers[3] = 0x34;
    let change = cpu.run_opcode(0x9230);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    let change = cpu.run_opcode(0x9130);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Skip);
}

#[test]
fn test_set_register_i_to_nnn() {
    let mut cpu = CPU::new();
    let change = cpu.run_opcode(0xA230);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.i.value, 0x230);
    let change = cpu.run_opcode(0xA130);
    assert_eq!(cpu.i.value, 0x130);
}

#[test]
fn test_jump_to_address_nnn_plus_reg_0() {
    let mut cpu = CPU::new();
    cpu.registers[0] = 0x11;
    let change = cpu.run_opcode(0xB230);
    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Jump(0x241));
}

#[test]
fn test_display_sprite() {
    let mut cpu = CPU::new();
    cpu.i.value = 0;
    cpu.memory[0] = 0b11111111;
    cpu.memory[1] = 0b00000000;
    cpu.vram[0][0] = 1;
    cpu.vram[0][1] = 0;
    cpu.vram[1][0] = 1;
    cpu.vram[1][1] = 0;
    cpu.registers[0] = 0;
    let change = cpu.run_opcode(0xD002);

    assert!(change.is_ok());
    assert_eq!(change.unwrap(), PcChange::Increment);
    assert_eq!(cpu.vram[0][0], 0);
    assert_eq!(cpu.vram[0][1], 1);
    assert_eq!(cpu.vram[1][0], 1);
    assert_eq!(cpu.vram[1][1], 0);
    assert_eq!(cpu.registers[FLAG_REGISTER], 1);
    assert!(cpu.vram_changed);
}

#[test]
fn test_set_register_x_to_timer_register() {
    let mut cpu = CPU::new();
    cpu.dt = 0x12;
    let change = cpu.run_opcode(0xF207);
    assert!(change.is_ok());
    assert_eq!(cpu.registers[2], 0x12);
    assert_eq!(change.unwrap(), PcChange::Increment);
}

#[test]
fn test_set_delay_timer_to_register_x() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x12;
    let change = cpu.run_opcode(0xF215);
    assert!(change.is_ok());
    assert_eq!(cpu.dt, 0x12);
    assert_eq!(change.unwrap(), PcChange::Increment);
}

#[test]
fn test_set_sound_timer_to_register_x() {
    let mut cpu = CPU::new();
    cpu.registers[2] = 0x12;
    let change = cpu.run_opcode(0xF218);
    assert!(change.is_ok());
    assert_eq!(cpu.st, 0x12);
    assert_eq!(change.unwrap(), PcChange::Increment);
}

#[test]
fn test_set_register_i_to_register_i_add_register_x() {
    let mut cpu = CPU::new();
    cpu.i.value = 0xFFA;
    cpu.registers[2] = 0x05;
    let change = cpu.run_opcode(0xF21E);
    assert!(change.is_ok());
    assert_eq!(cpu.i.value, 0xFFF);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x00);
    assert_eq!(change.unwrap(), PcChange::Increment);
    let change = cpu.run_opcode(0xF21E);
    assert!(change.is_ok());
    assert_eq!(cpu.i.value, 0x004);
    assert_eq!(cpu.registers[FLAG_REGISTER], 0x01);
    assert_eq!(change.unwrap(), PcChange::Increment);
}

#[test]
fn test_set_register_i_to_address_of_sprite_at_register_x() {
    let mut cpu = CPU::new();
    cpu.registers[5] = 0x09;
    let change = cpu.run_opcode(0xF529);
    assert!(change.is_ok());
    assert_eq!(cpu.i.value, 0x2D);
    assert_eq!(change.unwrap(), PcChange::Increment);
}

#[test]
fn test_set_memory_at_i_to_decimal_value_of_register_x() {
    let mut cpu = CPU::new();
    cpu.registers[5] = 123;
    cpu.i.value = 0x500;
    let change = cpu.run_opcode(0xF533);
    assert!(change.is_ok());
    assert_eq!(cpu.memory[0x500], 1);
    assert_eq!(cpu.memory[0x501], 2);
    assert_eq!(cpu.memory[0x502], 3);
    assert_eq!(change.unwrap(), PcChange::Increment);
}

#[test]
fn test_set_memory_at_i_to_registers() {
    let mut cpu = CPU::new();
    for i in 0..15 {
        cpu.registers[i] = i as u8;
    }
    cpu.i.value = 0x500;
    let change = cpu.run_opcode(0xFF55);
    assert!(change.is_ok());
    for i in 0..15 {
        assert_eq!(cpu.memory[0x500 + i] as usize, i);
    }
    assert_eq!(change.unwrap(), PcChange::Increment);
}

#[test]
fn test_set_registers_to_memory_at_i() {
    let mut cpu = CPU::new();
    for i in 0..15 {
        cpu.memory[0x500 + i] = i as u8;
    }
    cpu.i.value = 0x500;
    let change = cpu.run_opcode(0xFF65);
    assert!(change.is_ok());
    for i in 0..15 {
        assert_eq!(cpu.registers[i] as usize, i);
    }
    assert_eq!(change.unwrap(), PcChange::Increment);
}
