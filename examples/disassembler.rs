use std::{ffi::CStr, os::raw::c_char};

use spike_sys::*;

fn to_string(str: &[u8]) -> String {
    let p = unsafe {
        let p = str.as_ptr();
        CStr::from_ptr(p as *const c_char)
            .to_string_lossy()
            .into_owned()
    };
    p.clone()
}

pub fn main() {
    let dis = unsafe { rvv_new_disassembler(64) };
    assert!(dis != 0);
    let mut output: Vec<u8> = Vec::new();
    output.resize(128, 0);

    let inst: u64 = 0xc18472d7;
    let result =
        unsafe { rvv_disassemble(dis, inst, (&mut output).as_mut_ptr(), output.len() as u32) };
    assert_eq!(result, 0);
    let inst_str = to_string(&output);
    assert_eq!(inst_str, "vsetivli t0, 8, e64, m1, tu, mu");

    let inst: u64 = 0x02a2c157;
    let result =
        unsafe { rvv_disassemble(dis, inst, (&mut output).as_mut_ptr(), output.len() as u32) };
    assert_eq!(result, 0);
    let inst_str = to_string(&output);
    assert_eq!(inst_str, "vadd.vx v2, v10, t0");
}
