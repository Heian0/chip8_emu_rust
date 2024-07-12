use rand::Rng;

const RAM_SIZE: usize = 4096;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const START_ADDRESS: u16 = 0x200;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

// Fontset holds 16 digits from 0 -> F,
// 1,
// 2,
// 3...

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80
];

pub struct Chip8 {
    pc: u16,
    ram: [u8; RAM_SIZE],
    v_regi: [u8; NUM_REGS],
    i_regi: u16,
    display: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    stack: [u16; STACK_SIZE],
    stkp: u16,
    delay_t: u8,
    sound_t: u8,
    keys: [bool; NUM_KEYS],
}

impl Chip8 {
    pub fn init() -> Self {
        let mut chip8_emu: Chip8 = Self {
            pc: START_ADDRESS,
            ram: [0; RAM_SIZE],
            v_regi: [0; NUM_REGS],
            i_regi: 0,
            display: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            stack: [0; STACK_SIZE],
            stkp: 0,
            delay_t: 0,
            sound_t: 0,
            keys: [false; NUM_KEYS]
        };  

        chip8_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
        chip8_emu
    }

    fn push(&mut self, data: u16) {
        self.stack[self.stkp as usize] = data;
        self.stkp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.stkp -= 1;
        self.stack[self.stkp as usize]
    }

