use instruction::*;
use interconnect::*;

pub struct Nvc {
    reg_pc: u32,
    reg_gpr: [u32; 31],

    psw_zero: bool,
    psw_sign: bool,
    psw_overflow: bool,
    psw_carry: bool,
    psw_fp_precision_degredation: bool,
    psw_fp_underflow: bool,
    psw_fp_overflow: bool,
    psw_fp_zero_division: bool,
    psw_fp_invalid_operation: bool,
    psw_fp_reserved_operand: bool,
    psw_interrupt_disable: bool,
    psw_address_trap_enable: bool,
    psw_exception_pending: bool,
    psw_nmi_pending: bool,
    psw_interrupt_mask_level: usize,
}

impl Nvc {
    pub fn new() -> Nvc {
        Nvc {
            reg_pc: 0xfffffff0,
            reg_gpr: [0; 31],

            psw_zero: false,
            psw_sign: false,
            psw_overflow: false,
            psw_carry: false,
            psw_fp_precision_degredation: false,
            psw_fp_underflow: false,
            psw_fp_overflow: false,
            psw_fp_zero_division: false,
            psw_fp_invalid_operation: false,
            psw_fp_reserved_operand: false,
            psw_interrupt_disable: false,
            psw_address_trap_enable: false,
            psw_exception_pending: false,
            psw_nmi_pending: true,
            psw_interrupt_mask_level: 0,
        }
    }

    pub fn reg_pc(&self) -> u32 {
        self.reg_pc
    }

    pub fn reg_gpr(&self, index: usize) -> u32 {
        if index == 0 {
            0
        } else {
            self.reg_gpr[index - 1]
        }
    }

    fn set_reg_gpr(&mut self, index: usize, value: u32) {
        if index != 0 {
            self.reg_gpr[index - 1] = value;
        }
    }

    pub fn reg_psw(&self) -> u32 {
        (if self.psw_zero { 1 << 0 } else { 0 }) |
        (if self.psw_sign { 1 << 1 } else { 0 }) |
        (if self.psw_overflow { 1 << 2 } else { 0 }) |
        (if self.psw_carry { 1 << 3 } else { 0 }) |
        (if self.psw_fp_precision_degredation { 1 << 4 } else { 0 }) |
        (if self.psw_fp_underflow { 1 << 5 } else { 0 }) |
        (if self.psw_fp_overflow { 1 << 6 } else { 0 }) |
        (if self.psw_fp_zero_division { 1 << 7 } else { 0 }) |
        (if self.psw_fp_invalid_operation { 1 << 8 } else { 0 }) |
        (if self.psw_fp_reserved_operand { 1 << 9 } else { 0 }) |
        (if self.psw_interrupt_disable { 1 << 12 } else { 0 }) |
        (if self.psw_address_trap_enable { 1 << 13 } else { 0 }) |
        (if self.psw_exception_pending { 1 << 14 } else { 0 }) |
        (if self.psw_nmi_pending { 1 << 15 } else { 0 }) |
        (self.psw_interrupt_mask_level as u32) << 16
    }

    pub fn set_reg_psw(&mut self, value: u32) {
        self.psw_zero = ((value >> 0) & 0x01) != 0;
        self.psw_sign = ((value >> 1) & 0x01) != 0;
        self.psw_overflow = ((value >> 2) & 0x01) != 0;
        self.psw_carry = ((value >> 3) & 0x01) != 0;
        self.psw_fp_precision_degredation = ((value >> 4) & 0x01) != 0;
        self.psw_fp_underflow = ((value >> 5) & 0x01) != 0;
        self.psw_fp_overflow = ((value >> 6) & 0x01) != 0;
        self.psw_fp_zero_division = ((value >> 7) & 0x01) != 0;
        self.psw_fp_invalid_operation = ((value >> 8) & 0x01) != 0;
        self.psw_fp_reserved_operand = ((value >> 9) & 0x01) != 0;
        self.psw_interrupt_disable = ((value >> 12) & 0x01) != 0;
        self.psw_address_trap_enable = ((value >> 13) & 0x01) != 0;
        self.psw_exception_pending = ((value >> 14) & 0x01) != 0;
        self.psw_nmi_pending = ((value >> 15) & 0x01) != 0;
        self.psw_interrupt_mask_level = ((value as usize) >> 16) & 0x0f;
    }

