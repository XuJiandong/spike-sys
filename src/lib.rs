#[link(name = "spike-interfaces", kind = "static")]
extern "C" {
    pub fn rvv_new_processor(vlen: u32, elen: u32, mem_size: u64) -> u64;
    pub fn rvv_execute(processor: u64, instruction: u64) -> i32;
    /**
     *  offset: the offset in register file in bytes
     *
     *  return: 0, success; otherwise, failed. Same to other functions
     */
    pub fn rvv_get_vreg(processor: u64, offset: u64, mem: *const u8, mem_size: u64) -> i32;
    pub fn rvv_set_vreg(processor: u64, offset: u64, mem: *const u8, mem_size: u64) -> i32;
    pub fn rvv_get_xreg(processor: u64, index: u64, content: *mut u64) -> i32;
    pub fn rvv_set_xreg(processor: u64, index: u64, content: u64) -> i32;

    pub fn rvv_get_vlen(processor: u64) -> u64;
    pub fn rvv_get_elen(processor: u64) -> u64;
    pub fn rvv_get_vl(processor: u64) -> u64;
    pub fn rvv_get_sew(processor: u64) -> u64;
    pub fn rvv_get_vtype(processor: u64) -> u64;
    pub fn rvv_get_lmul(processor: u64) -> f32;
    pub fn rvv_get_vill(processor: u64) -> u64;
    pub fn rvv_load_mem(processor: u64, addr: u64, len: u64, bytes: *const u8) -> i32;
    pub fn rvv_store_mem(processor: u64, addr: u64, len: u64, bytes: *const u8) -> i32;

    pub fn rvv_delete_processor(processor: u64);

    pub fn rvv_new_disassembler(xlen: u32) -> u64;
    pub fn rvv_disassemble(dis: u64, inst: u64, output: *mut u8, output_len: *mut u32) -> i32;
    pub fn rvv_delete_disassembler(dis: u64);
}

#[derive(Debug)]
pub struct Error(i32);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error({})", self.0)
    }
}
impl std::error::Error for Error {}

pub struct Spike {
    addr: u64,
}

impl Spike {
    pub fn new(vlen: u32, elen: u32, mem_size: u64) -> Self {
        unsafe {
            Self {
                addr: rvv_new_processor(vlen, elen, mem_size),
            }
        }
    }

    pub fn execute(&self, instruction: u64) -> Result<(), Error> {
        let r = unsafe { rvv_execute(self.addr, instruction) };
        if r != 0 {
            Err(Error(r))
        } else {
            Ok(())
        }
    }

    pub fn get_vreg(&self, offset: u64, mem: *const u8, mem_size: u64) -> Result<(), Error> {
        let r = unsafe { rvv_get_vreg(self.addr, offset, mem, mem_size) };
        if r != 0 {
            Err(Error(r))
        } else {
            Ok(())
        }
    }

    pub fn set_vreg(&self, offset: u64, mem: *const u8, mem_size: u64) -> Result<(), Error> {
        let r = unsafe { rvv_set_vreg(self.addr, offset, mem, mem_size) };
        if r != 0 {
            Err(Error(r))
        } else {
            Ok(())
        }
    }

    pub fn get_xreg(&self, index: u64) -> Result<u64, Error> {
        let mut x = 0;
        let r = unsafe { rvv_get_xreg(self.addr, index, &mut x) };
        if r != 0 {
            Err(Error(r))
        } else {
            Ok(x)
        }
    }

    pub fn set_xreg(&self, index: u64, content: u64) -> Result<(), Error> {
        let r = unsafe { rvv_set_xreg(self.addr, index, content) };
        if r != 0 {
            Err(Error(r))
        } else {
            Ok(())
        }
    }

    pub fn get_vlen(&self) -> u64 {
        unsafe { rvv_get_vlen(self.addr) }
    }

    pub fn get_elen(&self) -> u64 {
        unsafe { rvv_get_elen(self.addr) }
    }

    pub fn get_vl(&self) -> u64 {
        unsafe { rvv_get_vl(self.addr) }
    }

    pub fn get_sew(&self) -> u64 {
        unsafe { rvv_get_sew(self.addr) }
    }

    pub fn get_vtype(&self) -> u64 {
        unsafe { rvv_get_vtype(self.addr) }
    }

    pub fn get_lmul(&self) -> f32 {
        unsafe { rvv_get_lmul(self.addr) }
    }

    pub fn get_vill(&self) -> u64 {
        unsafe { rvv_get_vill(self.addr) }
    }

    pub fn load_mem(&self, addr: u64, len: u64, bytes: *const u8) -> Result<(), Error> {
        let r = unsafe { rvv_load_mem(self.addr, addr, len, bytes) };
        if r != 0 {
            Err(Error(r))
        } else {
            Ok(())
        }
    }

    pub fn store_mem(&self, addr: u64, len: u64, bytes: *const u8) -> Result<(), Error> {
        let r = unsafe { rvv_store_mem(self.addr, addr, len, bytes) };
        if r != 0 {
            Err(Error(r))
        } else {
            Ok(())
        }
    }
}

impl Drop for Spike {
    fn drop(&mut self) {
        unsafe { rvv_delete_processor(self.addr) }
    }
}
