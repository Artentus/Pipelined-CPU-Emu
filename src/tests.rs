use crate::{cpu, Byte, Memory, Vga, Word};

#[test]
fn test_inc_a_from_zero() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0xA0);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::ZERO;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.a = Byte::new(1);

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_inc_a_from_max() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0xA0);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::MAX;
    initial.sign = true;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.a = Byte::ZERO;
    expected.sign = false;
    expected.zero = true;
    expected.carry = true;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_dec_a_from_zero() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0xC0);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::ZERO;
    initial.zero = true;
    initial.sign = false;
    initial.carry = false;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.a = Byte::MAX;
    expected.zero = false;
    expected.sign = true;
    expected.carry = false;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_dec_a_from_one() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0xC0);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::new(1);
    initial.zero = false;
    initial.carry = false;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.a = Byte::ZERO;
    expected.zero = true;
    expected.carry = true;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_inc_si_from_zero() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x35);

    let mut initial: cpu::CpuState = Default::default();
    initial.source_index = Word::ZERO;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.source_index = Word::new(1);

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_inc_si_from_max() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x35);

    let mut initial: cpu::CpuState = Default::default();
    initial.source_index = Word::MAX;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.source_index = Word::ZERO;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_dec_si_from_zero() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x32);

    let mut initial: cpu::CpuState = Default::default();
    initial.source_index = Word::ZERO;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.source_index = Word::MAX;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_dec_si_from_one() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x32);

    let mut initial: cpu::CpuState = Default::default();
    initial.source_index = Word::new(1);

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.source_index = Word::ZERO;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_mov_a_const_zero() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x01);
    mem.write(&mut vga, 0x0001, 0x00);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::MAX;
    initial.zero = false;
    initial.sign = true;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0004);
    expected.a = Byte::ZERO;
    expected.zero = false;
    expected.sign = true;

    cpu::test_code(&mut mem, 4, initial, expected);
}

#[test]
fn test_mov_a_const_max() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x01);
    mem.write(&mut vga, 0x0001, 0xFF);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::ZERO;
    initial.zero = true;
    initial.sign = false;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0004);
    expected.a = Byte::MAX;
    expected.zero = true;
    expected.sign = false;

    cpu::test_code(&mut mem, 4, initial, expected);
}

#[test]
fn test_mov_a_b() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x07);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::ZERO;
    initial.b = Byte::MAX;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.a = Byte::MAX;
    expected.b = Byte::MAX;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_mov_si_di() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x2C);

    let mut initial: cpu::CpuState = Default::default();
    initial.source_index = Word::ZERO;
    initial.destination_index = Word::MAX;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.source_index = Word::MAX;
    expected.destination_index = Word::MAX;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_transfer_gpr_to_spr() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x13);
    mem.write(&mut vga, 0x0001, 0x18);
    mem.write(&mut vga, 0x0002, 0x27);

    let mut initial: cpu::CpuState = Default::default();
    initial.transfer = Word::ZERO;
    initial.source_index = Word::ZERO;
    initial.a = Byte::new(0x01);
    initial.b = Byte::new(0x02);

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0005);
    expected.transfer = Word::new(0x0201);
    expected.source_index = Word::new(0x0201);
    expected.a = Byte::new(0x01);
    expected.b = Byte::new(0x02);

    cpu::test_code(&mut mem, 5, initial, expected);
}

#[test]
fn test_transfer_spr_to_gpr() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x28);
    mem.write(&mut vga, 0x0001, 0x1B);
    mem.write(&mut vga, 0x0002, 0x20);

    let mut initial: cpu::CpuState = Default::default();
    initial.transfer = Word::ZERO;
    initial.source_index = Word::new(0x0201);
    initial.a = Byte::ZERO;
    initial.b = Byte::ZERO;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0005);
    expected.transfer = Word::new(0x0201);
    expected.source_index = Word::new(0x0201);
    expected.a = Byte::new(0x01);
    expected.b = Byte::new(0x02);

    cpu::test_code(&mut mem, 5, initial, expected);
}

#[test]
fn test_stack() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x72);
    mem.write(&mut vga, 0x0001, 0x73);
    mem.write(&mut vga, 0x0002, 0x7A);

    let mut initial: cpu::CpuState = Default::default();
    initial.stack_pointer = Word::ZERO;
    initial.a = Byte::new(1);
    initial.b = Byte::new(2);
    initial.c = Byte::ZERO;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0005);
    expected.stack_pointer = Word::new(0xFFFF);
    expected.a = Byte::new(1);
    expected.b = Byte::new(2);
    expected.c = Byte::new(2);

    cpu::test_code(&mut mem, 8, initial, expected);
}

#[test]
fn test_add_a_b() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x88);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::new(1);
    initial.b = Byte::new(2);

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.a = Byte::new(3);
    expected.b = Byte::new(2);

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_add_a_b_overflow() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x88);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::MAX;
    initial.b = Byte::new(1);
    initial.carry = false;
    initial.sign = true;
    initial.zero = false;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.a = Byte::ZERO;
    expected.b = Byte::new(1);
    expected.carry = true;
    expected.sign = false;
    expected.zero = true;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_addc_a_b() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0x94);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::new(1);
    initial.b = Byte::new(1);
    initial.carry = true;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.a = Byte::new(3);
    expected.b = Byte::new(1);
    expected.carry = false;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_sub_a_b_pos() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0xA8);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::new(2);
    initial.b = Byte::new(1);
    initial.sign = false;
    initial.zero = false;
    initial.carry = false;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.a = Byte::new(1);
    expected.b = Byte::new(1);
    expected.sign = false;
    expected.zero = false;
    expected.carry = true;

    cpu::test_code(&mut mem, 3, initial, expected);
}

#[test]
fn test_sub_a_b_neg() {
    let mut mem = Memory::new();
    let mut vga = Vga::new();
    mem.write(&mut vga, 0x0000, 0xA8);

    let mut initial: cpu::CpuState = Default::default();
    initial.a = Byte::new(1);
    initial.b = Byte::new(2);
    initial.sign = false;
    initial.zero = false;
    initial.carry = false;

    let mut expected: cpu::CpuState = Default::default();
    expected.program_counter = Word::new(0x0003);
    expected.a = Byte::new(255);
    expected.b = Byte::new(2);
    expected.sign = true;
    expected.zero = false;
    expected.carry = false;

    cpu::test_code(&mut mem, 3, initial, expected);
}