    pub fn get_display(&self) -> &[bool] {
        &self.display        
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDRESS as usize;
        let end = (START_ADDRESS as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    // Reset emulator as needed
    pub fn reset(&mut self) {
        self.pc = START_ADDRESS;
        self.ram = [0; RAM_SIZE];
        self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_regi = [0; NUM_REGS];
        self.i_regi = 0;
        self.stkp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_t = 0;
        self.sound_t = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn clock(&mut self) {
        // Fetch
        let opcode: u16 = self.fetch();
        // Decode -> Execute
        self.execute(opcode);
    }

    fn fetch(&mut self) -> u16 {
        let high: u16 = self.ram[self.pc as usize] as u16;
        let low: u16 = self.ram[(self.pc + 1) as usize] as u16;
        let opcode: u16 = (high << 8) | low;
        self.pc += 2;
        opcode
    }

    fn execute(&mut self, opcode: u16) {
        let d1: u16 = (opcode & 0xF000) >> 12;
        let d2: u16 = (opcode & 0x0F00) >> 8;
        let d3: u16 = (opcode & 0x00F0) >> 4;
        let d4: u16 = opcode & 0x000F;

        match (d1, d2, d3, d4) {
           
            // NOP - Do nothing
            (0, 0, 0, 0) => return,
 
            // CLS - Clear display
            (0, 0, 0xE, 0) => {
                self.display = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },

            // RET - Return from subroutine
            (0, 0, 0xE, 0xE) => {
                let return_address: u16 = self.pop();
                self.pc = return_address;
            },

            // JMP NNN - Move the program counter to a given address
            (1, _, _, _) => {
                let nnn: u16 = opcode & 0xFFF;
                self.pc = nnn;
            },

            // CALL NNN - Call subroutine
            (2, _, _, _) => {
                let nnn: u16 = opcode & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            },
    
            // SKIP VX == NN - Skip if equal
            (3, _, _, _) => {
                let x: usize = d2 as usize;
                let nn: u8 = (opcode & 0xFF) as u8;
                if self.v_regi[x] == nn {
                    self.pc += 2;
                }
            },

            // SKIP VX != NN - Skip not equal
            (4, _, _, _) => {
                let x: usize = d2 as usize;
                let nn: u8 = (opcode & 0xFF) as u8;
                if self.v_regi[x] != nn {
                    self.pc += 2;
                }
            },

            // SKIP VX == VY - Skip if VX == VY
            (5, _, _, _) => {
                let x: usize = d2 as usize;
                let y: usize = d3 as usize;
                if self.v_regi[x] == self.v_regi[y] {
                    self.pc += 2;
                }
            },

            // VX = NN - Set V register to given value
            (6, _, _, _) => {
                let x: usize = d2 as usize;
                let nn: u8 = (opcode & 0xFF) as u8;
                self.v_regi[x] = nn;
            },

            // VX += NN - Add given value to VX reigister
            (7, _, _, _) => {
                let x: usize = d2 as usize;
                let nn: u8 = (opcode & 0xFF) as u8;
                self.v_regi[x] = self.v_regi[x].wrapping_add(nn);
            },

            // VX = VY - Set a register x to the same value as a register y
            (8, _, _, 0) => {
                let x: usize = d2 as usize;
                let y: usize = d3 as usize;
                self.v_regi[x] = self.v_regi[y];
            },
    
            // VX |= VY - Bitwise OR
            (8, _, _, 1) => {
                let x: usize = d2 as usize;
                let y: usize = d3 as usize;
                self.v_regi[x] |= self.v_regi[y];
            },

            // VX &= VY - Bitwise AND
            (8, _, _, 2) => {
                let x: usize = d2 as usize;
                let y: usize = d3 as usize;
                self.v_regi[x] &= self.v_regi[y];
            },

            // VX ^= VY - Bitwise XOR
            (8, _, _, 3) => {
                let x: usize = d2 as usize;
                let y: usize = d3 as usize;
                self.v_regi[x] ^= self.v_regi[y];
            },

            // VX += VY - Add with carry
            (8, _, _, 4) => {
                let x: usize = d2 as usize;
                let y: usize = d3 as usize;
                let (new_vx, carry) = self.v_regi[x].overflowing_add(self.v_regi[y]);
                let new_vf = if carry { 1 } else { 0 };
                self.v_regi[x] = new_vx;
                self.v_regi[0xF] = new_vf;
            },

            // VX -= VY - Subtract with carry
            (8, _, _, 5) => {
                let x: usize = d2 as usize;
                let y: usize = d3 as usize;
                let (new_vx, borrow) = self.v_regi[x].overflowing_sub(self.v_regi[y]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_regi[x] = new_vx;
                self.v_regi[0xF] = new_vf;
            },

            // VX >>= 1 - Shift right with dropoff stored in carry
            (8, _, _, 6) => {
                let x = d2 as usize;
                let lsb = self.v_regi[x] & 1;
                self.v_regi[x] >>= 1;
                self.v_regi[0xF] = lsb;
            },

            // VX = VY - VX - Subtract with carry, reversed operands
            (8, _, _, 7) => {
                let x: usize = d2 as usize;
                let y: usize = d3 as usize;
                let (new_vx, borrow) = self.v_regi[y].overflowing_sub(self.v_regi[x]);
                let new_vf = if borrow { 0 } else { 1 };
                self.v_regi[x] = new_vx;
                self.v_regi[0xF] = new_vf;
            },

            // VX <<= 1 - Left shift with dropoff stored in flag
            (8, _, _, 0xE) => {
                let x: usize = d2 as usize;
                let msb = (self.v_regi[x] >> 7) & 1;
                self.v_regi[x] <<= 1;
                self.v_regi[0xF] = msb;
            },
    
            // SKIP VX != VY - Skip if VX == VY
            (9, _, _, 0) => {
                let x: usize = d2 as usize;
                let y: usize = d3 as usize;
                if self.v_regi[x] != self.v_regi[y] {
                    self.pc += 2;
                }
            },

            // I = NNN - Set I register
            (0xA, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.i_regi = nnn;
            },
    
            // JMP V0 + NNN - Jump to V0 + NNN
            (0xB, _, _, _) => {
                let nnn = opcode & 0xFFF;
                self.pc = (self.v_regi[0] as u16) + nnn;
            },

            // VX = rand() & NN - Generate random number and store in VX register
            (0xC, _, _, _) => {
                let x: usize = d2 as usize;
                let nn: u8 = (opcode & 0xFF) as u8;
                let rng: u8 = rand::thread_rng().gen();
                self.v_regi[x] = rng & nn;
            },

            // DRAW - Draw sprite on screen at location (d2, d3). Sprites are always 8 pixels wide, but height
            // of sprite is stored in d4. Sprites are stored row by row starting from location stored in register I.
            (0xD, _, _, _) => {
                // Get the (x, y) coords for our sprite
                let x = self.v_regi[d2 as usize] as u16;
                let y = self.v_regi[d3 as usize] as u16;
                // The last digit determines how many rows high our sprite is
                let num_rows = d4;

                // Keep track if any pixels were flipped
                let mut flipped = false;
                // Iterate over each row of our sprite
                for y_line in 0..num_rows {
                    // Determine which memory address our row's data is stored
                    let addr = self.i_regi + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    // Iterate over each column in our row
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x + x_line) as usize % SCREEN_WIDTH;
                            let y = (y + y_line) as usize % SCREEN_HEIGHT;

                            // Get our pixel's index in the 1D screen array
                            let idx = x + SCREEN_WIDTH * y;
                            // Check if we're about to flip the pixel and set
                            flipped |= self.display[idx];
                            self.display[idx] ^= true;
                        }
                    }
                }
                // Populate VF register
                if flipped {
                    self.v_regi[0xF] = 1;
                } else {
                    self.v_regi[0xF] = 0;
                }
            },

            // SKIP KEY PRESS - Skip if key stored in VX is pressed
            (0xE, _, 9, 0xE) => {
                let x: usize = d2 as usize;
                let vx: u8 = self.v_regi[x];
                let key: bool = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },

            // SKIP KEY RELEASE - Skip if key stored in VX isnot pressed
            (0xE, _, 0xA, 1) => {
                let x = d2 as usize;
                let vx = self.v_regi[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },

            // VX = DT - Stores delay timer in a register specified by d2
            (0xF, _, 0, 7) => {
                let x: usize = d2 as usize;
                self.v_regi[x] = self.delay_t;
            },
    
            // WAIT KEY - Block until key pressed
            (0xF, _, 0, 0xA) => {
                let x = d2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_regi[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    // Redo opcode
                    self.pc -= 2;
                }
            },

            // DT = VX - Set delay timer to value in VX
             (0xF, _, 1, 5) => {
                let x = d2 as usize;
                self.delay_t = self.v_regi[x];
            },

            // ST = VX - Set sound timer to value in VX
            (0xF, _, 1, 8) => {
                let x = d2 as usize;
                self.sound_t = self.v_regi[x];
            },
    
            // I += VX - Add VX to I
            (0xF, _, 1, 0xE) => {
                let x = d2 as usize;
                let vx = self.v_regi[x] as u16;
                self.i_regi = self.i_regi.wrapping_add(vx);
            },
    
            // I = FONT - Set I to font address
            (0xF, _, 2, 9) => {
                let x = d2 as usize;
                let c = self.v_regi[x] as u16;
                self.i_regi = c * 5;
            },

            // BCD - Store BCD(VX) in I
            (0xF, _, 3, 3) => {
                let x = d2 as usize;
                let vx = self.v_regi[x] as f32;

                let hundreds: u8 = (vx / 100.0).floor() as u8;
                let tens: u8 = ((vx / 10.0) % 10.0).floor() as u8;
                let ones: u8 = (vx % 10.0) as u8;

                self.ram[self.i_regi as usize] = hundreds;
                self.ram[(self.i_regi + 1) as usize] = tens;
                self.ram[(self.i_regi + 2) as usize] = ones;
            },
            
            // STORE V0 - VX - Store V0 - VX in I register
            (0xF, _, 5, 5) => {
                let x = d2 as usize;
                let i = self.i_regi as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_regi[idx];
                }
            },

            // LOAD V0 - VX - Load I into V0 - VX
            (0xF, _, 6, 5) => {
                let x = d2 as usize;
                let i = self.i_regi as usize;
                for idx in 0..=x {
                    self.v_regi[idx] = self.ram[i + idx];
                }
            },
    
            (_, _, _, _) => unimplemented!("Received unimplemented opcode: {}", opcode),
        }
    }  

    pub fn clock_timers(&mut self) {
        if self.delay_t > 0 {
            self.delay_t -= 1;
        }

        if self.sound_t > 0 {
            if self.sound_t == 1 {
                // BEEP
            }
            self.sound_t -= 1;
        }
    } 
}