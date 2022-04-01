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
    pub fn rvv_get_lmul(processor: u64) -> u64;
    pub fn rvv_get_vill(processor: u64) -> u64;
    pub fn rvv_load_mem(processor: u64, addr: u64, len: u64, bytes: *const u8) -> i32;
    pub fn rvv_store_mem(processor: u64, addr: u64, len: u64, bytes: *const u8) -> i32;

    pub fn rvv_delete_processor(processor: u64);

    pub fn rvv_new_disassembler(xlen: u32) -> u64;
    pub fn rvv_disassemble(dis: u64, inst: u64, output: *mut u8, output_len: u32) -> i32;
    pub fn rvv_delete_disassembler(dis: u64);
}
