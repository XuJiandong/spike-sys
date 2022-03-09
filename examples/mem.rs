use spike_sys::*;

pub fn main() {
    let proc = unsafe { rvv_new_processor(128, 64, 4096) };
    let addr: u64 = 4096;
    let val: u64 = 17;
    // register t0 holds address of "4096"(addr)
    // register t1 holds the value, initially 17(val)
    let err = unsafe { rvv_execute(proc, 0x6285) }; // li t0, 4096
    assert_eq!(err, 0);
    let err = unsafe { rvv_execute(proc, 0x4345) }; // li     t1,17
    assert_eq!(err, 0);
    // store t1 to addr
    let err = unsafe { rvv_execute(proc, 0x0062b023) }; // sd      t1,0(t0)
    assert_eq!(err, 0);

    // check the content of addr
    let mut val_mem: u64 = 0;
    let err = unsafe { rvv_load_mem(proc, addr, 8, (&mut val_mem) as *mut u64 as *mut u8) };
    assert_eq!(err, 0);
    assert_eq!(val_mem, val);

    // modify the content of addr
    let new_val: u64 = 18;
    let err = unsafe { rvv_store_mem(proc, addr, 8, (&new_val) as *const u64 as *const u8) };
    assert_eq!(err, 0);

    // load the content of addr to t1
    let err = unsafe { rvv_execute(proc, 0x0002b303) }; // ld      t1,0(t0)
    assert_eq!(err, 0);

    // check the content of t1
    let mut t1: u64 = 0;
    let err = unsafe { rvv_get_xreg(proc, 6, (&mut t1) as *mut u64) };
    assert_eq!(err, 0);
    assert_eq!(t1, 18);

    unsafe { rvv_delete_processor(proc) };
    println!("done");
}
