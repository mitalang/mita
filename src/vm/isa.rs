pub type Inst = u32;

// Opcode field (bits 31:24)
pub const OP_NOP:     u8 = 0x00;
pub const OP_MV:      u8 = 0x01;
pub const OP_LI:      u8 = 0x02;
pub const OP_ADD:     u8 = 0x03;
pub const OP_SUB:     u8 = 0x04;
pub const OP_MUL:     u8 = 0x05;
pub const OP_DIV:     u8 = 0x06;
pub const OP_REM:     u8 = 0x07;
pub const OP_ADDI:    u8 = 0x08;
pub const OP_SEQ:     u8 = 0x09;
pub const OP_SNE:     u8 = 0x0A;
pub const OP_SLT:     u8 = 0x0B;
pub const OP_SGT:     u8 = 0x0C;
pub const OP_SLE:     u8 = 0x0D;
pub const OP_SGE:     u8 = 0x0E;
pub const OP_CONS:    u8 = 0x0F;
pub const OP_CAR:     u8 = 0x10;
pub const OP_CDR:     u8 = 0x11;
pub const OP_LW:      u8 = 0x12;
pub const OP_SW:      u8 = 0x13;
pub const OP_BEQ:     u8 = 0x14;
pub const OP_BNE:     u8 = 0x15;
pub const OP_BLT:     u8 = 0x16;
pub const OP_BGE:     u8 = 0x17;
pub const OP_JAL:     u8 = 0x18;
pub const OP_J:       u8 = 0x19;
pub const OP_CALL:    u8 = 0x1A;
pub const OP_TAIL:    u8 = 0x1B;
pub const OP_RET:     u8 = 0x1C;
pub const OP_BUILTIN: u8 = 0x1D;
pub const OP_SETGLOBAL: u8 = 0x1E;

// Register indices (RISC-V style)
pub const X0: u8 = 0;   // zero
pub const X1: u8 = 1;   // ra
pub const X2: u8 = 2;   // sp
pub const X3: u8 = 3;   // t0
pub const X4: u8 = 4;   // t1
pub const X5: u8 = 5;   // t2
pub const X6: u8 = 6;   // t3
pub const X7: u8 = 7;   // t4
pub const X8: u8 = 8;   // s0
pub const X9: u8 = 9;   // s1
pub const X10: u8 = 10; // a0
pub const X11: u8 = 11; // a1
pub const X12: u8 = 12; // a2
pub const X13: u8 = 13; // a3
pub const X14: u8 = 14; // a4
pub const X15: u8 = 15; // a5
pub const X16: u8 = 16; // a6
pub const X17: u8 = 17; // a7
pub const X18: u8 = 18; // s2
pub const X19: u8 = 19; // s3
pub const X20: u8 = 20; // s4
pub const X21: u8 = 21; // s5
pub const X22: u8 = 22; // s6
pub const X23: u8 = 23; // s7
pub const X24: u8 = 24; // s8
pub const X25: u8 = 25; // s9
pub const X26: u8 = 26; // s10
pub const X27: u8 = 27; // s11
pub const X28: u8 = 28; // t5
pub const X29: u8 = 29; // t6
pub const X30: u8 = 30; // t7
pub const X31: u8 = 31; // t8

// Encoding helpers
// R-type: [ opcode(8) | rd(6) | rs1(6) | rs2(6) | funct3(6) ]
pub fn encode_r(op: u8, rd: u8, rs1: u8, rs2: u8, funct3: u8) -> Inst {
    ((op as u32) << 24) | ((rd as u32) << 18) | ((rs1 as u32) << 12) | ((rs2 as u32) << 6) | (funct3 as u32)
}

// I-type: [ opcode(8) | rd(6) | rs1(6) | imm(12) ]
pub fn encode_i(op: u8, rd: u8, rs1: u8, imm: i16) -> Inst {
    let imm12 = (imm as u16) & 0x0FFF;
    ((op as u32) << 24) | ((rd as u32) << 18) | ((rs1 as u32) << 12) | (imm12 as u32)
}

// S-type: [ opcode(8) | rs2(6) | rs1(6) | imm(12) ]
pub fn encode_s(op: u8, rs2: u8, rs1: u8, imm: i16) -> Inst {
    let imm12 = (imm as u16) & 0x0FFF;
    ((op as u32) << 24) | ((rs2 as u32) << 18) | ((rs1 as u32) << 12) | (imm12 as u32)
}

// B-type: [ opcode(8) | rs1(6) | rs2(6) | imm(12) ]
pub fn encode_b(op: u8, rs1: u8, rs2: u8, offset: i16) -> Inst {
    let imm12 = (offset as u16) & 0x0FFF;
    ((op as u32) << 24) | ((rs1 as u32) << 18) | ((rs2 as u32) << 12) | (imm12 as u32)
}

// J-type: [ opcode(8) | rd(6) | imm(18) ]
pub fn encode_j(op: u8, rd: u8, offset: i32) -> Inst {
    let imm18 = (offset as u32) & 0x0003FFFF;
    ((op as u32) << 24) | ((rd as u32) << 18) | imm18
}

