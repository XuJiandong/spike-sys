use spike_sys::*;

pub fn main() {
    let proc = unsafe { rvv_new_processor() };
    let vlen = unsafe { rvv_get_vlen(proc) };
    let elen = unsafe { rvv_get_elen(proc) };
    println!("vlen = {}, elen = {}", vlen, elen);

    let vlenb = vlen / 8;
    let value_one: [u64; 2] = [1, 1];
    let mut result: [u64; 2] = [0, 0];

    let insn: u64 = 0xc18472d7; // vsetivli t0, 8, e64, m1
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    let vl = unsafe { rvv_get_vl(proc) };
    let sew = unsafe { rvv_get_sew(proc) };
    let lmul = unsafe { rvv_get_lmul(proc) };
    let vill = unsafe { rvv_get_vill(proc) };
    println!(
        "vl = {}, sew = {}, lmul = {}, vill = {}",
        vl, sew, lmul, vill
    );

    let err = unsafe { rvv_set_vreg(proc, vlenb * 10, (&value_one).as_ptr() as *const u8, 16) };
    assert_eq!(err, 0);
    let err = unsafe { rvv_set_vreg(proc, vlenb * 20, (&value_one).as_ptr() as *const u8, 16) };
    assert_eq!(err, 0);

    let insn = 0x02aa0157; // vadd.vv v2, v10, v20
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    let err = unsafe { rvv_get_vreg(proc, vlenb * 2, (&mut result).as_mut_ptr() as *mut u8, 16) };
    assert_eq!(err, 0);
    assert_eq!(result[0], 2);
    assert_eq!(result[1], 2);

    let err = unsafe { rvv_set_xreg(proc, 5, 100) };
    assert_eq!(err, 0);
    let insn = 0x02a2c157; // vadd.vx v2, v10, x5
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    let err = unsafe { rvv_get_vreg(proc, vlenb * 2, (&mut result).as_mut_ptr() as *mut u8, 16) };
    assert_eq!(err, 0);

    println!("result[0] = {}, result[1] = {}", result[0], result[1]);
    assert_eq!(result[0], 101);
    assert_eq!(result[1], 101);

    let insn = 0x02a53157; // vadd.vi v2, v10, 10
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    let err = unsafe { rvv_get_vreg(proc, vlenb * 2, (&mut result).as_mut_ptr() as *mut u8, 16) };
    assert_eq!(err, 0);

    println!("result[0] = {}, result[1] = {}", result[0], result[1]);
    assert_eq!(result[0], 11);
    assert_eq!(result[1], 11);

    unsafe { rvv_delete_processor(proc) };
    println!("done");
}
