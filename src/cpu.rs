use crate::{Byte, Lcd, Memory, Uart, Word};

const SPR_COUNT: usize = 5;
const SPR_PROGRAM_COUNTER: usize = 0;
const SPR_RETURN_ADDRESS: usize = 1;
const SPR_STACK_POINTER: usize = 2;
const SPR_SOURCE_INDEX: usize = 3;
const SPR_DESTINATION_INDEX: usize = 4;

const GPR_COUNT: usize = 4;
const GPR_A: usize = 0;
const GPR_B: usize = 1;
const GPR_C: usize = 2;
const GPR_D: usize = 3;

const PROGRAM_COUNTER_INIT: Word = Word::new(0x0000);
const STACK_POINTER_INIT: Word = Word::new(0x0000);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoadSource {
    Constant,
    Gpr(usize),
    TransferLow,
    TransferHigh,
    SprIndirect(usize),
    TransferIndirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreTarget {
    Gpr(usize),
    TransferLow,
    TransferHigh,
    SprIndirect(usize),
    TransferIndirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoadWordSource {
    Spr(usize),
    Transfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreWordTarget {
    Spr(usize),
    Transfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CountTarget {
    Gpr(usize),
    Spr(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JumpTarget {
    Transfer,
    Spr(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AluSource {
    Gpr(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StackTarget {
    Gpr(usize),
    TransferLow,
    TransferHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IORegister {
    LcdCmd,
    LcdData,
    UartData,
    UartCtrl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Nop,
    Mov(LoadSource, StoreTarget),
    MovWord(LoadWordSource, StoreWordTarget),
    Inc(CountTarget),
    Dec(CountTarget),
    Prebranch,
    Jmp(JumpTarget),
    Jo(JumpTarget),   // Overflow
    Jno(JumpTarget),  // !Overflow
    Js(JumpTarget),   // Sign
    Jns(JumpTarget),  // !Sign
    Jz(JumpTarget),   // Zero
    Jnz(JumpTarget),  // !Zero
    Jc(JumpTarget),   // Carry
    Jnc(JumpTarget),  // !Carry
    Jna(JumpTarget),  // Carry OR Zero
    Ja(JumpTarget),   // !Carry AND !Zero
    Jl(JumpTarget),   // Sign != Overflow
    Jge(JumpTarget),  // Sign == Overflow
    Jle(JumpTarget),  // Zero OR (Sign != Overflow)
    Jg(JumpTarget),   // !Zero AND (Sign == Overflow)
    Jlc(JumpTarget),  // Logical Carry
    Jnlc(JumpTarget), // !Logical Carry
    Clc,
    Shl(AluSource),
    Shr(AluSource),
    Add(AluSource, AluSource),
    Addc(AluSource, AluSource),
    Incc(AluSource),
    Sub(AluSource, AluSource),
    Subb(AluSource, AluSource),
    And(AluSource, AluSource),
    Or(AluSource, AluSource),
    Xor(AluSource, AluSource),
    Not(AluSource),
    Cmp(AluSource, AluSource),
    Test(AluSource),
    Push(StackTarget),
    Pop(StackTarget),
    Call(JumpTarget),
    Ret,
    Out(AluSource, IORegister),
    In(IORegister, AluSource),
}

pub struct Cpu {
    // registers
    transfer: Word,
    spr: [Word; SPR_COUNT],
    constant: Byte,
    gpr: [Byte; GPR_COUNT],

    // pipeline instruction registers
    stage0_instruction: Instruction,
    stage1_instruction: Instruction,
    stage2_instruction: Instruction,

    // ALU input latches
    lhs_latch: Byte,
    rhs_latch: Byte,

    // status flags
    overflow: bool,
    zero: bool,
    carry: bool,
    logical_carry: bool,
    sign: bool,
}
impl Cpu {
    #[inline]
    pub const fn new() -> Self {
        Self {
            transfer: Word::ZERO,
            spr: [Word::ZERO; SPR_COUNT],
            constant: Byte::ZERO,
            gpr: [Byte::ZERO; GPR_COUNT],

            stage0_instruction: Instruction::Nop,
            stage1_instruction: Instruction::Nop,
            stage2_instruction: Instruction::Nop,

            lhs_latch: Byte::ZERO,
            rhs_latch: Byte::ZERO,

            overflow: false,
            zero: false,
            carry: false,
            logical_carry: false,
            sign: false,
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.spr[SPR_PROGRAM_COUNTER] = PROGRAM_COUNTER_INIT;
        self.spr[SPR_STACK_POINTER] = STACK_POINTER_INIT;
        self.stage0_instruction = Instruction::Nop;
        self.stage1_instruction = Instruction::Nop;
        self.stage2_instruction = Instruction::Nop;
    }

    // Returns true if the load operation was a memory read
    fn load(&self, memory: &Memory, source: LoadSource) -> (bool, Byte) {
        match source {
            LoadSource::Constant => (false, self.constant),
            LoadSource::Gpr(index) => (false, self.gpr[index]),
            LoadSource::TransferLow => (false, self.transfer.low()),
            LoadSource::TransferHigh => (false, self.transfer.high()),
            LoadSource::SprIndirect(index) => {
                let addr = self.spr[index];
                let value = memory.read(addr.into());
                (true, value.into())
            }
            LoadSource::TransferIndirect => {
                let addr = self.transfer;
                let value = memory.read(addr.into());
                (true, value.into())
            }
        }
    }

    // Returns true if the store operation was a memory write
    fn store(&mut self, memory: &mut Memory, target: StoreTarget, value: Byte) -> bool {
        match target {
            StoreTarget::Gpr(index) => {
                self.gpr[index] = value;
                false
            }
            StoreTarget::TransferLow => {
                self.transfer.set_low(value);
                false
            }
            StoreTarget::TransferHigh => {
                self.transfer.set_high(value);
                false
            }
            StoreTarget::SprIndirect(index) => {
                let addr = self.spr[index];
                memory.write(addr.into(), value.into());
                true
            }
            StoreTarget::TransferIndirect => {
                let addr = self.transfer;
                memory.write(addr.into(), value.into());
                true
            }
        }
    }

    fn load_word(&self, source: LoadWordSource) -> Word {
        match source {
            LoadWordSource::Spr(index) => self.spr[index],
            LoadWordSource::Transfer => self.transfer,
        }
    }

    fn store_word(&mut self, target: StoreWordTarget, value: Word) {
        match target {
            StoreWordTarget::Spr(index) => self.spr[index] = value,
            StoreWordTarget::Transfer => self.transfer = value,
        }
    }

    fn jump_to(&mut self, target: JumpTarget) {
        match target {
            JumpTarget::Transfer => self.spr[SPR_PROGRAM_COUNTER] = self.transfer,
            JumpTarget::Spr(index) => self.spr[SPR_PROGRAM_COUNTER] = self.spr[index],
        }
    }

    fn alu_stage2(&mut self, target: Option<AluSource>, carry_override: Option<u16>) {
        let lhs: u8 = self.lhs_latch.into();
        let rhs: u8 = self.rhs_latch.into();

        let lhs_long = lhs as u16;
        let rhs_long = rhs as u16;

        let carry_add: u16 = match carry_override {
            Some(v) => v,
            None => {
                if self.carry {
                    0x0001
                } else {
                    0x0000
                }
            }
        };

        let result_long = lhs_long + rhs_long + carry_add;
        let result = result_long.to_le_bytes()[0];

        self.carry = result_long > 0x00FF;
        self.zero = result == 0x00;
        self.sign = (result & 0x80) != 0;

        let lhs_sign = (lhs & 0x80) != 0;
        let rhs_sign = (rhs & 0x80) != 0;
        self.overflow = (lhs_sign == rhs_sign) && (lhs_sign != self.sign);

        if let Some(AluSource::Gpr(index)) = target {
            self.gpr[index] = result.into();
        }
    }

    fn shift_op_stage1(&mut self, target: AluSource, op: fn(Byte) -> Byte) {
        let v = match target {
            AluSource::Gpr(index) => self.gpr[index],
        };

        self.rhs_latch = Byte::ZERO;
        self.lhs_latch = op(v);
    }

    fn logic_op_stage1(
        &mut self,
        source: AluSource,
        target: AluSource,
        op: fn(Byte, Byte) -> Byte,
    ) {
        let a = match source {
            AluSource::Gpr(index) => self.gpr[index],
        };
        let b = match target {
            AluSource::Gpr(index) => self.gpr[index],
        };

        self.lhs_latch = Byte::ZERO;
        self.rhs_latch = op(a, b);
    }

    fn arithmetic_op_stage1(&mut self, source: AluSource, target: AluSource, invert_rhs: bool) {
        let a = match target {
            AluSource::Gpr(index) => self.gpr[index],
        };
        let b = match source {
            AluSource::Gpr(index) => self.gpr[index],
        };

        self.lhs_latch = a;
        self.rhs_latch = if invert_rhs { !b } else { b };
    }

    fn flip_pc_ra(&mut self) {
        // In hardware this is implemented with register renaming,
        // but in the emulator we just swap the values for simplicity.
        let tmp = self.spr[SPR_PROGRAM_COUNTER];
        self.spr[SPR_PROGRAM_COUNTER] = self.spr[SPR_RETURN_ADDRESS];
        self.spr[SPR_RETURN_ADDRESS] = tmp;
    }

    pub fn clock(&mut self, memory: &mut Memory, lcd: &mut Lcd, uart: &mut Uart) {
        // Move instruction stream forward
        self.stage2_instruction = self.stage1_instruction;
        self.stage1_instruction = self.stage0_instruction;

        let mut fetch_stage1 = true; // Wether we can fetch this cycle based on pipeline stage 1
        let mut fetch_stage2 = true; // Wether we can fetch and increment the PC this cycle based on pipeline stage 2

        //
        // --------------------- Stage 2 ---------------------
        //

        match self.stage2_instruction {
            Instruction::Mov(source, target) => {
                let (mem_read, value) = self.load(memory, source);
                if mem_read {
                    // This is a memory read cycle so we have to supress fetch and PC increment.
                    fetch_stage2 = false;
                }

                let mem_write = self.store(memory, target, value);
                if mem_write {
                    // This is a memory write cycle so we have to supress fetch and PC increment.
                    fetch_stage2 = false;
                }

                // The emulator code allows for simultaneous memory reads and writes here,
                // but the actual hardware doesn't support this.
                // Opcodes that actually cause this behaviour would be a bug, so we do a sanity check.
                if mem_read && mem_write {
                    unreachable!("Opcode performed memory read and write at the same time");
                }
            }
            Instruction::Prebranch => {
                // This is a dummy instruction emitted by the assembler to stop
                // PC increments while jump instructions are being executed.
                fetch_stage2 = false;
            }
            Instruction::Clc => {
                // Since CLC is achived by executing 0 + 0 on real hardware, it actually sets all the flags
                self.carry = false;
                self.zero = false;
                self.overflow = false;
                self.sign = false;
            }
            Instruction::Shl(target) => {
                self.alu_stage2(Some(target), Some(0));
            }
            Instruction::Shr(target) => {
                self.alu_stage2(Some(target), Some(0));
            }
            Instruction::Add(_, target) => {
                self.alu_stage2(Some(target), Some(0));
            }
            Instruction::Addc(_, target) => {
                self.alu_stage2(Some(target), None);
            }
            Instruction::Inc(target) => match target {
                CountTarget::Gpr(index) => self.alu_stage2(Some(AluSource::Gpr(index)), Some(1)),
                CountTarget::Spr(index) => {
                    self.spr[index] += 1;
                }
            },
            Instruction::Incc(target) => {
                self.alu_stage2(Some(target), None);
            }
            Instruction::Sub(_, target) => {
                self.alu_stage2(Some(target), Some(1));
            }
            Instruction::Subb(_, target) => {
                self.alu_stage2(Some(target), None);
            }
            Instruction::Dec(target) => match target {
                CountTarget::Gpr(index) => self.alu_stage2(Some(AluSource::Gpr(index)), Some(0)),
                CountTarget::Spr(_) => {} // Decrementing SPRs takes place in stage 1
            },
            Instruction::And(_, target) => {
                self.alu_stage2(Some(target), Some(0));
            }
            Instruction::Or(_, target) => {
                self.alu_stage2(Some(target), Some(0));
            }
            Instruction::Xor(_, target) => {
                self.alu_stage2(Some(target), Some(0));
            }
            Instruction::Not(target) => {
                self.alu_stage2(Some(target), Some(0));
            }
            Instruction::Cmp(_, _) => {
                // Same as SUB except no target
                self.alu_stage2(None, Some(1));
            }
            Instruction::Test(_) => {
                self.alu_stage2(None, Some(0));
            }
            Instruction::Push(source) => {
                let addr = self.spr[SPR_STACK_POINTER];
                let value = match source {
                    StackTarget::Gpr(index) => self.gpr[index],
                    StackTarget::TransferLow => self.transfer.low(),
                    StackTarget::TransferHigh => self.transfer.high(),
                };
                memory.write(addr.into(), value.into());

                // This is a memory write cycle so we have to supress fetch and PC increment.
                fetch_stage2 = false;
            }
            Instruction::Pop(target) => {
                let addr = self.spr[SPR_STACK_POINTER];
                let value = memory.read(addr.into());
                match target {
                    StackTarget::Gpr(index) => self.gpr[index] = value.into(),
                    StackTarget::TransferLow => self.transfer.set_low(value.into()),
                    StackTarget::TransferHigh => self.transfer.set_high(value.into()),
                }

                self.spr[SPR_STACK_POINTER] += 1;

                // This is a memory read cycle so we have to supress fetch and PC increment.
                fetch_stage2 = false;
            }
            Instruction::Call(_) => self.flip_pc_ra(),
            Instruction::Ret => self.flip_pc_ra(),
            Instruction::Out(source, target) => {
                let value = match source {
                    AluSource::Gpr(index) => self.gpr[index],
                };

                match target {
                    IORegister::LcdCmd => lcd.write_cmd(value.into()),
                    IORegister::LcdData => lcd.write_data(value.into()),
                    IORegister::UartData => uart.write_data(value.into()),
                    IORegister::UartCtrl => unreachable!(), // Register is not writable, sanity check
                }
            }
            Instruction::In(source, target) => {
                let value = match source {
                    IORegister::LcdCmd => unreachable!(), // Register is not readable, sanity check
                    IORegister::LcdData => lcd.read_data(),
                    IORegister::UartData => uart.read_data(),
                    IORegister::UartCtrl => uart.read_ctrl(),
                };

                match target {
                    AluSource::Gpr(index) => self.gpr[index] = value.into(),
                }
            }
            _ => {}
        }

        //
        // --------------------- Stage 1 ---------------------
        //

        // If stage 2 has supressed the fetch, this data is invalid,
        // but we read it regardless for simplicity.
        let mem_data = memory.read(self.spr[SPR_PROGRAM_COUNTER].into());

        match self.stage1_instruction {
            Instruction::Mov(source, _) => {
                if let LoadSource::Constant = source {
                    self.constant = mem_data.into();

                    // The current memory data is a constant value belonging to the instruction,
                    // so we have to supress the fetch during this cycle.
                    fetch_stage1 = false;
                }
            }
            Instruction::MovWord(source, target) => {
                let value = self.load_word(source);
                self.store_word(target, value);
            }
            Instruction::Jmp(target) => {
                self.jump_to(target);
            }
            Instruction::Jo(target) => {
                if self.overflow {
                    self.jump_to(target);
                }
            }
            Instruction::Jno(target) => {
                if !self.overflow {
                    self.jump_to(target);
                }
            }
            Instruction::Js(target) => {
                if self.sign {
                    self.jump_to(target);
                }
            }
            Instruction::Jns(target) => {
                if !self.sign {
                    self.jump_to(target);
                }
            }
            Instruction::Jz(target) => {
                if self.zero {
                    self.jump_to(target);
                }
            }
            Instruction::Jnz(target) => {
                if !self.zero {
                    self.jump_to(target);
                }
            }
            Instruction::Jc(target) => {
                if self.carry {
                    self.jump_to(target);
                }
            }
            Instruction::Jnc(target) => {
                if !self.carry {
                    self.jump_to(target);
                }
            }
            Instruction::Jna(target) => {
                if self.carry || self.zero {
                    self.jump_to(target);
                }
            }
            Instruction::Ja(target) => {
                if !self.carry && !self.zero {
                    self.jump_to(target);
                }
            }
            Instruction::Jl(target) => {
                if self.sign != self.overflow {
                    self.jump_to(target);
                }
            }
            Instruction::Jge(target) => {
                if self.sign == self.overflow {
                    self.jump_to(target);
                }
            }
            Instruction::Jle(target) => {
                if self.zero || (self.sign != self.overflow) {
                    self.jump_to(target);
                }
            }
            Instruction::Jg(target) => {
                if !self.zero && (self.sign == self.overflow) {
                    self.jump_to(target);
                }
            }
            Instruction::Jlc(target) => {
                if self.logical_carry {
                    self.jump_to(target);
                }
            }
            Instruction::Jnlc(target) => {
                if !self.logical_carry {
                    self.jump_to(target);
                }
            }
            Instruction::Shl(target) => self.shift_op_stage1(target, |v| v << 1),
            Instruction::Shr(target) => self.shift_op_stage1(target, |v| v >> 1),
            Instruction::Add(source, target) => {
                self.arithmetic_op_stage1(source, target, false);
            }
            Instruction::Addc(source, target) => {
                self.arithmetic_op_stage1(source, target, false);
            }
            Instruction::Inc(target) => match target {
                CountTarget::Gpr(index) => {
                    self.rhs_latch = Byte::ZERO;
                    self.lhs_latch = self.gpr[index];
                }
                CountTarget::Spr(_) => {} // Incrementing SPRs takes place in stage 2
            },
            Instruction::Incc(target) => {
                self.rhs_latch = Byte::ZERO;
                self.lhs_latch = match target {
                    AluSource::Gpr(index) => self.gpr[index],
                };
            }
            Instruction::Sub(source, target) => {
                self.arithmetic_op_stage1(source, target, true);
            }
            Instruction::Subb(source, target) => {
                self.arithmetic_op_stage1(source, target, true);
            }
            Instruction::Dec(target) => match target {
                CountTarget::Gpr(index) => {
                    self.rhs_latch = Byte::MAX;
                    self.lhs_latch = self.gpr[index];
                }
                CountTarget::Spr(index) => {
                    self.spr[index] -= 1;
                }
            },
            Instruction::And(source, target) => {
                self.logic_op_stage1(source, target, |a, b| a & b);
            }
            Instruction::Or(source, target) => {
                self.logic_op_stage1(source, target, |a, b| a | b);
            }
            Instruction::Xor(source, target) => {
                self.logic_op_stage1(source, target, |a, b| a ^ b);
            }
            Instruction::Not(target) => {
                self.lhs_latch = Byte::ZERO;
                self.rhs_latch = !match target {
                    AluSource::Gpr(index) => self.gpr[index],
                };
            }
            Instruction::Cmp(source1, source2) => {
                // Same as SUB
                self.arithmetic_op_stage1(source1, source2, true);
            }
            Instruction::Test(source) => {
                // Same as AND with itself
                self.logic_op_stage1(source, source, |a, b| a & b);
            }
            Instruction::Push(_) => {
                self.spr[SPR_STACK_POINTER] -= 1;
            }
            Instruction::Call(target) => {
                let addr = match target {
                    JumpTarget::Transfer => self.transfer,
                    JumpTarget::Spr(index) => self.spr[index],
                };

                // RA will be swapped with PC in stage 2
                self.spr[SPR_RETURN_ADDRESS] = addr;
            }
            _ => {}
        }

        //
        // --------------------- Stage 0 ---------------------
        //

        // Fetch
        if fetch_stage1 && fetch_stage2 {
            // We can safely fetch
            self.stage0_instruction = decode_opcode(mem_data);
        } else if fetch_stage1 || fetch_stage2 {
            // One of the stages prevents the fetch
            self.stage0_instruction = Instruction::Nop;
        } else {
            // Both stages prevent the fetch. This means we have a pipeline contention,
            // so we have to feed the failed instruction in stage 1 back in.
            self.stage0_instruction = self.stage1_instruction;
        }

        // If stage 2 didn't access the memory bus, increment PC
        if fetch_stage2 {
            self.spr[SPR_PROGRAM_COUNTER] += 1;
        }
    }
}

fn decode_opcode(opcode: u8) -> Instruction {
    // Note: while the pipeline CPU is in development, opcodes are not finalized
    match opcode {
        0x00 => Instruction::Nop,

        // 8bit registers
        0x01 => Instruction::Mov(LoadSource::Constant, StoreTarget::Gpr(GPR_A)), // mov a,#1
        0x02 => Instruction::Mov(LoadSource::Constant, StoreTarget::Gpr(GPR_B)), // mov b,#1
        0x03 => Instruction::Mov(LoadSource::Constant, StoreTarget::Gpr(GPR_C)), // mov c,#1
        0x04 => Instruction::Mov(LoadSource::Constant, StoreTarget::Gpr(GPR_D)), // mov d,#1

        0x05 => Instruction::Mov(LoadSource::Constant, StoreTarget::TransferLow), // mov tl,#1
        0x06 => Instruction::Mov(LoadSource::Constant, StoreTarget::TransferHigh), // mov th,#1

        0x07 => Instruction::Mov(LoadSource::Gpr(GPR_B), StoreTarget::Gpr(GPR_A)), // mov a,b
        0x08 => Instruction::Mov(LoadSource::Gpr(GPR_C), StoreTarget::Gpr(GPR_A)), // mov a,c
        0x09 => Instruction::Mov(LoadSource::Gpr(GPR_D), StoreTarget::Gpr(GPR_A)), // mov a,d
        0x0A => Instruction::Mov(LoadSource::Gpr(GPR_A), StoreTarget::Gpr(GPR_B)), // mov b,a
        0x0B => Instruction::Mov(LoadSource::Gpr(GPR_C), StoreTarget::Gpr(GPR_B)), // mov b,c
        0x0C => Instruction::Mov(LoadSource::Gpr(GPR_D), StoreTarget::Gpr(GPR_B)), // mov b,d
        0x0D => Instruction::Mov(LoadSource::Gpr(GPR_A), StoreTarget::Gpr(GPR_C)), // mov c,a
        0x0E => Instruction::Mov(LoadSource::Gpr(GPR_B), StoreTarget::Gpr(GPR_C)), // mov c,b
        0x0F => Instruction::Mov(LoadSource::Gpr(GPR_D), StoreTarget::Gpr(GPR_C)), // mov c,d
        0x10 => Instruction::Mov(LoadSource::Gpr(GPR_A), StoreTarget::Gpr(GPR_D)), // mov d,a
        0x11 => Instruction::Mov(LoadSource::Gpr(GPR_B), StoreTarget::Gpr(GPR_D)), // mov d,b
        0x12 => Instruction::Mov(LoadSource::Gpr(GPR_C), StoreTarget::Gpr(GPR_D)), // mov d,c

        0x13 => Instruction::Mov(LoadSource::Gpr(GPR_A), StoreTarget::TransferLow), // mov tl,a
        0x14 => Instruction::Mov(LoadSource::Gpr(GPR_B), StoreTarget::TransferLow), // mov tl,b
        0x15 => Instruction::Mov(LoadSource::Gpr(GPR_C), StoreTarget::TransferLow), // mov tl,c
        0x16 => Instruction::Mov(LoadSource::Gpr(GPR_D), StoreTarget::TransferLow), // mov tl,d

        0x17 => Instruction::Mov(LoadSource::Gpr(GPR_A), StoreTarget::TransferHigh), // mov th,a
        0x18 => Instruction::Mov(LoadSource::Gpr(GPR_B), StoreTarget::TransferHigh), // mov th,b
        0x19 => Instruction::Mov(LoadSource::Gpr(GPR_C), StoreTarget::TransferHigh), // mov th,c
        0x1A => Instruction::Mov(LoadSource::Gpr(GPR_D), StoreTarget::TransferHigh), // mov th,d

        0x1B => Instruction::Mov(LoadSource::TransferLow, StoreTarget::Gpr(GPR_A)), // mov a,tl
        0x1C => Instruction::Mov(LoadSource::TransferLow, StoreTarget::Gpr(GPR_B)), // mov b,tl
        0x1D => Instruction::Mov(LoadSource::TransferLow, StoreTarget::Gpr(GPR_C)), // mov c,tl
        0x1E => Instruction::Mov(LoadSource::TransferLow, StoreTarget::Gpr(GPR_D)), // mov d,tl

        0x1F => Instruction::Mov(LoadSource::TransferHigh, StoreTarget::Gpr(GPR_A)), // mov a,th
        0x20 => Instruction::Mov(LoadSource::TransferHigh, StoreTarget::Gpr(GPR_B)), // mov b,th
        0x21 => Instruction::Mov(LoadSource::TransferHigh, StoreTarget::Gpr(GPR_C)), // mov c,th
        0x22 => Instruction::Mov(LoadSource::TransferHigh, StoreTarget::Gpr(GPR_D)), // mov d,th

        // 16bit registers
        0x23 => Instruction::MovWord(
            LoadWordSource::Transfer,
            StoreWordTarget::Spr(SPR_RETURN_ADDRESS),
        ), // mov ra,tx
        0x24 => Instruction::MovWord(
            LoadWordSource::Spr(SPR_RETURN_ADDRESS),
            StoreWordTarget::Transfer,
        ), // mov tx,ra
        0x25 => Instruction::MovWord(
            LoadWordSource::Transfer,
            StoreWordTarget::Spr(SPR_STACK_POINTER),
        ), // mov sp,tx
        0x26 => Instruction::MovWord(
            LoadWordSource::Spr(SPR_STACK_POINTER),
            StoreWordTarget::Transfer,
        ), // mov tx,sp
        0x27 => Instruction::MovWord(
            LoadWordSource::Transfer,
            StoreWordTarget::Spr(SPR_SOURCE_INDEX),
        ), // mov si,tx
        0x28 => Instruction::MovWord(
            LoadWordSource::Spr(SPR_SOURCE_INDEX),
            StoreWordTarget::Transfer,
        ), // mov tx,si
        0x29 => Instruction::MovWord(
            LoadWordSource::Transfer,
            StoreWordTarget::Spr(SPR_DESTINATION_INDEX),
        ), // mov di,tx
        0x2A => Instruction::MovWord(
            LoadWordSource::Spr(SPR_DESTINATION_INDEX),
            StoreWordTarget::Transfer,
        ), // mov tx,di

        0x2B => Instruction::MovWord(
            LoadWordSource::Spr(SPR_SOURCE_INDEX),
            StoreWordTarget::Spr(SPR_DESTINATION_INDEX),
        ), // mov di,si
        0x2C => Instruction::MovWord(
            LoadWordSource::Spr(SPR_DESTINATION_INDEX),
            StoreWordTarget::Spr(SPR_SOURCE_INDEX),
        ), // mov si,di
        0x2D => Instruction::MovWord(
            LoadWordSource::Spr(SPR_STACK_POINTER),
            StoreWordTarget::Spr(SPR_SOURCE_INDEX),
        ), // mov si,sp
        0x2E => Instruction::MovWord(
            LoadWordSource::Spr(SPR_STACK_POINTER),
            StoreWordTarget::Spr(SPR_DESTINATION_INDEX),
        ), // mov di,sp

        0x30 => Instruction::Dec(CountTarget::Spr(SPR_RETURN_ADDRESS)), // dec ra
        0x31 => Instruction::Dec(CountTarget::Spr(SPR_STACK_POINTER)),  // dec sp
        0x32 => Instruction::Dec(CountTarget::Spr(SPR_SOURCE_INDEX)),   // dec si
        0x33 => Instruction::Dec(CountTarget::Spr(SPR_DESTINATION_INDEX)), // dec di

        0x34 => Instruction::Inc(CountTarget::Spr(SPR_STACK_POINTER)), // inc sp
        0x35 => Instruction::Inc(CountTarget::Spr(SPR_SOURCE_INDEX)),  // inc si
        0x36 => Instruction::Inc(CountTarget::Spr(SPR_DESTINATION_INDEX)), // inc di

        // IO
        0x37 => Instruction::Out(AluSource::Gpr(GPR_A), IORegister::LcdCmd), // out lcdCmd,a
        0x38 => Instruction::Out(AluSource::Gpr(GPR_A), IORegister::LcdData), // out lcdData,a

        0x39 => Instruction::Out(AluSource::Gpr(GPR_A), IORegister::UartData), // out uartData,a
        0x3A => Instruction::In(IORegister::UartData, AluSource::Gpr(GPR_A)),  // in a,uartData
        0x3B => Instruction::In(IORegister::UartCtrl, AluSource::Gpr(GPR_A)),  // in a,uartCtrl

        // memory
        0x40 => Instruction::Mov(
            LoadSource::SprIndirect(SPR_SOURCE_INDEX),
            StoreTarget::Gpr(GPR_A),
        ), // mov a,[si]
        0x41 => Instruction::Mov(
            LoadSource::SprIndirect(SPR_SOURCE_INDEX),
            StoreTarget::Gpr(GPR_B),
        ), // mov b,[si]
        0x42 => Instruction::Mov(
            LoadSource::SprIndirect(SPR_SOURCE_INDEX),
            StoreTarget::Gpr(GPR_C),
        ), // mov c,[si]
        0x43 => Instruction::Mov(
            LoadSource::SprIndirect(SPR_SOURCE_INDEX),
            StoreTarget::Gpr(GPR_D),
        ), // mov d,[si]

        0x44 => Instruction::Mov(
            LoadSource::SprIndirect(SPR_DESTINATION_INDEX),
            StoreTarget::Gpr(GPR_A),
        ), // mov a,[di]
        0x45 => Instruction::Mov(
            LoadSource::SprIndirect(SPR_DESTINATION_INDEX),
            StoreTarget::Gpr(GPR_B),
        ), // mov b,[di]
        0x46 => Instruction::Mov(
            LoadSource::SprIndirect(SPR_DESTINATION_INDEX),
            StoreTarget::Gpr(GPR_C),
        ), // mov c,[di]
        0x47 => Instruction::Mov(
            LoadSource::SprIndirect(SPR_DESTINATION_INDEX),
            StoreTarget::Gpr(GPR_D),
        ), // mov d,[di]

        0x48 => Instruction::Mov(LoadSource::TransferIndirect, StoreTarget::Gpr(GPR_A)), // mov a,[tx]
        0x49 => Instruction::Mov(LoadSource::TransferIndirect, StoreTarget::Gpr(GPR_B)), // mov b,[tx]
        0x4A => Instruction::Mov(LoadSource::TransferIndirect, StoreTarget::Gpr(GPR_C)), // mov c,[tx]
        0x4B => Instruction::Mov(LoadSource::TransferIndirect, StoreTarget::Gpr(GPR_D)), // mov d,[tx]

        0x4C => Instruction::Mov(
            LoadSource::Gpr(GPR_A),
            StoreTarget::SprIndirect(SPR_SOURCE_INDEX),
        ), // mov [si],a
        0x4D => Instruction::Mov(
            LoadSource::Gpr(GPR_B),
            StoreTarget::SprIndirect(SPR_SOURCE_INDEX),
        ), // mov [si],b
        0x4E => Instruction::Mov(
            LoadSource::Gpr(GPR_C),
            StoreTarget::SprIndirect(SPR_SOURCE_INDEX),
        ), // mov [si],c
        0x4F => Instruction::Mov(
            LoadSource::Gpr(GPR_D),
            StoreTarget::SprIndirect(SPR_SOURCE_INDEX),
        ), // mov [si],d

        0x50 => Instruction::Mov(
            LoadSource::Gpr(GPR_A),
            StoreTarget::SprIndirect(SPR_DESTINATION_INDEX),
        ), // mov [di],a
        0x51 => Instruction::Mov(
            LoadSource::Gpr(GPR_B),
            StoreTarget::SprIndirect(SPR_DESTINATION_INDEX),
        ), // mov [di],b
        0x52 => Instruction::Mov(
            LoadSource::Gpr(GPR_C),
            StoreTarget::SprIndirect(SPR_DESTINATION_INDEX),
        ), // mov [di],c
        0x53 => Instruction::Mov(
            LoadSource::Gpr(GPR_D),
            StoreTarget::SprIndirect(SPR_DESTINATION_INDEX),
        ), // mov [di],d

        0x54 => Instruction::Mov(LoadSource::Gpr(GPR_A), StoreTarget::TransferIndirect), // mov [tx]a
        0x55 => Instruction::Mov(LoadSource::Gpr(GPR_B), StoreTarget::TransferIndirect), // mov [tx]b
        0x56 => Instruction::Mov(LoadSource::Gpr(GPR_C), StoreTarget::TransferIndirect), // mov [tx]c
        0x57 => Instruction::Mov(LoadSource::Gpr(GPR_D), StoreTarget::TransferIndirect), // mov [tx]d

        // flow control
        0x5C => Instruction::Call(JumpTarget::Transfer), // call tx
        0x5D => Instruction::Call(JumpTarget::Spr(SPR_DESTINATION_INDEX)), // call di
        0x5E => Instruction::Ret,                        // ret

        0x5F => Instruction::Prebranch,

        0x60 => Instruction::Jmp(JumpTarget::Transfer), // jmp tx
        0x71 => Instruction::Jmp(JumpTarget::Spr(SPR_DESTINATION_INDEX)), // jmp di

        0x61 => Instruction::Jo(JumpTarget::Transfer), // jo tx
        0x62 => Instruction::Jno(JumpTarget::Transfer), // jno tx
        0x63 => Instruction::Js(JumpTarget::Transfer), // js tx
        0x64 => Instruction::Jns(JumpTarget::Transfer), // jns tx
        0x65 => Instruction::Jz(JumpTarget::Transfer), // jz tx   // je tx
        0x66 => Instruction::Jnz(JumpTarget::Transfer), // jnz tx  // jne tx
        0x67 => Instruction::Jc(JumpTarget::Transfer), // jc tx   // jb tx   // jnae tx
        0x68 => Instruction::Jnc(JumpTarget::Transfer), // jnc tx  // jnb tx  // jae tx
        0x69 => Instruction::Jna(JumpTarget::Transfer), // jna tx  // jbe tx
        0x6A => Instruction::Ja(JumpTarget::Transfer), // ja tx   // jnbe tx
        0x6B => Instruction::Jl(JumpTarget::Transfer), // jl tx   // jnge tx
        0x6C => Instruction::Jge(JumpTarget::Transfer), // jge tx  // jnl tx
        0x6D => Instruction::Jle(JumpTarget::Transfer), // jle tx  // jng tx
        0x6E => Instruction::Jg(JumpTarget::Transfer), // jg tx   // jnle tx
        0x6F => Instruction::Jlc(JumpTarget::Transfer), // jlc tx
        0x70 => Instruction::Jnlc(JumpTarget::Transfer), // jnlc tx

        // stack
        0x72 => Instruction::Push(StackTarget::Gpr(GPR_A)), // push a
        0x73 => Instruction::Push(StackTarget::Gpr(GPR_B)), // push b
        0x74 => Instruction::Push(StackTarget::Gpr(GPR_C)), // push c
        0x75 => Instruction::Push(StackTarget::Gpr(GPR_D)), // push d

        0x76 => Instruction::Push(StackTarget::TransferLow), // push tl
        0x77 => Instruction::Push(StackTarget::TransferHigh), // push th

        0x78 => Instruction::Pop(StackTarget::Gpr(GPR_A)), // pop a
        0x79 => Instruction::Pop(StackTarget::Gpr(GPR_B)), // pop b
        0x7A => Instruction::Pop(StackTarget::Gpr(GPR_C)), // pop c
        0x7B => Instruction::Pop(StackTarget::Gpr(GPR_D)), // pop d

        0x7C => Instruction::Pop(StackTarget::TransferLow), // pop tl
        0x7D => Instruction::Pop(StackTarget::TransferHigh), // pop th

        // ALU
        0x7F => Instruction::Clc, // clc

        0x80 => Instruction::Shl(AluSource::Gpr(GPR_A)), // shl a
        0x81 => Instruction::Shl(AluSource::Gpr(GPR_B)), // shl b
        0x82 => Instruction::Shl(AluSource::Gpr(GPR_C)), // shl c
        0x83 => Instruction::Shl(AluSource::Gpr(GPR_D)), // shl d

        0x84 => Instruction::Shr(AluSource::Gpr(GPR_A)), // shr a
        0x85 => Instruction::Shr(AluSource::Gpr(GPR_B)), // shr b
        0x86 => Instruction::Shr(AluSource::Gpr(GPR_C)), // shr c
        0x87 => Instruction::Shr(AluSource::Gpr(GPR_D)), // shr d

        0x88 => Instruction::Add(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_A)), // add a,b
        0x89 => Instruction::Add(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_A)), // add a,c
        0x8A => Instruction::Add(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_A)), // add a,d
        0x8B => Instruction::Add(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_B)), // add b,a
        0x8C => Instruction::Add(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_B)), // add b,c
        0x8D => Instruction::Add(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_B)), // add b,d
        0x8E => Instruction::Add(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_C)), // add c,a
        0x8F => Instruction::Add(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_C)), // add c,b
        0x90 => Instruction::Add(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_C)), // add c,d
        0x91 => Instruction::Add(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_D)), // add d,a
        0x92 => Instruction::Add(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_D)), // add d,b
        0x93 => Instruction::Add(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_D)), // add d,c

        0x94 => Instruction::Addc(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_A)), // addc a,b
        0x95 => Instruction::Addc(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_A)), // addc a,c
        0x96 => Instruction::Addc(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_A)), // addc a,d
        0x97 => Instruction::Addc(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_B)), // addc b,a
        0x98 => Instruction::Addc(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_B)), // addc b,c
        0x99 => Instruction::Addc(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_B)), // addc b,d
        0x9A => Instruction::Addc(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_C)), // addc c,a
        0x9B => Instruction::Addc(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_C)), // addc c,b
        0x9C => Instruction::Addc(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_C)), // addc c,d
        0x9D => Instruction::Addc(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_D)), // addc d,a
        0x9E => Instruction::Addc(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_D)), // addc d,b
        0x9F => Instruction::Addc(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_D)), // addc d,c

        0xA0 => Instruction::Inc(CountTarget::Gpr(GPR_A)), // inc a
        0xA1 => Instruction::Inc(CountTarget::Gpr(GPR_B)), // inc b
        0xA2 => Instruction::Inc(CountTarget::Gpr(GPR_C)), // inc c
        0xA3 => Instruction::Inc(CountTarget::Gpr(GPR_D)), // inc d

        0xA4 => Instruction::Incc(AluSource::Gpr(GPR_A)), // incc a
        0xA5 => Instruction::Incc(AluSource::Gpr(GPR_B)), // incc b
        0xA6 => Instruction::Incc(AluSource::Gpr(GPR_C)), // incc c
        0xA7 => Instruction::Incc(AluSource::Gpr(GPR_D)), // incc d

        0xA8 => Instruction::Sub(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_A)), // sub a,b
        0xA9 => Instruction::Sub(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_A)), // sub a,c
        0xAA => Instruction::Sub(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_A)), // sub a,d
        0xAB => Instruction::Sub(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_B)), // sub b,a
        0xAC => Instruction::Sub(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_B)), // sub b,c
        0xAD => Instruction::Sub(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_B)), // sub b,d
        0xAE => Instruction::Sub(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_C)), // sub c,a
        0xAF => Instruction::Sub(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_C)), // sub c,b
        0xB0 => Instruction::Sub(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_C)), // sub c,d
        0xB1 => Instruction::Sub(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_D)), // sub d,a
        0xB2 => Instruction::Sub(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_D)), // sub d,b
        0xB3 => Instruction::Sub(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_D)), // sub d,c

        0xB4 => Instruction::Subb(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_A)), // subb a,b
        0xB5 => Instruction::Subb(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_A)), // subb a,c
        0xB6 => Instruction::Subb(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_A)), // subb a,d
        0xB7 => Instruction::Subb(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_B)), // subb b,a
        0xB8 => Instruction::Subb(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_B)), // subb b,c
        0xB9 => Instruction::Subb(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_B)), // subb b,d
        0xBA => Instruction::Subb(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_C)), // subb c,a
        0xBB => Instruction::Subb(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_C)), // subb c,b
        0xBC => Instruction::Subb(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_C)), // subb c,d
        0xBD => Instruction::Subb(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_D)), // subb d,a
        0xBE => Instruction::Subb(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_D)), // subb d,b
        0xBF => Instruction::Subb(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_D)), // subb d,c

        0xC0 => Instruction::Dec(CountTarget::Gpr(GPR_A)), // dec a
        0xC1 => Instruction::Dec(CountTarget::Gpr(GPR_B)), // dec b
        0xC2 => Instruction::Dec(CountTarget::Gpr(GPR_C)), // dec c
        0xC3 => Instruction::Dec(CountTarget::Gpr(GPR_D)), // dec d

        0xC4 => Instruction::And(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_A)), // and a,b
        0xC5 => Instruction::And(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_A)), // and a,c
        0xC6 => Instruction::And(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_A)), // and a,d
        0xC7 => Instruction::And(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_B)), // and b,a
        0xC8 => Instruction::And(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_B)), // and b,c
        0xC9 => Instruction::And(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_B)), // and b,d
        0xCA => Instruction::And(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_C)), // and c,a
        0xCB => Instruction::And(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_C)), // and c,b
        0xCC => Instruction::And(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_C)), // and c,d
        0xCD => Instruction::And(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_D)), // and d,a
        0xCE => Instruction::And(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_D)), // and d,b
        0xCF => Instruction::And(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_D)), // and d,c

        0xD0 => Instruction::Or(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_A)), // or a,b
        0xD1 => Instruction::Or(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_A)), // or a,c
        0xD2 => Instruction::Or(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_A)), // or a,d
        0xD3 => Instruction::Or(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_B)), // or b,a
        0xD4 => Instruction::Or(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_B)), // or b,c
        0xD5 => Instruction::Or(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_B)), // or b,d
        0xD6 => Instruction::Or(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_C)), // or c,a
        0xD7 => Instruction::Or(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_C)), // or c,b
        0xD8 => Instruction::Or(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_C)), // or c,d
        0xD9 => Instruction::Or(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_D)), // or d,a
        0xDA => Instruction::Or(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_D)), // or d,b
        0xDB => Instruction::Or(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_D)), // or d,c

        0xDC => Instruction::Xor(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_A)), // xor a,b
        0xDD => Instruction::Xor(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_A)), // xor a,c
        0xDE => Instruction::Xor(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_A)), // xor a,d
        0xDF => Instruction::Xor(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_B)), // xor b,a
        0xE0 => Instruction::Xor(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_B)), // xor b,c
        0xE1 => Instruction::Xor(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_B)), // xor b,d
        0xE2 => Instruction::Xor(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_C)), // xor c,a
        0xE3 => Instruction::Xor(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_C)), // xor c,b
        0xE4 => Instruction::Xor(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_C)), // xor c,d
        0xE5 => Instruction::Xor(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_D)), // xor d,a
        0xE6 => Instruction::Xor(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_D)), // xor d,b
        0xE7 => Instruction::Xor(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_D)), // xor d,c

        0xE8 => Instruction::Xor(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_A)), // xor a,a
        0xE9 => Instruction::Xor(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_B)), // xor b,b
        0xEA => Instruction::Xor(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_C)), // xor c,c
        0xEB => Instruction::Xor(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_D)), // xor d,d

        0xEC => Instruction::Not(AluSource::Gpr(GPR_A)), // not a
        0xED => Instruction::Not(AluSource::Gpr(GPR_B)), // not b
        0xEE => Instruction::Not(AluSource::Gpr(GPR_C)), // not c
        0xEF => Instruction::Not(AluSource::Gpr(GPR_D)), // not d

        0xF0 => Instruction::Cmp(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_A)), // cmp a,b
        0xF1 => Instruction::Cmp(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_A)), // cmp a,c
        0xF2 => Instruction::Cmp(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_A)), // cmp a,d
        0xF3 => Instruction::Cmp(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_B)), // cmp b,a
        0xF4 => Instruction::Cmp(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_B)), // cmp b,c
        0xF5 => Instruction::Cmp(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_B)), // cmp b,d
        0xF6 => Instruction::Cmp(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_C)), // cmp c,a
        0xF7 => Instruction::Cmp(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_C)), // cmp c,b
        0xF8 => Instruction::Cmp(AluSource::Gpr(GPR_D), AluSource::Gpr(GPR_C)), // cmp c,d
        0xF9 => Instruction::Cmp(AluSource::Gpr(GPR_A), AluSource::Gpr(GPR_D)), // cmp d,a
        0xFA => Instruction::Cmp(AluSource::Gpr(GPR_B), AluSource::Gpr(GPR_D)), // cmp d,b
        0xFB => Instruction::Cmp(AluSource::Gpr(GPR_C), AluSource::Gpr(GPR_D)), // cmp d,c

        0xFC => Instruction::Test(AluSource::Gpr(GPR_A)), // test a
        0xFD => Instruction::Test(AluSource::Gpr(GPR_B)), // test b
        0xFE => Instruction::Test(AluSource::Gpr(GPR_C)), // test c
        0xFF => Instruction::Test(AluSource::Gpr(GPR_D)), // test d

        _ => Instruction::Nop, // Any invalid instruction is treated as a NOP
    }
}