    pub fn step(&mut self, interconnect: &mut Interconnect) {
        let first_halfword = interconnect.read_halfword(self.reg_pc);
        let mut next_pc = self.reg_pc.wrapping_add(2);

        let opcode = Opcode::from_halfword(first_halfword);
        let instruction_format = opcode.instruction_format();

        let second_halfword = if instruction_format.has_second_halfword() {
            let second_halfword = interconnect.read_halfword(next_pc);
            next_pc = next_pc.wrapping_add(2);
            second_halfword
        } else {
            0
        };

        // TODO: Not too convinced of this pattern, but we'll see what else we
        //  may have to special case moving forward
        let mut take_branch = false;

        match opcode {
            Opcode::MovReg => format_i(|reg1, reg2| {
                let value = self.reg_gpr(reg1);
                self.set_reg_gpr(reg2, value);
            }, first_halfword),
            Opcode::Sub => format_i(|reg1, reg2| {
                let lhs = self.reg_gpr(reg2);
                let rhs = self.reg_gpr(reg1);

                let (res, carry) = lhs.overflowing_sub(rhs);
                self.set_reg_gpr(reg2, res);

                self.set_zero_sign_flags(res);
                self.psw_overflow = (((lhs ^ rhs) & !(rhs ^ res)) & 0x80000000) != 0;
                self.psw_carry = carry;
            }, first_halfword),
            Opcode::Jmp => format_i(|reg1, _| {
                next_pc = self.reg_gpr(reg1);
            }, first_halfword),
            Opcode::MovImm => format_ii(|imm5, reg2| {
                let value = sign_extend_imm5(imm5);
                self.set_reg_gpr(reg2, value);
            }, first_halfword),
            Opcode::Cli => format_ii(|_, _| {
                self.psw_interrupt_disable = false;
            }, first_halfword),
            Opcode::Ldsr => format_ii(|imm5, reg2| {
                let value = self.reg_gpr(reg2);
                let system_register = opcode.system_register(imm5);
                match system_register {
                    SystemRegister::Psw => self.set_reg_psw(value),
                }
            }, first_halfword),
            Opcode::Sei => format_ii(|_, _| {
                self.psw_interrupt_disable = true;
            }, first_halfword),
            Opcode::Bv => {
                take_branch = self.psw_overflow;
            },
            Opcode::Bc => {
                take_branch = self.psw_carry;
            },
            Opcode::Bz => {
                take_branch = self.psw_zero;
            },
            Opcode::Bnh => {
                take_branch = self.psw_carry | self.psw_zero;
            },
            Opcode::Bn => {
                take_branch = self.psw_sign;
            },
            Opcode::Br => {
                take_branch = true;
            },
            Opcode::Blt => {
                take_branch = self.psw_sign ^ self.psw_overflow;
            },
            Opcode::Ble => {
                take_branch = (self.psw_sign ^ self.psw_overflow) != self.psw_zero;
            },
            Opcode::Bnv => {
                take_branch = !self.psw_overflow;
            },
            Opcode::Bnc => {
                take_branch = !self.psw_carry;
            },
            Opcode::Bnz => {
                take_branch = !self.psw_zero;
            },
            Opcode::Bh => {
                take_branch = !(self.psw_carry || self.psw_zero);
            },
            Opcode::Bp => {
                take_branch = !self.psw_sign;
            },
            Opcode::Nop => (),
            Opcode::Bge => {
                take_branch = !(self.psw_sign != self.psw_overflow);
            },
            Opcode::Bgt => {
                take_branch = !((self.psw_sign != self.psw_overflow) || self.psw_zero);
            },
            Opcode::Movea => format_v(|reg1, reg2, imm16| {
                let res = self.reg_gpr(reg1).wrapping_add((imm16 as i16) as u32);
                self.set_reg_gpr(reg2, res);
            }, first_halfword, second_halfword),
            Opcode::Movhi => format_v(|reg1, reg2, imm16| {
                let res = self.reg_gpr(reg1).wrapping_add((imm16 as u32) << 16);
                self.set_reg_gpr(reg2, res);
            }, first_halfword, second_halfword),
            Opcode::Ldb => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                let value = (interconnect.read_byte(addr) as i8) as u32;
                self.set_reg_gpr(reg2, value);
            }, first_halfword, second_halfword),
            Opcode::Ldh => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                let value = (interconnect.read_halfword(addr) as i16) as u32;
                self.set_reg_gpr(reg2, value);
            }, first_halfword, second_halfword),
            Opcode::Ldw | Opcode::Inw => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                let value = interconnect.read_word(addr);
                self.set_reg_gpr(reg2, value);
            }, first_halfword, second_halfword),
            Opcode::Stb | Opcode::Outb => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                let value = self.reg_gpr(reg2) as u8;
                interconnect.write_byte(addr, value);
            }, first_halfword, second_halfword),
            Opcode::Sth | Opcode::Outh => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                let value = self.reg_gpr(reg2) as u16;
                interconnect.write_halfword(addr, value);
            }, first_halfword, second_halfword),
            Opcode::Stw | Opcode::Outw => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                let value = self.reg_gpr(reg2);
                interconnect.write_word(addr, value);
            }, first_halfword, second_halfword),
            Opcode::Inb => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                let value = interconnect.read_byte(addr) as u32;
                self.set_reg_gpr(reg2, value);
            }, first_halfword, second_halfword),
            Opcode::Inh => format_vi(|reg1, reg2, disp16| {
                let addr = self.reg_gpr(reg1).wrapping_add(disp16 as u32);
                let value = interconnect.read_halfword(addr) as u32;
                self.set_reg_gpr(reg2, value);
            }, first_halfword, second_halfword),
        }

        if take_branch {
            let disp9 = first_halfword & 0x01ff;
            let disp = (disp9 as u32) | if disp9 & 0x0100 == 0 { 0x00000000 } else { 0xfffffe00 };
            next_pc = self.reg_pc.wrapping_add(disp);
        }

        self.reg_pc = next_pc;

        interconnect.cycles(opcode.num_cycles(take_branch));
    }

    fn set_zero_sign_flags(&mut self, value: u32) {
        self.psw_zero = value == 0;
        self.psw_sign = value & 0x80000000 != 0;
    }
}

fn format_i<F: FnOnce(usize, usize)>(f: F, first_halfword: u16) {
    let reg1 = (first_halfword & 0x1f) as usize;
    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
    f(reg1, reg2);
}

fn format_ii<F: FnOnce(usize, usize)>(f: F, first_halfword: u16) {
    let imm5 = (first_halfword & 0x1f) as usize;
    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
    f(imm5, reg2);
}

fn format_v<F: FnOnce(usize, usize, u16)>(f: F, first_halfword: u16, second_halfword: u16) {
    let reg1 = (first_halfword & 0x1f) as usize;
    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
    let imm16 = second_halfword;
    f(reg1, reg2, imm16);
}

fn format_vi<F: FnOnce(usize, usize, i16)>(f: F, first_halfword: u16, second_halfword: u16) {
    let reg1 = (first_halfword & 0x1f) as usize;
    let reg2 = ((first_halfword >> 5) & 0x1f) as usize;
    let disp16 = second_halfword as i16;
    f(reg1, reg2, disp16);
}

fn sign_extend_imm5(imm5: usize) -> u32 {
    (imm5 as u32) | (if imm5 & 0x10 == 0 { 0x00000000 } else { 0xffffffe0 })
}