// U-type: [ opcode(8) | rd(6) | imm(18) ]
pub fn encode_u(op: u8, rd: u8, idx: u32) -> Inst {
    let imm18 = idx & 0x0003FFFF;
    ((op as u32) << 24) | ((rd as u32) << 18) | imm18
}

// Decoding helpers
pub fn decode_op(inst: Inst) -> u8 {
    ((inst >> 24) & 0xFF) as u8
}

pub fn decode_rd(inst: Inst) -> u8 {
    ((inst >> 18) & 0x3F) as u8
}

pub fn decode_rs1(inst: Inst) -> u8 {
    ((inst >> 12) & 0x3F) as u8
}

pub fn decode_rs2(inst: Inst) -> u8 {
    ((inst >> 6) & 0x3F) as u8
}

pub fn decode_funct3(inst: Inst) -> u8 {
    (inst & 0x3F) as u8
}

pub fn decode_b_rs1(inst: Inst) -> u8 {
    decode_rd(inst)
}

pub fn decode_b_rs2(inst: Inst) -> u8 {
    decode_rs1(inst)
}

pub fn decode_imm12(inst: Inst) -> i16 {
    let raw = (inst & 0x0FFF) as u16;
    if raw & 0x0800 != 0 {
        (raw | 0xF000) as i16
    } else {
        raw as i16
    }
}

pub fn decode_imm18(inst: Inst) -> i32 {
    let raw = inst & 0x0003FFFF;
    if raw & 0x00020000 != 0 {
        (raw | 0xFFFC0000) as i32
    } else {
        raw as i32
    }
}

// Pretty-print instruction
pub fn disasm(inst: Inst, _pc: usize, const_pool: &[crate::vm::value::Value]) -> String {
    let op = decode_op(inst);
    match op {
        OP_NOP => "NOP".to_string(),
        OP_MV => format!("MV x{}, x{}", decode_rd(inst), decode_rs1(inst)),
        OP_LI => {
            let idx = decode_imm18(inst) as usize;
            let val = if idx < const_pool.len() { format!("{}", const_pool[idx]) } else { "?".to_string() };
            format!("LI x{}, {} ; {}", decode_rd(inst), idx, val)
        }
        OP_ADD => format!("ADD x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_SUB => format!("SUB x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_MUL => format!("MUL x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_DIV => format!("DIV x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_REM => format!("REM x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_ADDI => format!("ADDI x{}, x{}, {}", decode_rd(inst), decode_rs1(inst), decode_imm12(inst)),
        OP_SEQ => format!("SEQ x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_SNE => format!("SNE x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_SLT => format!("SLT x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_SGT => format!("SGT x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_SLE => format!("SLE x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_SGE => format!("SGE x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_CONS => format!("CONS x{}, x{}, x{}", decode_rd(inst), decode_rs1(inst), decode_rs2(inst)),
        OP_CAR => format!("CAR x{}, x{}", decode_rd(inst), decode_rs1(inst)),
        OP_CDR => format!("CDR x{}, x{}", decode_rd(inst), decode_rs1(inst)),
        OP_LW => format!("LW x{}, {}(x{})", decode_rd(inst), decode_imm12(inst), decode_rs1(inst)),
        OP_SW => format!("SW x{}, {}(x{})", decode_rs2(inst), decode_imm12(inst), decode_rs1(inst)),
        OP_BEQ => format!("BEQ x{}, x{}, {}", decode_b_rs1(inst), decode_b_rs2(inst), decode_imm12(inst)),
        OP_BNE => format!("BNE x{}, x{}, {}", decode_b_rs1(inst), decode_b_rs2(inst), decode_imm12(inst)),
        OP_BLT => format!("BLT x{}, x{}, {}", decode_b_rs1(inst), decode_b_rs2(inst), decode_imm12(inst)),
        OP_BGE => format!("BGE x{}, x{}, {}", decode_b_rs1(inst), decode_b_rs2(inst), decode_imm12(inst)),
        OP_JAL => format!("JAL x{}, {}", decode_rd(inst), decode_imm18(inst)),
        OP_J => format!("J {}", decode_imm18(inst)),
        OP_CALL => format!("CALL x{}, x{}, {}", decode_rd(inst), decode_rs1(inst), decode_funct3(inst)),
        OP_TAIL => format!("TAIL x{}, {}", decode_rs1(inst), decode_funct3(inst)),
        OP_RET => format!("RET x{}", decode_rs1(inst)),
        OP_BUILTIN => format!("BUILTIN x{}, {}", decode_rd(inst), decode_imm18(inst)),
        OP_SETGLOBAL => {
            let idx = decode_imm18(inst) as usize;
            let name = if idx < const_pool.len() { format!("{}", const_pool[idx]) } else { "?".to_string() };
            format!("SETGLOBAL x{}, {} ; {}", decode_rd(inst), idx, name)
        }
        _ => format!(".word 0x{:08X}", inst),
    }
}
