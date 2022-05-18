use spike_sys::*;

pub fn main() {
    let proc = unsafe { rvv_new_processor(2048, 1024, 0) };
    let vlen = unsafe { rvv_get_vlen(proc) };
    let elen = unsafe { rvv_get_elen(proc) };
    println!("vlen = {}, elen = {}", vlen, elen);

    let insn: u64 = 0xc28172d7; // vsetivli t0, 2, e256, m1
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    let vl = unsafe { rvv_get_vl(proc) };
    let sew = unsafe { rvv_get_sew(proc) };
    let lmul = unsafe { rvv_get_lmul(proc) };
    let vill = unsafe { rvv_get_vill(proc) };

    assert_eq!(vl, 2);
    assert_eq!(sew, 256);
    assert_eq!(lmul, 1.0);
    assert_eq!(vill, 0);

    println!(
        "vl = {}, sew = {}, lmul = {}, vill = {}",
        vl, sew, lmul, vill
    );
    unsafe { rvv_delete_processor(proc) };
    println!("done");
}
