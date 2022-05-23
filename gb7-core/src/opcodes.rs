use std::collections::HashMap;

use crate::cpu::{CpuFlags, Register, WideRegister};

use lazy_static::lazy_static;

pub enum Opcode {
    ADC(Register), // Add register value and carry to A
    ADCHL,         // Add deferenced [HL] and carry to A
    ADCI,          // Add immediate u8 and carry to A

    ADD(Register),        // Add register value to A
    ADDHL,                // Add dereferenced [HL] to A
    ADDI,                 // Add immediate u8 to A
    ADDHLR(WideRegister), // Add wide register value to HL
    ADDSP,                // Add immediate signed byte to SP

    AND(Register), // Bitwise AND between register value and A
    ANDHL,         // Bitwise AND between dererenced [HL] and A
    ANDI,          // Bitwise AND between immediate u8 and A

    BIT(u8, Register), // Test bit in register
    BITHL(u8),         // Test bit in dereferenced [HL]

    CALL,              // Call immediate u16
    CALLCC(CpuFlags),  // Call if condition met
    CALLNCC(CpuFlags), // Call if condition not met

    CB, // Read another opcode and perform the CB variant

    CCF, // Complement Carry Flag

    CP(Register), // Compare register value with A
    CPHL,         // Compare dereferenced [HL] with A
    CPI,          // Compare immediate u8 with A

    CPL, // Complement register A

    DAA, // Decimal Adjust Accumulator

    DEC(Register),      // Decrement register value
    DECHL,              // Decrement dereferenced [HL] value
    DECW(WideRegister), // Decrement wide register value

    DI, // Disable interrupts

    EI, // Enable interrupts

    HALT, // Halt CPU

    INC(Register),      // Increment register value
    INCHL,              // Increment dereferenced [HL] value
    INCW(WideRegister), // Increment wide register value

    JP,              // Jump to immediate u16
    JPCC(CpuFlags),  // Jump if condition met
    JPNCC(CpuFlags), // Jump if condition not met
    JPHL,            // Jump to address in HL

    JR,              // Relative jump with immediate signed byte
    JRCC(CpuFlags),  // Relative jump if condition met
    JRNCC(CpuFlags), // Relative jump if condition not met

    LDRR(Register, Register), // Load right register into left register
    LDRI(Register),           // Load immediate u8 into register
    LDWRI(WideRegister),      // Load immediate u16 into wide register
    LDHLR(Register),          // Load register into dereferenced [HL]
    LDHLI,                    // Load immediate u8 into dereferenced [HL]
    LDRHL(Register),          // Load deferenced [HL] into register
    LDWRA(WideRegister),      // Store register A into address [WR]
    LDIWA,                    // Store register A into address from immediate u16
    LDAWR(WideRegister),      // Store value from [WR] into register A
    LDAIW,                    // Store dereferenced immediate u16 into register A
    LDHLIA,                   // Store register A into dereferenced [HL] and increment HL
    LDHLDA,                   // Store register A into dereferenced [HL] and decrement HL
    LDAHLD,                   // Store dereferenced [HL] into A and decrement HL
    LDAHLI,                   // Store dereferenced [HL] into A and increment HL
    LDISP,                    // Load immediate u16 into SP
    LDHLSP,                   // Add immediate signed byte to SP and store in HL
    LDSPHL,                   // Load register HL into register SP
    LDIOA,                    // Load A into address 0xFF00 + immediate u8
    LDIOCA,                   // Load A into address 0xFF00 + register C
    LDAIO,                    // Load address 0xFF00 + immediate u8 into register A
    LDAIOC,                   // Load address 0xFF00 + register C into register A

    NOP, // No operation

    OR(Register), // Store into A the bitwise OR of register and A
    ORHL,         // Store into A the bitwise OR of A and dereferenced [HL]
    ORI,          // Store into A the bitwise OR of A and immediate u8

    POPWR(WideRegister),  // Pop wide register from stack
    PUSHWR(WideRegister), // Push wide register onto stack

    RES(u8, Register), // Reset bit of register
    RESHL(u8),         // Reset bit of dereferenced [HL]

    RET,              // Return from subroutine
    RETCC(CpuFlags),  // Return if condition met
    RETNCC(CpuFlags), // Return if condition not met
    RETI,             // Return from subroutine and enable interrupts

    RL(Register), // Rotate bits left in register through carry
    RLHL,         // Rotate bits left in deferenced [HL] through carry
    RLA,          // Rotate bits left in register A

