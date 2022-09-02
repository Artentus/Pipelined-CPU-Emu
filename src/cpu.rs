use bitflags::bitflags;
use modular_bitfield::error::InvalidBitPattern;
use modular_bitfield::*;
use std::fmt::Display;

use crate::{Audio, Controler, Lcd, Memory, Uart, Vga};

const PIPE_ROM_SIZE: usize = 0x8000;
type PipeRom = &'static [u8; PIPE_ROM_SIZE];

const PIPE_1A: PipeRom = include_bytes!("../res/Pipe1A.bin");
const PIPE_1B: PipeRom = include_bytes!("../res/Pipe1B.bin");
const PIPE_2A: PipeRom = include_bytes!("../res/Pipe2A.bin");
const PIPE_2B: PipeRom = include_bytes!("../res/Pipe2B.bin");

const LOGICAL_CARRY_PRESERVE_JUMPER: bool = false;

#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
enum AluBusRegister {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
}

#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 4]
enum AluOp {
    Nop = 0,
    Shl = 1,
    Shr = 2,
    Add = 3,
    AddC = 4,
    Inc = 5,
    IncC = 6,
    Sub = 7,
    SubB = 8,
    Dec = 9,
    And = 10,
    Or = 11,
    Xor = 12,
    Not = 13,
    Clc = 14,
}

#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 3]
enum TransferBusRegister {
    None = 0,
    PcRa0 = 1,
    PcRa1 = 2,
    Sp = 3,
    Si = 4,
    Di = 5,
    Tx = 6,
}

#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 4]
enum MainBusAssertDevice {
    None = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    Constant = 5,
    Tl = 6,
    Th = 7,
    AluResult = 8,
    IoCntrl = 9,
    IoVga = 10,
    IoUartData = 11,
    IoUartCtrl = 12,
    IoLcdCommand = 14,
    MemBridge = 15,
}

#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 4]
enum MainBusLoadDevice {
    None = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    Constant = 5,
    Tl = 6,
    Th = 7,
    IoAudioData = 9,
    IoVga = 10,
    IoUartData = 11,
    IoUartCtrl = 12,
    IoLcdData = 13,
    IoLcdCommand = 14,
    MemBridge = 15,
}

#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
enum IncrementRegister {
    None = 0,
    Sp = 1,
    Si = 2,
    Di = 3,
}

#[derive(BitfieldSpecifier, Clone, Copy, PartialEq, Eq)]
#[bits = 3]
enum AddressBusRegister {
    None = 0,
    PcRa0 = 1,
    PcRa1 = 2,
    Sp = 3,
    Si = 4,
    Di = 5,
    Tx = 6,
}

#[allow(dead_code)]
mod pipeline_data {
    use super::*;

    #[bitfield(bits = 8)]
    pub(super) struct Pipe1AData {
        pub(super) lhs_bus_assert: AluBusRegister,
        pub(super) rhs_bus_assert: AluBusRegister,
        pub(super) alu_op: AluOp,
    }

    #[bitfield(bits = 8)]
    pub(super) struct Pipe1BData {
        pub(super) transfer_bus_load: TransferBusRegister,
        pub(super) load_constant: bool, // If true `transfer_bus_load` defines which register to decrement
        pub(super) transfer_bus_assert: TransferBusRegister,
        pub(super) no_fetch: bool,
    }

    #[bitfield(bits = 8)]
    pub(super) struct Pipe2AData {
        pub(super) main_bus_assert: MainBusAssertDevice,
        pub(super) main_bus_load: MainBusLoadDevice,
    }

    #[bitfield(bits = 8)]
    pub(super) struct Pipe2BData {
        pub(super) increment_register: IncrementRegister,
        pub(super) address_bus_assert: AddressBusRegister,
        pub(super) bus_request: bool,
        pub(super) flip_pc_ra: bool,
        pub(super) break_clock: bool,
    }
}

use pipeline_data::*;

