use spike_sys::*;

pub fn main() {
    let proc = unsafe { rvv_new_processor(128, 64, 4096) };
    let addr: u64 = 4096;
    // set vl
    let insn: u64 = 0xc18472d7; // vsetivli t0, 8, e64, m1
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    let vlen = unsafe { rvv_get_vlen(proc) };
    let elen = unsafe { rvv_get_elen(proc) };
    println!("vlen = {}, elen = {}", vlen, elen);

    let vlenb = vlen / 8;

    // register t0 holds address of "4096"(addr)
    // register t1 holds the value, initially 17(val)
    let err = unsafe { rvv_execute(proc, 0x6285) }; // li t0, 4096
    assert_eq!(err, 0);
    let err = unsafe { rvv_execute(proc, 0x4345) }; // li     t1,17
    assert_eq!(err, 0);
    // store t1 to addr
    let err = unsafe { rvv_execute(proc, 0x0062b023) }; // sd      t1,0(t0)
    assert_eq!(err, 0);

    // modify the content of addr
    let new_val: u64 = 18;
    let err = unsafe { rvv_store_mem(proc, addr, 8, (&new_val) as *const u64 as *const u8) };
    assert_eq!(err, 0);

    // modify the content of addr + 8
    let new_val: u64 = 19;
    let err = unsafe { rvv_store_mem(proc, addr + 8, 8, (&new_val) as *const u64 as *const u8) };
    assert_eq!(err, 0);

    // load
    let insn: u64 = 0x202f107; // vle64.v   v2, (t0)
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    // check
    let mut result: [u64; 2] = [0, 0];
    let err = unsafe { rvv_get_vreg(proc, vlenb * 2, (&mut result).as_mut_ptr() as *mut u8, 16) };
    assert_eq!(err, 0);

    assert_eq!(result[0], 18);
    assert_eq!(result[1], 19);

    // increase t0 by 16, t0 now points to 4096+16
    let insn: u64 = 0x02c1; //  addi    t0,t0,16
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    // store v2 to memory at `4096 + 16`
    let insn: u64 = 0x0202f127; // vse64.v   v2, (t0)
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    // check the content of addr (`4096 + 16`)
    let mut result: [u64; 2] = [0, 0];
    let err = unsafe { rvv_load_mem(proc, addr, 16, (&mut result) as *mut u64 as *mut u8) };
    assert_eq!(err, 0);
    assert_eq!(result[0], 18);
    assert_eq!(result[1], 19);

    unsafe { rvv_delete_processor(proc) };
    println!("done");
}
