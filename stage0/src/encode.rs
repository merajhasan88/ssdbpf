//! RISC-V (RV64I) instruction encoders: pure functions, no memory, no OS.
//! This module carries over unchanged into the stage-3 JIT and onto the Pico 2.

/// A register number, x0..x31.
pub type Reg = u8;

/// An immediate
pub type Imm = i32;

/// Register x0 (ABI name `zero`).
pub const ZERO: Reg = 0;
/// Register x1 (ABI name `ra`)
pub const RA: Reg = 1;
/// Register x10 (ABI name `a0`)
pub const A0: Reg = 10;
/// Register x11 (ABI name `a1`)
pub const A1: Reg = 11;
/// Register x12 (ABI name `a2`)
pub const A2: Reg = 12;
/// Register x13 (ABI name `a3`)
pub const A3: Reg = 13;
/// Register x15 (ABI name `a5`)
pub const A5: Reg = 15;

#[derive(Debug, PartialEq, Eq)]
pub enum EncodeError {
    /// A register number outside x0..x31.
    BadRegister(Reg),
    /// An immediate that does not fit the instruction's immediate field.
    ImmediateOutOfRange(i64),
    /// A field either out of range or in case of opcode not having right-most bits correct
    FieldOutOfRange(u32, FieldName),
}

/// Field Name
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldName {
    Opcode,
    Funct3,
    Funct7,
}

/// Checks if register is greater than 31
fn check_register(r: Reg) -> Result<u32, EncodeError> {
    let value = u32::from(r);
    if value < 32 {
        Ok(value)
    } else {
        Err(EncodeError::BadRegister(r))
    }
}

/// Checks Immediate range and sends back lowest 12 bits
fn check_immediate(i: Imm) -> Result<u32, EncodeError> {
    if (-2048..=2047).contains(&i) {
        Ok((i as u32) & 0xFFF)
    } else {
        Err(EncodeError::ImmediateOutOfRange(i64::from(i)))
    }
}

/// An opcode, funct3 or funct7 value too big for its field (or an opcode not ending in 11).
fn check_field(f: u32, name: FieldName) -> Result<u32, EncodeError> {
    let check = match name {
        FieldName::Funct7 => f < 128,
        FieldName::Funct3 => f < 8,
        FieldName::Opcode => f < 128 && (f & 0b11 == 0b11),
    };
    if check {
        Ok(f)
    } else {
        Err(EncodeError::FieldOutOfRange(f, name))
    }
}

// ---- Instruction formats (RV32I chapter, "Base Instruction Formats" figure) ----

/// R-type: funct7 | rs2 | rs1 | funct3 | rd | opcode
fn r_type(
    opcode: u32,
    rd: Reg,
    funct3: u32,
    rs1: Reg,
    rs2: Reg,
    funct7: u32,
) -> Result<u32, EncodeError> {
    let unshifted_rs2 = check_register(rs2)?;
    let unshifted_rs1 = check_register(rs1)?;
    let unshifted_rd = check_register(rd)?;
    let unshifted_funct7 = check_field(funct7, FieldName::Funct7)?;
    let unshifted_funct3 = check_field(funct3, FieldName::Funct3)?;
    let sizevalidated_opcode = check_field(opcode, FieldName::Opcode)?;
    Ok(unshifted_funct7 << 25
        | unshifted_rs2 << 20
        | unshifted_rs1 << 15
        | unshifted_funct3 << 12
        | unshifted_rd << 7
        | sizevalidated_opcode)
}

/// I-type: imm[11:0] | rs1 | funct3 | rd | opcode
fn i_type(opcode: u32, rd: Reg, funct3: u32, rs1: Reg, imm: Imm) -> Result<u32, EncodeError> {
    let unshifted_imm = check_immediate(imm)?;
    let unshifted_rs1 = check_register(rs1)?;
    let unshifted_funct3 = check_field(funct3, FieldName::Funct3)?;
    let unshifted_rd = check_register(rd)?;
    let sizevalidated_opcode = check_field(opcode, FieldName::Opcode)?;
    Ok(unshifted_imm << 20
        | unshifted_rs1 << 15
        | unshifted_funct3 << 12
        | unshifted_rd << 7
        | sizevalidated_opcode)
}

// ---- Instructions (opcode/funct3/funct7 values: the manual's instruction listing) ----

pub fn add(rd: Reg, rs1: Reg, rs2: Reg) -> Result<u32, EncodeError> {
    r_type(0b011_0011, rd, 0b000, rs1, rs2, 0b000_0000)
}

pub fn addi(rd: Reg, rs1: Reg, imm: Imm) -> Result<u32, EncodeError> {
    i_type(0b001_0011, rd, 0b000, rs1, imm)
}

pub fn jalr(rd: Reg, rs1: Reg, imm: Imm) -> Result<u32, EncodeError> {
    i_type(0b110_0111, rd, 0b000, rs1, imm)
}

// ---- Pseudo-instructions (asm manual): `mv rd, rs` = `addi rd, rs, 0`; `ret` = `jalr x0, 0(x1)` ----

