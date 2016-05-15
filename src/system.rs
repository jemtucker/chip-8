const SIZE_MEM: usize = 4096;
const SIZE_GFX: usize = 2048;
const SIZE_KEYS: usize = 16;

pub struct System {
    memory: [u8; SIZE_MEM],
    gfx: [bool; SIZE_GFX], 
    keys: [bool; SIZE_KEYS]
}

impl System {
    pub fn new() -> System {
        System {
            memory: [0; SIZE_MEM],
            gfx: [false; SIZE_GFX],
            keys: [false; SIZE_KEYS]
        }
    }

    pub fn get_mem(&self, i: usize) -> u8 {
        self.memory[i]
    }
}