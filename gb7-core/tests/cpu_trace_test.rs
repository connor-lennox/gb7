use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

use test_case::test_case;

use gb7_core::{cartridge, gameboy::Gameboy};

fn get_log_string(gb: &Gameboy) -> String {
    format!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})",
        gb.cpu.registers.a, gb.cpu.registers.flags.bits(), gb.cpu.registers.b, gb.cpu.registers.c, gb.cpu.registers.d, gb.cpu.registers.e, gb.cpu.registers.h, gb.cpu.registers.l, gb.cpu.sp, gb.cpu.pc,
        gb.read(gb.cpu.pc), gb.read(gb.cpu.pc+1), gb.read(gb.cpu.pc+2), gb.read(gb.cpu.pc+3))
}

#[test_case("01-special" ; "special")]
#[test_case("02-interrupts" ; "interrupts")]
#[test_case("03-op sp,hl" ; "sp,hl")]
#[test_case("04-op r,imm" ; "r,imm")]
#[test_case("05-op rp" ; "rp")]
#[test_case("06-ld r,r" ; "ld r,r")]
#[test_case("07-jr,jp,call,ret,rst" ; "jr,jp,call,ret,rst")]
#[test_case("08-misc instrs" ; "misc")]
#[test_case("09-op r,r" ; "op r,r")]
#[test_case("10-bit ops" ; "bit ops")]
#[test_case("11-op a,(hl)" ; "op a,hl")]
fn run_blargg_test(test_name: &str) {
    let mut cart_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    cart_path.push(format!("resources/blargg/{}.gb", test_name));

    let cart_data = fs::read(cart_path).unwrap();

    let cart = cartridge::load_cartridge(&cart_data);

    let mut gameboy = Gameboy::new_dmg(cart);

    // Blargg tests will output to the serial port on the gameboy as well as the screen.
    // By polling the serial port after each instruction, we can read out lines and
    // look for some target text that indicates a pass/fail.

    let mut line_buffer = String::new();
    loop {
        gameboy.execute();
        // Simulate taking data off the serial bus
        if gameboy.read(0xFF02) == 0x81 {
            let new_char = gameboy.read(0xFF01) as char;
            if new_char == '\n' {
                if line_buffer == String::from("Passed") {
                    // Pass case
                    break;
                }
                if !(line_buffer == String::from(test_name)) && line_buffer.len() > 0 {
                    // Fail case
                    panic!("Test failed: {}", line_buffer);
                }
                // Reset buffer
                line_buffer = String::new();
            } else {
                line_buffer.push(new_char);
            }
            gameboy.write(0xFF02, 0);
        }
    }
}

// #[test]
// fn r_imm_log_test() {
//     let mut log_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//     log_path.push("resources/blargg/Blargg2.txt");
//     let ref_file = File::open(log_path).expect("Could not open reference log");
//     let mut ref_lines = io::BufReader::new(ref_file).lines();

//     let mut cart_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//     cart_path.push("resources/blargg/02-interrupts.gb");

//     let cart_data = fs::read(cart_path).unwrap();

//     let cart = cartridge::load_cartridge(&cart_data);

//     let mut gameboy = Gameboy::new_dmg(cart);

//     // Note: LY register (0xFF44) must be stubbed to 0x90!

//     let mut count = 0;
//     loop {
        
//         if !check_interrupts(&gameboy) {
//             let log_string = get_log_string(&gameboy);
//             let ref_op = ref_lines.next();

//             if ref_op.is_none() {
//                 // Finished log
//                 break;
//             }

//             let ref_string = ref_op.unwrap().unwrap();
//             if !(log_string == ref_string) {
//                 panic!("reference log mismatch: expected\n{}\nbut got\n{}\n at line {}", ref_string, log_string, count);
//             }
//         }

//         gameboy.execute();
//         count += 1;

//         if count == 152481 {
//             print!("");
//         }
//     }
// }

// fn check_interrupts(gameboy: &Gameboy) -> bool {
//     if !gameboy.cpu.ime && !gameboy.cpu.halted {
//         return false;
//     }

//     let if_reg = gameboy.read(0xFF0F);
//     let interrupts = gameboy.read(0xFFFF) & if_reg;

//     interrupts != 0
// }