    RLC(Register), // Rotate bits left in register
    RLCHL,         // Rotate bits left in dereferenced [HL]
    RLCA,          // Rotate bits left in register A

    RR(Register), // Rotate bits right in register through carry
    RRHL,         // Rotate bits right in deferenced [HL] through carry
    RRA,          // Rotate bits right in register A

    RRC(Register), // Rotate bits right in register
    RRCHL,         // Rotate bits right in dereferenced [HL]
    RRCA,          // Rotate bits right in register A

    RST(u16), // Call reset vector

    SBC(Register), // Subtract register value and carry from A
    SBCHL,         // Subtract dereferenced [HL] and carry from A
    SBCI,          // Subtract immediate u8 and carry from A

    SCF, // Set carry flag

    SET(u8, Register), // Set bit in register value
    SETHL(u8),         // Set bit in dereferenced [HL] value

    SLA(Register), // Arithmetic shift left register value
    SLAHL,         // Arithmetic shift left dereferenced [HL] value

    SRA(Register), // Arithmetic shift right register value
    SRAHL,         // Arithmetic shift right dereferenced [HL] value

    SRL(Register), // Logical shift right register value
    SRLHL,         // Logical shift right dereferenced [HL] value

    STOP, // Stop CPU

    SUB(Register), // Subtract register value from A
    SUBHL,         // Subtract dereferenced [HL] value from A
    SUBI,          // Subtract immediate u8 from A

    SWAP(Register), // Swap upper 4 bits and lower 4 bits of register value
    SWAPHL,         // Swap operation on bits of dereferenced [HL] value

    XOR(Register), // XOR A with register value
    XORHL,         // XOR A with dereferenced [HL]
    XORI,          // XOR A with immediate u8
}