bitflags! {
    struct Flags : u8 {
        const OVERFLOW = 1<<0;
        const SIGN = 1<<1;
        const ZERO = 1<<2;
        const CARRY_A = 1<<3;
        const CARRY_L = 1<<4;
        const PC_RA_FLIP = 1<<5;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AluLhsOp {
    Pass,
    ShiftLeft,
    ShiftRight,
    Zero,
}

impl From<AluOp> for AluLhsOp {
    fn from(op: AluOp) -> Self {
        match op {
            AluOp::Shl => Self::ShiftLeft,
            AluOp::Shr => Self::ShiftRight,
            AluOp::Nop
            | AluOp::Add
            | AluOp::AddC
            | AluOp::Inc
            | AluOp::IncC
            | AluOp::Sub
            | AluOp::SubB
            | AluOp::Dec => Self::Pass,
            AluOp::And | AluOp::Or | AluOp::Xor | AluOp::Not | AluOp::Clc => Self::Zero,
        }
    }
}

fn execute_alu_lhs_op(lhs: u8, cl_in: bool, op: AluLhsOp) -> (u8, bool) {
    match op {
        AluLhsOp::Pass => {
            if LOGICAL_CARRY_PRESERVE_JUMPER {
                (lhs, cl_in)
            } else {
                (lhs, false)
            }
        }
        AluLhsOp::ShiftLeft => (
            if cl_in { (lhs << 1) | 0x01 } else { lhs << 1 },
            (lhs & 0x80) != 0,
        ),
        AluLhsOp::ShiftRight => (
            if cl_in { (lhs >> 1) | 0x80 } else { lhs >> 1 },
            (lhs & 0x01) != 0,
        ),
        AluLhsOp::Zero => (0, false),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AluRhsOp {
    PassRhs,
    NotRhs,
    Zero,
    One,
    And,
    Or,
    Xor,
}

impl From<AluOp> for AluRhsOp {
    fn from(op: AluOp) -> Self {
        match op {
            AluOp::And => Self::And,
            AluOp::Or => Self::Or,
            AluOp::Xor => Self::Xor,
            AluOp::Dec => Self::One,
            AluOp::Add | AluOp::AddC => Self::PassRhs,
            AluOp::Sub | AluOp::SubB | AluOp::Not => Self::NotRhs,
            AluOp::Nop | AluOp::Shl | AluOp::Shr | AluOp::Inc | AluOp::IncC | AluOp::Clc => {
                Self::Zero
            }
        }
    }
}

fn execute_alu_rhs_op(lhs: u8, rhs: u8, op: AluRhsOp) -> u8 {
    match op {
        AluRhsOp::PassRhs => rhs,
        AluRhsOp::NotRhs => !rhs,
        AluRhsOp::Zero => 0,
        AluRhsOp::One => 0xFF,
        AluRhsOp::And => lhs & rhs,
        AluRhsOp::Or => lhs | rhs,
        AluRhsOp::Xor => lhs ^ rhs,
    }
}

fn get_ca_override(op: AluOp) -> Option<bool> {
    match op {
        AluOp::Nop
        | AluOp::Shl
        | AluOp::Shr
        | AluOp::Add
        | AluOp::Dec
        | AluOp::And
        | AluOp::Or
        | AluOp::Xor
        | AluOp::Not
        | AluOp::Clc => Some(false),
        AluOp::Inc | AluOp::Sub => Some(true),
        AluOp::AddC | AluOp::IncC | AluOp::SubB => None,
    }
}

const NOP: u8 = 0;

pub struct Cpu {
    // special purpose registers
    pc_ra_0: u16,
    pc_ra_1: u16,
    sp: u16,
    si: u16,
    di: u16,
    tx: u16,

    // general purpose registers
    a: u8,
    b: u8,
    c: u8,
    d: u8,

    // internal registers
    constant: u8,
    alu_lhs: u8,
    alu_rhs: u8,
    ca_override: Option<bool>,
    flags: Flags,

    // pipeline registers
    stage0_instruction: u8,
    stage1_instruction: u8,
    stage2_instruction: u8,
}

impl Cpu {
    #[inline]
    pub const fn new() -> Self {
        Self {
            pc_ra_0: 0,
            pc_ra_1: 0,
            sp: 0,
            si: 0,
            di: 0,
            tx: 0,

            a: 0,
            b: 0,
            c: 0,
            d: 0,

            constant: 0,
            alu_lhs: 0,
            alu_rhs: 0,
            ca_override: None,
            flags: Flags::empty(),

            stage0_instruction: NOP,
            stage1_instruction: NOP,
            stage2_instruction: NOP,
        }
    }

    #[inline]
    pub fn reset(&mut self, pc: u16) {
        self.pc_ra_0 = pc;
        self.pc_ra_1 = 0;
        self.sp = 0;
        self.si = 0;
        self.di = 0;
        self.ca_override = None;
        self.flags = Flags::empty();
        self.stage0_instruction = NOP;
        self.stage1_instruction = NOP;
        self.stage2_instruction = NOP;
    }

    #[inline]
    fn pc(&self) -> u16 {
        if self.flags.contains(Flags::PC_RA_FLIP) {
            self.pc_ra_1
        } else {
            self.pc_ra_0
        }
    }

    #[inline]
    fn ra(&self) -> u16 {
        if self.flags.contains(Flags::PC_RA_FLIP) {
            self.pc_ra_0
        } else {
            self.pc_ra_1
        }
    }

    #[inline]
    fn inc_pc(&mut self) {
        if self.flags.contains(Flags::PC_RA_FLIP) {
            self.pc_ra_1 += 1;
        } else {
            self.pc_ra_0 += 1;
        }
    }

    #[inline]
    fn tl(&self) -> u8 {
        self.tx.to_le_bytes()[0]
    }

    #[inline]
    fn th(&self) -> u8 {
        self.tx.to_le_bytes()[1]
    }

    #[inline]
    fn set_tl(&mut self, value: u8) {
        let mut bytes = self.tx.to_le_bytes();
        bytes[0] = value;
        self.tx = u16::from_le_bytes(bytes);
    }

    #[inline]
    fn set_th(&mut self, value: u8) {
        let mut bytes = self.tx.to_le_bytes();
        bytes[1] = value;
        self.tx = u16::from_le_bytes(bytes);
    }

    fn execute_alu(&mut self) -> u8 {
        let lhs_sign = (self.alu_lhs & 0x80) != 0;
        let rhs_sign = (self.alu_rhs & 0x80) != 0;

        let ca_in = self
            .ca_override
            .unwrap_or(self.flags.contains(Flags::CARRY_A));
        let (result, ca_out) = self.alu_lhs.carrying_add(self.alu_rhs, ca_in);

        let sign = (result & 0x80) != 0;
        let zero = result == 0;
        let overflow = (lhs_sign == rhs_sign) & (lhs_sign != sign);

        self.flags.set(Flags::CARRY_A, ca_out);
        self.flags.set(Flags::SIGN, sign);
        self.flags.set(Flags::ZERO, zero);
        self.flags.set(Flags::OVERFLOW, overflow);

        result
    }

    #[inline]
    fn get_alu_bus_value(&self, reg: AluBusRegister) -> u8 {
        match reg {
            AluBusRegister::A => self.a,
            AluBusRegister::B => self.b,
            AluBusRegister::C => self.c,
            AluBusRegister::D => self.d,
        }
    }

    #[inline]
    fn get_transfer_bus_value(&self, reg: TransferBusRegister) -> u16 {
        match reg {
            TransferBusRegister::None => 0,
            TransferBusRegister::PcRa0 => self.pc_ra_0,
            TransferBusRegister::PcRa1 => self.pc_ra_1,
            TransferBusRegister::Sp => self.sp,
            TransferBusRegister::Si => self.si,
            TransferBusRegister::Di => self.di,
            TransferBusRegister::Tx => self.tx,
        }
    }

    // Returns true if a break instruction was reached
    pub fn clock(
        &mut self,
        memory: &mut Memory,
        lcd: &mut Lcd,
        uart: &mut Uart,
        audio: &mut Audio,
        vga: &mut Vga,
        controler: &mut Controler,
    ) -> Result<bool, InvalidBitPattern<u8>> {
        // Move instruction stream forward
        self.stage2_instruction = self.stage1_instruction;
        self.stage1_instruction = self.stage0_instruction;

        // Decode instructions in the pipeline using ROMs
        let flag_value = ((self.flags.bits() as usize) | 0x40) << 8;

        let pipe1_address = (self.stage1_instruction as usize) | flag_value;
        let pipe1a_data = Pipe1AData::from_bytes([PIPE_1A[pipe1_address]]);
        let pipe1b_data = Pipe1BData::from_bytes([PIPE_1B[pipe1_address]]);

        let pipe2_address = (self.stage2_instruction as usize) | flag_value;
        let pipe2a_data = Pipe2AData::from_bytes([PIPE_2A[pipe2_address]]);
        let pipe2b_data = Pipe2BData::from_bytes([PIPE_2B[pipe2_address]]);

        // The state of the PC-RA flipping is defined by the pipeline ROM output
        self.flags
            .set(Flags::PC_RA_FLIP, pipe2b_data.flip_pc_ra_or_err()?);

        // Wether we can fetch this cycle based on pipeline stage 1
        let fetch_stage1 = !pipe1b_data.no_fetch_or_err()?;
        // Wether we can fetch and increment the PC this cycle based on pipeline stage 2
        let fetch_stage2 = !pipe2b_data.bus_request_or_err()?;

        //
        // --------------------- Stage 2 ---------------------
        //

        let address = match pipe2b_data.address_bus_assert_or_err()? {
            AddressBusRegister::None => 0,
            AddressBusRegister::PcRa0 => self.pc_ra_0,
            AddressBusRegister::PcRa1 => self.pc_ra_1,
            AddressBusRegister::Sp => self.sp,
            AddressBusRegister::Si => self.si,
            AddressBusRegister::Di => self.di,
            AddressBusRegister::Tx => self.tx,
        };

        let alu_result = self.execute_alu();
        let mem_data = memory.read(vga, address);

        // If stage 2 doesn't access the memory bus, increment PC
        if fetch_stage2 {
            // On hardware, jumping and incrementing PC is actually undefined behaviour, but the way
            // we implement it here if a jump occurs in stage 1 it will override the incremented PC.
            self.inc_pc();
        }

        let main_bus = match pipe2a_data.main_bus_assert_or_err()? {
            MainBusAssertDevice::None => 0,
            MainBusAssertDevice::A => self.a,
            MainBusAssertDevice::B => self.b,
            MainBusAssertDevice::C => self.c,
            MainBusAssertDevice::D => self.d,
            MainBusAssertDevice::Constant => self.constant,
            MainBusAssertDevice::Tl => self.tl(),
            MainBusAssertDevice::Th => self.th(),
            MainBusAssertDevice::AluResult => alu_result,
            MainBusAssertDevice::IoCntrl => controler.read_data(),
            MainBusAssertDevice::IoVga => vga.read_data(),
            MainBusAssertDevice::IoUartData => uart.read_data(),
            MainBusAssertDevice::IoUartCtrl => uart.read_ctrl(),
            MainBusAssertDevice::IoLcdCommand => lcd.read_cmd(),
            MainBusAssertDevice::MemBridge => mem_data,
        };

        match pipe2a_data.main_bus_load_or_err()? {
            MainBusLoadDevice::None => {}
            MainBusLoadDevice::A => self.a = main_bus,
            MainBusLoadDevice::B => self.b = main_bus,
            MainBusLoadDevice::C => self.c = main_bus,
            MainBusLoadDevice::D => self.d = main_bus,
            MainBusLoadDevice::Constant => self.constant = main_bus,
            MainBusLoadDevice::Tl => self.set_tl(main_bus),
            MainBusLoadDevice::Th => self.set_th(main_bus),
            MainBusLoadDevice::IoAudioData => audio.write_data(main_bus),
            MainBusLoadDevice::IoVga => {}
            MainBusLoadDevice::IoUartData => uart.write_data(main_bus),
            MainBusLoadDevice::IoUartCtrl => {}
            MainBusLoadDevice::IoLcdData => lcd.write_data(main_bus),
            MainBusLoadDevice::IoLcdCommand => lcd.write_cmd(main_bus),
            MainBusLoadDevice::MemBridge => memory.write(vga, address, main_bus),
        }

        match pipe2b_data.increment_register_or_err()? {
            IncrementRegister::None => {}
            IncrementRegister::Sp => self.sp = self.sp.wrapping_add(1),
            IncrementRegister::Si => self.si = self.si.wrapping_add(1),
            IncrementRegister::Di => self.di = self.di.wrapping_add(1),
        }

        //
        // --------------------- Stage 1 ---------------------
        //

        let lhs_bus = self.get_alu_bus_value(pipe1a_data.lhs_bus_assert_or_err()?);
        let rhs_bus = self.get_alu_bus_value(pipe1a_data.rhs_bus_assert_or_err()?);
        let alu_op = pipe1a_data.alu_op_or_err()?;

        let (lhs_out, cl_out) =
            execute_alu_lhs_op(lhs_bus, self.flags.contains(Flags::CARRY_L), alu_op.into());
        let rhs_out = execute_alu_rhs_op(lhs_bus, rhs_bus, alu_op.into());

        if alu_op != AluOp::Nop {
            self.alu_lhs = lhs_out;
            self.alu_rhs = rhs_out;
            self.flags.set(Flags::CARRY_L, cl_out);
            self.ca_override = get_ca_override(alu_op);
        }

        if pipe1b_data.load_constant() {
            match pipe1b_data.transfer_bus_load_or_err()? {
                TransferBusRegister::None => self.constant = mem_data,
                TransferBusRegister::PcRa0 => self.pc_ra_0 = self.pc_ra_0.wrapping_sub(1),
                TransferBusRegister::PcRa1 => self.pc_ra_1 = self.pc_ra_1.wrapping_sub(1),
                TransferBusRegister::Sp => self.sp = self.sp.wrapping_sub(1),
                TransferBusRegister::Si => self.si = self.si.wrapping_sub(1),
                TransferBusRegister::Di => self.di = self.di.wrapping_sub(1),
                TransferBusRegister::Tx => self.tx = self.tx.wrapping_sub(1),
            }
        } else {
            let transfer_bus =
                self.get_transfer_bus_value(pipe1b_data.transfer_bus_assert_or_err()?);

            match pipe1b_data.transfer_bus_load_or_err()? {
                TransferBusRegister::None => {}
                TransferBusRegister::PcRa0 => self.pc_ra_0 = transfer_bus,
                TransferBusRegister::PcRa1 => self.pc_ra_1 = transfer_bus,
                TransferBusRegister::Sp => self.sp = transfer_bus,
                TransferBusRegister::Si => self.si = transfer_bus,
                TransferBusRegister::Di => self.di = transfer_bus,
                TransferBusRegister::Tx => self.tx = transfer_bus,
            }
        }

        //
        // --------------------- Stage 0 ---------------------
        //

        // Fetch
        if fetch_stage1 && fetch_stage2 {
            // We can safely fetch
            self.stage0_instruction = mem_data;
        } else if fetch_stage1 || fetch_stage2 {
            // One of the stages prevents the fetch
            self.stage0_instruction = NOP;
        } else {
            // Both stages prevent the fetch. This means we have a pipeline contention,
            // so we have to feed the failed instruction in stage 1 back in.
            self.stage0_instruction = self.stage1_instruction;
        }

        pipe2b_data.break_clock_or_err()
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let overflow_val: u8 = self.flags.contains(Flags::OVERFLOW).into();
        let sign_val: u8 = self.flags.contains(Flags::SIGN).into();
        let zero_val: u8 = self.flags.contains(Flags::ZERO).into();
        let carry_a_val: u8 = self.flags.contains(Flags::CARRY_A).into();
        let carry_l_val: u8 = self.flags.contains(Flags::CARRY_L).into();
        let flip_val: u8 = self.flags.contains(Flags::PC_RA_FLIP).into();

        writeln!(f, "PC: 0x{:0>4X}", self.pc())?;
        writeln!(f, "RA: 0x{:0>4X}", self.ra())?;
        writeln!(f, "SP: 0x{:0>4X}", self.sp)?;
        writeln!(f, "SI: 0x{:0>4X}", self.si)?;
        writeln!(f, "DI: 0x{:0>4X}", self.di)?;
        writeln!(f, "TX: 0x{:0>4X}", self.tx)?;
        writeln!(f)?;
        writeln!(f, "A:  0x{:0>2X}", self.a)?;
        writeln!(f, "B:  0x{:0>2X}", self.b)?;
        writeln!(f, "C:  0x{:0>2X}", self.c)?;
        writeln!(f, "D:  0x{:0>2X}", self.d)?;
        writeln!(f, "TL: 0x{:0>2X}", self.tl())?;
        writeln!(f, "TH: 0x{:0>2X}", self.th())?;
        writeln!(f)?;
        writeln!(f, "Constant: 0x{:0>2X}", self.constant)?;
        writeln!(f)?;
        writeln!(f, "F L C Z S O")?;
        writeln!(
            f,
            "{} {} {} {} {} {}",
            flip_val, carry_l_val, carry_a_val, zero_val, sign_val, overflow_val
        )?;

        Ok(())
    }
}
