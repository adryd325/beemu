use std::{thread, time};

fn u4(input:u8) -> u8 {
    // Hack to trim u8 to u4
    let mut out: u8 = 0;
    for i in 0..4 {
        out |= input & 1 << i
    }
    return out;
}

fn main() {
    let mut carry: bool;
    let mut control_word: u16;
    let mut sum_register: u8;
    let mut bus: u8;

    let mut flags: u8 = 0b00;
    let mut step: usize = 0b000;

    let mut program_counter: u8 = 0b0000;
    let mut instruction_register: usize = 0b00000000;
    let mut mem_address_register: usize = 0b0000;
    
    let mut a_register: u8 = 0b00000000;
    let mut b_register: u8 = 0b00000000;
    let mut output_register: u8 = 0b00000000;

    // CPU Flags
    const CF: u8  = 2 << 0; // Carry 
    const ZF: u8  = 2 << 1; // Zero

    // Microinstructions
    const HLT: u16 = 2 << 0; // Halt
    const MI: u16  = 2 << 1; // Memory address register in
    const RI: u16  = 2 << 2; // RAM in
    const RO: u16  = 2 << 3; // RAM out
    const IO: u16  = 2 << 4; // Instruction register out
    const II: u16  = 2 << 5; // Instruction register in
    const AI: u16  = 2 << 6; // A register in
    const AO: u16  = 2 << 7; // A register out
    const EO: u16  = 2 << 8; // ALU/sum register out
    const SU: u16  = 2 << 9; // ALU subtract
    const BI: u16  = 2 << 10; // B register in
    const OI: u16  = 2 << 11; // Output in
    const CE: u16  = 2 << 12; // Counter enable
    const CO: u16  = 2 << 13; // Counter out
    const J: u16   = 2 << 14; // Jump (counter in)
    const FI: u16  = 2 << 15; // Flags in

    // Instructions
    const NOP: u8 = 0b0000;
    const LDA: u8 = 0b0001;
    const ADD: u8 = 0b0010;
    const SUB: u8 = 0b0011;
    const STA: u8 = 0b0100;
    const LDI: u8 = 0b0101;
    const JMP: u8 = 0b0110;
    const JC: u8 = 0b0111;
    const JZ: u8 = 0b1000;
    const OUT: u8 = 0b1110;
    const HALT: u8 = 0b1111;

    const INSTRUCTIONS: [[u16; 5]; 16] = [
        [MI|CO,  RO|II|CE,  0,      0,      0         ], // 0000 - NOP
        [MI|CO,  RO|II|CE,  IO|MI,  RO|AI,  0         ], // 0001 - LDA
        [MI|CO,  RO|II|CE,  IO|MI,  RO|BI,  EO|AI|FI  ], // 0010 - ADD
        [MI|CO,  RO|II|CE,  IO|MI,  RO|BI,  EO|AI|SU|FI], // 0011 - SUB
        [MI|CO,  RO|II|CE,  IO|MI,  AO|RI,  0         ], // 0100 - STA
        [MI|CO,  RO|II|CE,  IO|AI,  0,      0         ], // 0101 - LDI
        [MI|CO,  RO|II|CE,  IO|J,   0,      0         ], // 0110 - JMP
        [MI|CO,  RO|II|CE,  IO|J,   0,      0,        ], // 0111 - JC // this has a special implementation
        [MI|CO,  RO|II|CE,  IO|J,   0,      0,        ], // 1000 - JZ // this has a special implementation
        [MI|CO,  RO|II|CE,  0,      0,      0,        ], // 1001
        [MI|CO,  RO|II|CE,  0,      0,      0,        ], // 1010
        [MI|CO,  RO|II|CE,  0,      0,      0,        ], // 1011
        [MI|CO,  RO|II|CE,  0,      0,      0,        ], // 1100
        [MI|CO,  RO|II|CE,  0,      0,      0,        ], // 1101
        [MI|CO,  RO|II|CE,  AO|OI,  0,      0,        ], // 1110 - OUT
        [MI|CO,  RO|II|CE,  HLT,    0,      0,        ], // 1111 - HALT
    ];
            
    // let mut memory: [u8; 16] = [
    //     LDA << 4 | 0xf,
    //     ADD << 4 | 0xf,
    //     OUT << 4 | 0x0,
    //     STA << 4 | 0xf,
    //     JC << 4 | 0x6,
    //     JMP << 4 | 0x0,
    //     HALT << 4 | 0x0,
    //     0,
    //     0,
    //     0,
    //     0,
    //     0,
    //     0,
    //     0,
    //     0,
    //     0x01,
    // ];

    let mut memory = [
        LDI << 4 | 0x1,
        STA << 4 | 0xe,
        LDI << 4 | 0x0,
        OUT << 4,
        ADD << 4 | 0xe,
        STA << 4 | 0xf,
        LDA << 4 | 0xe,
        STA << 4 | 0xd,
        LDA << 4 | 0xf,
        STA << 4 | 0xe,
        LDA << 4 | 0xd,
        JC << 4  | 0x0,
        JMP << 4 | 0x3,
        0x00,
        0x00,
        0x00
    ];

    loop {
        bus = 0;
        // Instructions
        control_word = INSTRUCTIONS[instruction_register >> 4][step];

        println!("-----------------------------------------------");

        // META
        if control_word & HLT == HLT {
            println!(" --- Halt!");
            break;
        }
        if control_word & CE == CE {
            println!(" --- Counter enable");
            program_counter += 1;
            if program_counter == 16 {
                program_counter = 0
            }
        }
        if control_word & FI == FI {
            println!(" --- Flags set");
            // Reset flags if we have flags in
            flags = 0;
        }

        // Sum register
        if control_word & SU == SU {
            println!(" --- Subtract");
            let (tmp_sum_register, tmp_carry) = a_register.overflowing_sub(b_register);
            sum_register = tmp_sum_register;
            carry = tmp_carry;
        } else {        
            let (tmp_sum_register, tmp_carry) = a_register.overflowing_add(b_register);
            sum_register = tmp_sum_register;
            carry = tmp_carry;
        }
        // CPU Flags    
        if carry && control_word & FI == FI {
            flags = flags | CF
        }
        if sum_register == 0 && control_word & FI == FI {
            flags = flags | ZF
        }

        // WRITE
        if control_word & RO == RO {
            println!(" --> RAM out");
            bus = memory[mem_address_register]
        }
        if control_word & IO == IO {
            println!(" --> Instruction register out");
            bus = u4(instruction_register as u8);
        }
        if control_word & AO == AO {
            println!(" --> A register out");
            bus = a_register;
        }
        if control_word & EO == EO {
            println!(" --> Sum register out");
            bus = sum_register;
        }
        if control_word & CO == CO {
            println!(" --> Program counter out");
            bus = program_counter as u8;
        }


        // READ
        if control_word & MI == MI {
            println!(" <-- Memory address register in");
            // Limit to u4
            mem_address_register = u4(bus) as usize;
        }
        if control_word & RI == RI {
            println!(" <-- Memory in");
            memory[mem_address_register] = bus;
        }
        if control_word & AI == AI {
            println!(" <-- A register in");
            a_register = bus;
        }
        if control_word & BI == BI {
            println!(" <-- B register in");
            b_register = bus;
        }
        if control_word & OI == OI {
            println!(" <-- Out register in");
            output_register = bus;
        }
        if control_word & II == II {
            println!(" <-- Instruction register in");
            // Limit u4
            instruction_register = bus as usize;
        }

        
        if instruction_register >> 4 != 0b0111 && 
           instruction_register >> 4 != 0b1000 {
            if control_word & J == J {
                println!(" <-- Jump (Program counter in)");
                // Limit to u4
                program_counter = u4(bus);
            }
        }

        if instruction_register >> 4 == 0b0111 && flags & CF == CF && control_word & J == J ||
           instruction_register >> 4 == 0b1000 && flags & ZF == ZF && control_word & J == J {
             println!(" <-- Jump (Program counter in)");
             // Limit to u4
             program_counter = u4(bus);
     }
        
        println!("control_word:    {:#018b} {:#06x} {}",control_word,control_word,control_word);
        println!("control_word:    {:#018b} {:#06x} {}",control_word,control_word,control_word);
        println!("a_register:      {:#010b}         {:#04x}   {}",a_register,a_register,a_register);
        println!("b_register:      {:#010b}         {:#04x}   {}",b_register,b_register,b_register);
        println!("sum_register:    {:#010b}         {:#04x}   {}",sum_register,sum_register,sum_register);
        println!("\x1b[1;7;32moutput_register: {:#010b}         {:#04x}   {}\x1b[0m",output_register,output_register,output_register);
        println!("bus:             {:#010b}         {:#04x}   {}",bus,bus,bus);
        println!("flags:           {:#010b}         {:#04x}   {}",flags,flags,flags);
        println!("mem:");
        for index in 0..16 {
            print!("{:02x} ",memory[index])
        }
        println!();

        thread::sleep(time::Duration::from_millis(20));
        // Increase step
        step += 1;
        if step == 5 {step = 0};
    }
}