lazy_static! {
    pub static ref OPCODES: HashMap<u8, Opcode> = HashMap::from([
        (0x00, Opcode::NOP),
        (0x01, Opcode::LDWRI(WideRegister::BC)),
        (0x02, Opcode::LDWRA(WideRegister::BC)),
        (0x03, Opcode::INCW(WideRegister::BC)),
        (0x04, Opcode::INC(Register::B)),
        (0x05, Opcode::DEC(Register::B)),
        (0x06, Opcode::LDRI(Register::B)),
        (0x07, Opcode::RLCA),
        (0x08, Opcode::LDISP),
        (0x09, Opcode::ADDHLR(WideRegister::BC)),
        (0x0A, Opcode::LDAWR(WideRegister::BC)),
        (0x0B, Opcode::DECW(WideRegister::BC)),
        (0x0C, Opcode::INC(Register::C)),
        (0x0D, Opcode::DEC(Register::C)),
        (0x0E, Opcode::LDRI(Register::C)),
        (0x0F, Opcode::RRCA),
        (0x10, Opcode::STOP),
        (0x11, Opcode::LDWRI(WideRegister::DE)),
        (0x12, Opcode::LDWRA(WideRegister::DE)),
        (0x13, Opcode::INCW(WideRegister::DE)),
        (0x14, Opcode::INC(Register::D)),
        (0x15, Opcode::DEC(Register::D)),
        (0x16, Opcode::LDRI(Register::D)),
        (0x17, Opcode::RLA),
        (0x18, Opcode::JR),
        (0x19, Opcode::ADDHLR(WideRegister::DE)),
        (0x1A, Opcode::LDAWR(WideRegister::DE)),
        (0x1B, Opcode::DECW(WideRegister::DE)),
        (0x1C, Opcode::INC(Register::E)),
        (0x1D, Opcode::DEC(Register::E)),
        (0x1E, Opcode::LDRI(Register::E)),
        (0x1F, Opcode::RRA),
        (0x20, Opcode::JRNCC(CpuFlags::Z)),
        (0x21, Opcode::LDWRI(WideRegister::HL)),
        (0x22, Opcode::LDHLIA),
        (0x23, Opcode::INCW(WideRegister::HL)),
        (0x24, Opcode::INC(Register::H)),
        (0x25, Opcode::DEC(Register::H)),
        (0x26, Opcode::LDRI(Register::H)),
        (0x27, Opcode::DAA),
        (0x28, Opcode::JRCC(CpuFlags::Z)),
        (0x29, Opcode::ADDHLR(WideRegister::HL)),
        (0x2A, Opcode::LDAHLI),
        (0x2B, Opcode::DECW(WideRegister::HL)),
        (0x2C, Opcode::INC(Register::L)),
        (0x2D, Opcode::DEC(Register::L)),
        (0x2E, Opcode::LDRI(Register::L)),
        (0x2F, Opcode::CPL),
        (0x30, Opcode::JRNCC(CpuFlags::C)),
        (0x31, Opcode::LDWRI(WideRegister::SP)),
        (0x32, Opcode::LDHLDA),
        (0x33, Opcode::INCW(WideRegister::SP)),
        (0x34, Opcode::INCHL),
        (0x35, Opcode::DECHL),
        (0x36, Opcode::LDHLI),
        (0x37, Opcode::SCF),
        (0x38, Opcode::JRCC(CpuFlags::C)),
        (0x39, Opcode::ADDHLR(WideRegister::SP)),
        (0x3A, Opcode::LDAHLD),
        (0x3B, Opcode::DECW(WideRegister::SP)),
        (0x3C, Opcode::INC(Register::A)),
        (0x3D, Opcode::DEC(Register::A)),
        (0x3E, Opcode::LDRI(Register::A)),
        (0x3F, Opcode::CCF),
        (0x40, Opcode::LDRR(Register::B, Register::B)),
        (0x41, Opcode::LDRR(Register::B, Register::C)),
        (0x42, Opcode::LDRR(Register::B, Register::D)),
        (0x43, Opcode::LDRR(Register::B, Register::E)),
        (0x44, Opcode::LDRR(Register::B, Register::H)),
        (0x45, Opcode::LDRR(Register::B, Register::L)),
        (0x46, Opcode::LDRHL(Register::B)),
        (0x47, Opcode::LDRR(Register::B, Register::A)),
        (0x48, Opcode::LDRR(Register::C, Register::B)),
        (0x49, Opcode::LDRR(Register::C, Register::C)),
        (0x4A, Opcode::LDRR(Register::C, Register::D)),
        (0x4B, Opcode::LDRR(Register::C, Register::E)),
        (0x4C, Opcode::LDRR(Register::C, Register::H)),
        (0x4D, Opcode::LDRR(Register::C, Register::L)),
        (0x4E, Opcode::LDRHL(Register::C)),
        (0x4F, Opcode::LDRR(Register::C, Register::A)),
        (0x50, Opcode::LDRR(Register::D, Register::B)),
        (0x51, Opcode::LDRR(Register::D, Register::C)),
        (0x52, Opcode::LDRR(Register::D, Register::D)),
        (0x53, Opcode::LDRR(Register::D, Register::E)),
        (0x54, Opcode::LDRR(Register::D, Register::H)),
        (0x55, Opcode::LDRR(Register::D, Register::L)),
        (0x56, Opcode::LDRHL(Register::D)),
        (0x57, Opcode::LDRR(Register::D, Register::A)),
        (0x58, Opcode::LDRR(Register::E, Register::B)),
        (0x59, Opcode::LDRR(Register::E, Register::C)),
        (0x5A, Opcode::LDRR(Register::E, Register::D)),
        (0x5B, Opcode::LDRR(Register::E, Register::E)),
        (0x5C, Opcode::LDRR(Register::E, Register::H)),
        (0x5D, Opcode::LDRR(Register::E, Register::L)),
        (0x5E, Opcode::LDRHL(Register::E)),
        (0x5F, Opcode::LDRR(Register::E, Register::A)),
        (0x60, Opcode::LDRR(Register::H, Register::B)),
        (0x61, Opcode::LDRR(Register::H, Register::C)),
        (0x62, Opcode::LDRR(Register::H, Register::D)),
        (0x63, Opcode::LDRR(Register::H, Register::E)),
        (0x64, Opcode::LDRR(Register::H, Register::H)),
        (0x65, Opcode::LDRR(Register::H, Register::L)),
        (0x66, Opcode::LDRHL(Register::H)),
        (0x67, Opcode::LDRR(Register::H, Register::A)),
        (0x68, Opcode::LDRR(Register::L, Register::B)),
        (0x69, Opcode::LDRR(Register::L, Register::C)),
        (0x6A, Opcode::LDRR(Register::L, Register::D)),
        (0x6B, Opcode::LDRR(Register::L, Register::E)),
        (0x6C, Opcode::LDRR(Register::L, Register::H)),
        (0x6D, Opcode::LDRR(Register::L, Register::L)),
        (0x6E, Opcode::LDRHL(Register::L)),
        (0x6F, Opcode::LDRR(Register::L, Register::A)),
        (0x70, Opcode::LDHLR(Register::B)),
        (0x71, Opcode::LDHLR(Register::C)),
        (0x72, Opcode::LDHLR(Register::D)),
        (0x73, Opcode::LDHLR(Register::E)),
        (0x74, Opcode::LDHLR(Register::H)),
        (0x75, Opcode::LDHLR(Register::L)),
        (0x76, Opcode::HALT),
        (0x77, Opcode::LDHLR(Register::A)),
        (0x78, Opcode::LDRR(Register::A, Register::B)),
        (0x79, Opcode::LDRR(Register::A, Register::C)),
        (0x7A, Opcode::LDRR(Register::A, Register::D)),
        (0x7B, Opcode::LDRR(Register::A, Register::E)),
        (0x7C, Opcode::LDRR(Register::A, Register::H)),
        (0x7D, Opcode::LDRR(Register::A, Register::L)),
        (0x7E, Opcode::LDRHL(Register::A)),
        (0x7F, Opcode::LDRR(Register::A, Register::A)),
        (0x80, Opcode::ADD(Register::B)),
        (0x81, Opcode::ADD(Register::C)),
        (0x82, Opcode::ADD(Register::D)),
        (0x83, Opcode::ADD(Register::E)),
        (0x84, Opcode::ADD(Register::H)),
        (0x85, Opcode::ADD(Register::L)),
        (0x86, Opcode::ADDHL),
        (0x87, Opcode::ADD(Register::A)),
        (0x88, Opcode::ADC(Register::B)),
        (0x89, Opcode::ADC(Register::C)),
        (0x8A, Opcode::ADC(Register::D)),
        (0x8B, Opcode::ADC(Register::E)),
        (0x8C, Opcode::ADC(Register::H)),
        (0x8D, Opcode::ADC(Register::L)),
        (0x8E, Opcode::ADCHL),
        (0x8F, Opcode::ADC(Register::A)),
        (0x90, Opcode::SUB(Register::B)),
        (0x91, Opcode::SUB(Register::C)),
        (0x92, Opcode::SUB(Register::D)),
        (0x93, Opcode::SUB(Register::E)),
        (0x94, Opcode::SUB(Register::H)),
        (0x95, Opcode::SUB(Register::L)),
        (0x96, Opcode::SUBHL),
        (0x97, Opcode::SUB(Register::A)),
        (0x98, Opcode::SBC(Register::B)),
        (0x99, Opcode::SBC(Register::C)),
        (0x9A, Opcode::SBC(Register::D)),
        (0x9B, Opcode::SBC(Register::E)),
        (0x9C, Opcode::SBC(Register::H)),
        (0x9D, Opcode::SBC(Register::L)),
        (0x9E, Opcode::SBCHL),
        (0x9F, Opcode::SBC(Register::A)),
        (0xA0, Opcode::AND(Register::B)),
        (0xA1, Opcode::AND(Register::C)),
        (0xA2, Opcode::AND(Register::D)),
        (0xA3, Opcode::AND(Register::E)),
        (0xA4, Opcode::AND(Register::H)),
        (0xA5, Opcode::AND(Register::L)),
        (0xA6, Opcode::ANDHL),
        (0xA7, Opcode::AND(Register::A)),
        (0xA8, Opcode::XOR(Register::B)),
        (0xA9, Opcode::XOR(Register::C)),
        (0xAA, Opcode::XOR(Register::D)),
        (0xAB, Opcode::XOR(Register::E)),
        (0xAC, Opcode::XOR(Register::H)),
        (0xAD, Opcode::XOR(Register::L)),
        (0xAE, Opcode::XORHL),
        (0xAF, Opcode::XOR(Register::A)),
        (0xB0, Opcode::OR(Register::B)),
        (0xB1, Opcode::OR(Register::C)),
        (0xB2, Opcode::OR(Register::D)),
        (0xB3, Opcode::OR(Register::E)),
        (0xB4, Opcode::OR(Register::H)),
        (0xB5, Opcode::OR(Register::L)),
        (0xB6, Opcode::ORHL),
        (0xB7, Opcode::OR(Register::A)),
        (0xB8, Opcode::CP(Register::B)),
        (0xB9, Opcode::CP(Register::C)),
        (0xBA, Opcode::CP(Register::D)),
        (0xBB, Opcode::CP(Register::E)),
        (0xBC, Opcode::CP(Register::H)),
        (0xBD, Opcode::CP(Register::L)),
        (0xBE, Opcode::CPHL),
        (0xBF, Opcode::CP(Register::A)),
        (0xC0, Opcode::RETNCC(CpuFlags::Z)),
        (0xC1, Opcode::POPWR(WideRegister::BC)),
        (0xC2, Opcode::JPNCC(CpuFlags::Z)),
        (0xC3, Opcode::JP),
        (0xC4, Opcode::CALLNCC(CpuFlags::Z)),
        (0xC5, Opcode::PUSHWR(WideRegister::BC)),
        (0xC6, Opcode::ADDI),
        (0xC7, Opcode::RST(0x00)),
        (0xC8, Opcode::RETCC(CpuFlags::Z)),
        (0xC9, Opcode::RET),
        (0xCA, Opcode::JPCC(CpuFlags::Z)),
        (0xCB, Opcode::CB),
        (0xCC, Opcode::CALLCC(CpuFlags::Z)),
        (0xCD, Opcode::CALL),
        (0xCE, Opcode::ADCI),
        (0xCF, Opcode::RST(0x08)),
        (0xD0, Opcode::RETNCC(CpuFlags::C)),
        (0xD1, Opcode::POPWR(WideRegister::DE)),
        (0xD2, Opcode::JPNCC(CpuFlags::C)),
        // 0xD3
        (0xD4, Opcode::CALLNCC(CpuFlags::C)),
        (0xD5, Opcode::PUSHWR(WideRegister::DE)),
        (0xD6, Opcode::SUBI),
        (0xD7, Opcode::RST(0x10)),
        (0xD8, Opcode::RETCC(CpuFlags::C)),
        (0xD9, Opcode::RETI),
        (0xDA, Opcode::JPCC(CpuFlags::C)),
        // 0xDB
        (0xDC, Opcode::CALLCC(CpuFlags::C)),
        // 0xDD
        (0xDE, Opcode::SBCI),
        (0xDF, Opcode::RST(0x18)),
        (0xE0, Opcode::LDIOA),
        (0xE1, Opcode::POPWR(WideRegister::HL)),
        (0xE2, Opcode::LDIOCA),
        // 0xE3
        // 0xE4
        (0xE5, Opcode::PUSHWR(WideRegister::HL)),
        (0xE6, Opcode::ANDI),
        (0xE7, Opcode::RST(0x20)),
        (0xE8, Opcode::ADDSP),
        (0xE9, Opcode::JPHL),
        (0xEA, Opcode::LDIWA),
        // 0xEB
        // 0xEC
        // 0xED
        (0xEE, Opcode::XORI),
        (0xEF, Opcode::RST(0x28)),
        (0xF0, Opcode::LDAIO),
        (0xF1, Opcode::POPWR(WideRegister::AF)),
        (0xF2, Opcode::LDAIOC),
        (0xF3, Opcode::DI),
        // 0xF4
        (0xF5, Opcode::PUSHWR(WideRegister::AF)),
        (0xF6, Opcode::ORI),
        (0xF7, Opcode::RST(0x30)),
        (0xF8, Opcode::LDHLSP),
        (0xF9, Opcode::LDSPHL),
        (0xFA, Opcode::LDAIW),
        (0xFB, Opcode::EI),
        // 0xFC
        // 0xFD
        (0xFE, Opcode::CPI),
        (0xFF, Opcode::RST(0x38)),
    ]);

    pub static ref CB_OPCODES: HashMap<u8, Opcode> = HashMap::from([
        (0x00, Opcode::RLC(Register::B)),
        (0x01, Opcode::RLC(Register::C)),
        (0x02, Opcode::RLC(Register::D)),
        (0x03, Opcode::RLC(Register::E)),
        (0x04, Opcode::RLC(Register::H)),
        (0x05, Opcode::RLC(Register::L)),
        (0x06, Opcode::RLCHL),
        (0x07, Opcode::RLC(Register::A)),
        (0x08, Opcode::RRC(Register::B)),
        (0x09, Opcode::RRC(Register::C)),
        (0x0A, Opcode::RRC(Register::D)),
        (0x0B, Opcode::RRC(Register::E)),
        (0x0C, Opcode::RRC(Register::H)),
        (0x0D, Opcode::RRC(Register::L)),
        (0x0E, Opcode::RRCHL),
        (0x0F, Opcode::RRC(Register::A)),
        (0x10, Opcode::RL(Register::B)),
        (0x11, Opcode::RL(Register::C)),
        (0x12, Opcode::RL(Register::D)),
        (0x13, Opcode::RL(Register::E)),
        (0x14, Opcode::RL(Register::H)),
        (0x15, Opcode::RL(Register::L)),
        (0x16, Opcode::RLHL),
        (0x17, Opcode::RL(Register::A)),
        (0x18, Opcode::RR(Register::B)),
        (0x19, Opcode::RR(Register::C)),
        (0x1A, Opcode::RR(Register::D)),
        (0x1B, Opcode::RR(Register::E)),
        (0x1C, Opcode::RR(Register::H)),
        (0x1D, Opcode::RR(Register::L)),
        (0x1E, Opcode::RRHL),
        (0x1F, Opcode::RR(Register::A)),
        (0x20, Opcode::SLA(Register::B)),
        (0x21, Opcode::SLA(Register::C)),
        (0x22, Opcode::SLA(Register::D)),
        (0x23, Opcode::SLA(Register::E)),
        (0x24, Opcode::SLA(Register::H)),
        (0x25, Opcode::SLA(Register::L)),
        (0x26, Opcode::SLAHL),
        (0x27, Opcode::SLA(Register::A)),
        (0x28, Opcode::SRA(Register::B)),
        (0x29, Opcode::SRA(Register::C)),
        (0x2A, Opcode::SRA(Register::D)),
        (0x2B, Opcode::SRA(Register::E)),
        (0x2C, Opcode::SRA(Register::H)),
        (0x2D, Opcode::SRA(Register::L)),
        (0x2E, Opcode::SRAHL),
        (0x2F, Opcode::SRA(Register::A)),
        (0x30, Opcode::SWAP(Register::B)),
        (0x31, Opcode::SWAP(Register::C)),
        (0x32, Opcode::SWAP(Register::D)),
        (0x33, Opcode::SWAP(Register::E)),
        (0x34, Opcode::SWAP(Register::H)),
        (0x35, Opcode::SWAP(Register::L)),
        (0x36, Opcode::SWAPHL),
        (0x37, Opcode::SWAP(Register::A)),
        (0x38, Opcode::SRL(Register::B)),
        (0x39, Opcode::SRL(Register::C)),
        (0x3A, Opcode::SRL(Register::D)),
        (0x3B, Opcode::SRL(Register::E)),
        (0x3C, Opcode::SRL(Register::H)),
        (0x3D, Opcode::SRL(Register::L)),
        (0x3E, Opcode::SRLHL),
        (0x3F, Opcode::SRL(Register::A)),
        (0x40, Opcode::BIT(0, Register::B)),
        (0x41, Opcode::BIT(0, Register::C)),
        (0x42, Opcode::BIT(0, Register::D)),
        (0x43, Opcode::BIT(0, Register::E)),
        (0x44, Opcode::BIT(0, Register::H)),
        (0x45, Opcode::BIT(0, Register::L)),
        (0x46, Opcode::BITHL(0)),
        (0x47, Opcode::BIT(0, Register::A)),
        (0x48, Opcode::BIT(1, Register::B)),
        (0x49, Opcode::BIT(1, Register::C)),
        (0x4A, Opcode::BIT(1, Register::D)),
        (0x4B, Opcode::BIT(1, Register::E)),
        (0x4C, Opcode::BIT(1, Register::H)),
        (0x4D, Opcode::BIT(1, Register::L)),
        (0x4E, Opcode::BITHL(1)),
        (0x4F, Opcode::BIT(1, Register::A)),
        (0x50, Opcode::BIT(2, Register::B)),
        (0x51, Opcode::BIT(2, Register::C)),
        (0x52, Opcode::BIT(2, Register::D)),
        (0x53, Opcode::BIT(2, Register::E)),
        (0x54, Opcode::BIT(2, Register::H)),
        (0x55, Opcode::BIT(2, Register::L)),
        (0x56, Opcode::BITHL(2)),
        (0x57, Opcode::BIT(2, Register::A)),
        (0x58, Opcode::BIT(3, Register::B)),
        (0x59, Opcode::BIT(3, Register::C)),
        (0x5A, Opcode::BIT(3, Register::D)),
        (0x5B, Opcode::BIT(3, Register::E)),
        (0x5C, Opcode::BIT(3, Register::H)),
        (0x5D, Opcode::BIT(3, Register::L)),
        (0x5E, Opcode::BITHL(3)),
        (0x5F, Opcode::BIT(3, Register::A)),
        (0x60, Opcode::BIT(4, Register::B)),
        (0x61, Opcode::BIT(4, Register::C)),
        (0x62, Opcode::BIT(4, Register::D)),
        (0x63, Opcode::BIT(4, Register::E)),
        (0x64, Opcode::BIT(4, Register::H)),
        (0x65, Opcode::BIT(4, Register::L)),
        (0x66, Opcode::BITHL(4)),
        (0x67, Opcode::BIT(4, Register::A)),
        (0x68, Opcode::BIT(5, Register::B)),
        (0x69, Opcode::BIT(5, Register::C)),
        (0x6A, Opcode::BIT(5, Register::D)),
        (0x6B, Opcode::BIT(5, Register::E)),
        (0x6C, Opcode::BIT(5, Register::H)),
        (0x6D, Opcode::BIT(5, Register::L)),
        (0x6E, Opcode::BITHL(5)),
        (0x6F, Opcode::BIT(5, Register::A)),
        (0x70, Opcode::BIT(6, Register::B)),
        (0x71, Opcode::BIT(6, Register::C)),
        (0x72, Opcode::BIT(6, Register::D)),
        (0x73, Opcode::BIT(6, Register::E)),
        (0x74, Opcode::BIT(6, Register::H)),
        (0x75, Opcode::BIT(6, Register::L)),
        (0x76, Opcode::BITHL(6)),
        (0x77, Opcode::BIT(6, Register::A)),
        (0x78, Opcode::BIT(7, Register::B)),
        (0x79, Opcode::BIT(7, Register::C)),
        (0x7A, Opcode::BIT(7, Register::D)),
        (0x7B, Opcode::BIT(7, Register::E)),
        (0x7C, Opcode::BIT(7, Register::H)),
        (0x7D, Opcode::BIT(7, Register::L)),
        (0x7E, Opcode::BITHL(7)),
        (0x7F, Opcode::BIT(7, Register::A)),
        (0x80, Opcode::RES(0, Register::B)),
        (0x81, Opcode::RES(0, Register::C)),
        (0x82, Opcode::RES(0, Register::D)),
        (0x83, Opcode::RES(0, Register::E)),
        (0x84, Opcode::RES(0, Register::H)),
        (0x85, Opcode::RES(0, Register::L)),
        (0x86, Opcode::RESHL(0)),
        (0x87, Opcode::RES(0, Register::A)),
        (0x88, Opcode::RES(1, Register::B)),
        (0x89, Opcode::RES(1, Register::C)),
        (0x8A, Opcode::RES(1, Register::D)),
        (0x8B, Opcode::RES(1, Register::E)),
        (0x8C, Opcode::RES(1, Register::H)),
        (0x8D, Opcode::RES(1, Register::L)),
        (0x8E, Opcode::RESHL(1)),
        (0x8F, Opcode::RES(1, Register::A)),
        (0x90, Opcode::RES(2, Register::B)),
        (0x91, Opcode::RES(2, Register::C)),
        (0x92, Opcode::RES(2, Register::D)),
        (0x93, Opcode::RES(2, Register::E)),
        (0x94, Opcode::RES(2, Register::H)),
        (0x95, Opcode::RES(2, Register::L)),
        (0x96, Opcode::RESHL(2)),
        (0x97, Opcode::RES(2, Register::A)),
        (0x98, Opcode::RES(3, Register::B)),
        (0x99, Opcode::RES(3, Register::C)),
        (0x9A, Opcode::RES(3, Register::D)),
        (0x9B, Opcode::RES(3, Register::E)),
        (0x9C, Opcode::RES(3, Register::H)),
        (0x9D, Opcode::RES(3, Register::L)),
        (0x9E, Opcode::RESHL(3)),
        (0x9F, Opcode::RES(3, Register::A)),
        (0xA0, Opcode::RES(4, Register::B)),
        (0xA1, Opcode::RES(4, Register::C)),
        (0xA2, Opcode::RES(4, Register::D)),
        (0xA3, Opcode::RES(4, Register::E)),
        (0xA4, Opcode::RES(4, Register::H)),
        (0xA5, Opcode::RES(4, Register::L)),
        (0xA6, Opcode::RESHL(4)),
        (0xA7, Opcode::RES(4, Register::A)),
        (0xA8, Opcode::RES(5, Register::B)),
        (0xA9, Opcode::RES(5, Register::C)),
        (0xAA, Opcode::RES(5, Register::D)),
        (0xAB, Opcode::RES(5, Register::E)),
        (0xAC, Opcode::RES(5, Register::H)),
        (0xAD, Opcode::RES(5, Register::L)),
        (0xAE, Opcode::RESHL(5)),
        (0xAF, Opcode::RES(5, Register::A)),
        (0xB0, Opcode::RES(6, Register::B)),
        (0xB1, Opcode::RES(6, Register::C)),
        (0xB2, Opcode::RES(6, Register::D)),
        (0xB3, Opcode::RES(6, Register::E)),
        (0xB4, Opcode::RES(6, Register::H)),
        (0xB5, Opcode::RES(6, Register::L)),
        (0xB6, Opcode::RESHL(6)),
        (0xB7, Opcode::RES(6, Register::A)),
        (0xB8, Opcode::RES(7, Register::B)),
        (0xB9, Opcode::RES(7, Register::C)),
        (0xBA, Opcode::RES(7, Register::D)),
        (0xBB, Opcode::RES(7, Register::E)),
        (0xBC, Opcode::RES(7, Register::H)),
        (0xBD, Opcode::RES(7, Register::L)),
        (0xBE, Opcode::RESHL(7)),
        (0xBF, Opcode::RES(7, Register::A)),
        (0xC0, Opcode::SET(0, Register::B)),
        (0xC1, Opcode::SET(0, Register::C)),
        (0xC2, Opcode::SET(0, Register::D)),
        (0xC3, Opcode::SET(0, Register::E)),
        (0xC4, Opcode::SET(0, Register::H)),
        (0xC5, Opcode::SET(0, Register::L)),
        (0xC6, Opcode::SETHL(0)),
        (0xC7, Opcode::SET(0, Register::A)),
        (0xC8, Opcode::SET(1, Register::B)),
        (0xC9, Opcode::SET(1, Register::C)),
        (0xCA, Opcode::SET(1, Register::D)),
        (0xCB, Opcode::SET(1, Register::E)),
        (0xCC, Opcode::SET(1, Register::H)),
        (0xCD, Opcode::SET(1, Register::L)),
        (0xCE, Opcode::SETHL(1)),
        (0xCF, Opcode::SET(1, Register::A)),
        (0xD0, Opcode::SET(2, Register::B)),
        (0xD1, Opcode::SET(2, Register::C)),
        (0xD2, Opcode::SET(2, Register::D)),
        (0xD3, Opcode::SET(2, Register::E)),
        (0xD4, Opcode::SET(2, Register::H)),
        (0xD5, Opcode::SET(2, Register::L)),
        (0xD6, Opcode::SETHL(2)),
        (0xD7, Opcode::SET(2, Register::A)),
        (0xD8, Opcode::SET(3, Register::B)),
        (0xD9, Opcode::SET(3, Register::C)),
        (0xDA, Opcode::SET(3, Register::D)),
        (0xDB, Opcode::SET(3, Register::E)),
        (0xDC, Opcode::SET(3, Register::H)),
        (0xDD, Opcode::SET(3, Register::L)),
        (0xDE, Opcode::SETHL(3)),
        (0xDF, Opcode::SET(3, Register::A)),
        (0xE0, Opcode::SET(4, Register::B)),
        (0xE1, Opcode::SET(4, Register::C)),
        (0xE2, Opcode::SET(4, Register::D)),
        (0xE3, Opcode::SET(4, Register::E)),
        (0xE4, Opcode::SET(4, Register::H)),
        (0xE5, Opcode::SET(4, Register::L)),
        (0xE6, Opcode::SETHL(4)),
        (0xE7, Opcode::SET(4, Register::A)),
        (0xE8, Opcode::SET(5, Register::B)),
        (0xE9, Opcode::SET(5, Register::C)),
        (0xEA, Opcode::SET(5, Register::D)),
        (0xEB, Opcode::SET(5, Register::E)),
        (0xEC, Opcode::SET(5, Register::H)),
        (0xED, Opcode::SET(5, Register::L)),
        (0xEE, Opcode::SETHL(5)),
        (0xEF, Opcode::SET(5, Register::A)),
        (0xF0, Opcode::SET(6, Register::B)),
        (0xF1, Opcode::SET(6, Register::C)),
        (0xF2, Opcode::SET(6, Register::D)),
        (0xF3, Opcode::SET(6, Register::E)),
        (0xF4, Opcode::SET(6, Register::H)),
        (0xF5, Opcode::SET(6, Register::L)),
        (0xF6, Opcode::SETHL(6)),
        (0xF7, Opcode::SET(6, Register::A)),
        (0xF8, Opcode::SET(7, Register::B)),
        (0xF9, Opcode::SET(7, Register::C)),
        (0xFA, Opcode::SET(7, Register::D)),
        (0xFB, Opcode::SET(7, Register::E)),
        (0xFC, Opcode::SET(7, Register::H)),
        (0xFD, Opcode::SET(7, Register::L)),
        (0xFE, Opcode::SETHL(7)),
        (0xFF, Opcode::SET(7, Register::A)),
    ]);
}