pub fn mv(rd: Reg, rs: Reg) -> Result<u32, EncodeError> {
    addi(rd, rs, 0)
}

/// If jalr or i_type ever gets a bug, ret() reports an error instead of quietly returning a wrong word
pub fn ret() -> Result<u32, EncodeError> {
    jalr(ZERO, RA, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected words come from:  echo '<asm>' | llvm-mc-14 -triple=riscv64 -show-encoding
    // llvm-mc prints little-endian bytes; the word is those bytes read in reverse order.

    #[test]
    fn test_register_size() {
        assert_eq!(check_register(31), Ok(31));
        assert_eq!(check_register(32), Err(EncodeError::BadRegister(32)));
    }
    #[test]
    fn test_immediate_size() {
        assert_eq!(check_immediate(2047), Ok(2047));
        assert_eq!(check_immediate(-2048), Ok(((-2048_i32) as u32) & 0xFFF));
        assert_eq!(
            check_immediate(-2049),
            Err(EncodeError::ImmediateOutOfRange(-2049))
        );
        assert_eq!(
            check_immediate(2048),
            Err(EncodeError::ImmediateOutOfRange(2048))
        );
    }
    #[test]
    fn field_validation() {
        assert_eq!(
            check_field(128, FieldName::Funct7),
            Err(EncodeError::FieldOutOfRange(128, FieldName::Funct7))
        );
        assert_eq!(check_field(127, FieldName::Funct7), Ok(127));
        assert_eq!(
            check_field(128, FieldName::Opcode),
            Err(EncodeError::FieldOutOfRange(128, FieldName::Opcode))
        );
        assert_eq!(
            check_field(0b111_1110, FieldName::Opcode),
            Err(EncodeError::FieldOutOfRange(0b111_1110, FieldName::Opcode))
        );
        assert_eq!(check_field(127, FieldName::Opcode), Ok(127));
        assert_eq!(
            check_field(8, FieldName::Funct3),
            Err(EncodeError::FieldOutOfRange(8, FieldName::Funct3))
        );
        assert_eq!(check_field(7, FieldName::Funct3), Ok(7));
    }
    #[test]
    fn add_a5_a5_a1() {
        // llvm-mc: [0xb3,0x87,0xb7,0x00]  ->  0x00b7_87b3   (a5 = x15, a1 = x11)
        assert_eq!(add(A5, A5, A1), Ok(0x00b7_87b3));
    }
    #[test]
    fn addi_a5_a5_minus1() {
        assert_eq!(
            addi(A5, A5, -1),
            Ok(0b1111_1111_1111_0111_1000_0111_1001_0011)
        );
    }
    #[test]
    fn mv_a3_a2() {
        assert_eq!(mv(A3, A2), Ok(0b0000_0000_0000_0110_0000_0110_1001_0011));
    }
    #[test]
    fn mv_a5_a0() {
        assert_eq!(mv(A5, A0), Ok(0b0000_0000_0000_0101_0000_0111_1001_0011));
    }
    #[test]
    fn mv_a0_a5() {
        assert_eq!(mv(A0, A5), Ok(0b0000_0000_0000_0111_1000_0101_0001_0011));
    }
    #[test]
    fn test_ret() {
        assert_eq!(ret(), Ok(0x0000_8067));
    }
    #[test]
    fn addi_a0_a0_2047() {
        assert_eq!(addi(A0, A0, 2047), Ok(0x7ff50513));
    }
    #[test]
    fn addi_a0_a0_minus2048() {
        assert_eq!(addi(A0, A0, -2048), Ok(0x80050513));
    }
    #[test]
    fn addi_immediate_2048() {
        assert_eq!(
            addi(A0, A0, 2048),
            Err(EncodeError::ImmediateOutOfRange(2048))
        );
    }
    #[test]
    fn addi_immediate_minus2049() {
        assert_eq!(
            addi(A0, A0, -2049),
            Err(EncodeError::ImmediateOutOfRange(-2049))
        );
    }
    #[test]
    fn addi_register_test_rd() {
        assert_eq!(addi(32, A0, 0), Err(EncodeError::BadRegister(32)));
    }
    #[test]
    fn add_register_test_rd() {
        assert_eq!(add(32, A0, A1), Err(EncodeError::BadRegister(32)));
    }
    #[test]
    fn jalr_register_test_rd() {
        assert_eq!(jalr(32, A0, 10), Err(EncodeError::BadRegister(32)));
    }

    #[test]
    fn addi_register_test_rs1() {
        assert_eq!(addi(A0, 32, 0), Err(EncodeError::BadRegister(32)));
    }
    #[test]
    fn add_register_test_rs1() {
        assert_eq!(add(A0, 32, A1), Err(EncodeError::BadRegister(32)));
    }
    #[test]
    fn add_register_test_rs2() {
        assert_eq!(add(A0, A1, 32), Err(EncodeError::BadRegister(32)));
    }
    #[test]
    fn jalr_register_test_rs1() {
        assert_eq!(jalr(A0, 32, 10), Err(EncodeError::BadRegister(32)));
    }
}
