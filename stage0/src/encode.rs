//! RISC-V (RV64I) instruction encoders: pure functions, no memory, no OS.
//! This module carries over unchanged into the stage-3 JIT and onto the Pico 2.

/// A register number, x0..x31.
pub type Reg = u8;

/// Register x0 (ABI name `zero`).
pub const ZERO: Reg = 0;
/// Register x1 (ABI name `ra`)
pub const RA: Reg = 1;
/// Register x10 (ABI name `a0`)
pub const A0: Reg = 10;
/// Register x11 (ABI name `a1`)
pub const A1: Reg = 11;
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
fn i_type(opcode: u32, rd: Reg, funct3: u32, rs1: Reg, imm: i32) -> Result<u32, EncodeError> {
    todo!()
}

// ---- Instructions (opcode/funct3/funct7 values: the manual's instruction listing) ----

pub fn add(rd: Reg, rs1: Reg, rs2: Reg) -> Result<u32, EncodeError> {
    r_type(0b011_0011, rd, 0b000, rs1, rs2, 0b000_0000)
}

pub fn addi(rd: Reg, rs1: Reg, imm: i32) -> Result<u32, EncodeError> {
    todo!()
}

pub fn jalr(rd: Reg, rs1: Reg, imm: i32) -> Result<u32, EncodeError> {
    todo!()
}

// ---- Pseudo-instructions (asm manual): `mv rd, rs` = `addi rd, rs, 0`; `ret` = `jalr x0, 0(x1)` ----

pub fn mv(rd: Reg, rs: Reg) -> Result<u32, EncodeError> {
    todo!()
}

/// Can `ret` ever fail? Decide, and pick the return type to match.
pub fn ret() -> Result<u32, EncodeError> {
    todo!()
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

    // TODO: mv a5, a0   |   mv a0, a5   |   ret   |   addi a5, a5, -1
    // TODO: error cases: a register number above 31, and immediates just outside the 12-bit signed range
}
