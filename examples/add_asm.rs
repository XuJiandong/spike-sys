use ckb_vm::CoreMachine;
use spike_sys::*;

pub fn main() {
    let proc = unsafe { rvv_new_processor(128, 64, 0) };
    let vlen = unsafe { rvv_get_vlen(proc) };
    let vlenb = vlen / 8;

    let value_one: [u64; 2] = [1, 1];
    let mut result: [u64; 2] = [0, 0];

    let insn: u64 = rvv_as("vsetivli t0, 8, e64, m1") as u64;
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    let err = unsafe { rvv_set_vreg(proc, vlenb * 10, (&value_one).as_ptr() as *const u8, 16) };
    assert_eq!(err, 0);
    let err = unsafe { rvv_set_vreg(proc, vlenb * 20, (&value_one).as_ptr() as *const u8, 16) };
    assert_eq!(err, 0);

    let insn = rvv_as("vadd.vv v2, v10, v20") as u64;
    let err = unsafe { rvv_execute(proc, insn) };
    assert_eq!(err, 0);

    let err = unsafe { rvv_get_vreg(proc, vlenb * 2, (&mut result).as_mut_ptr() as *mut u8, 16) };
    assert_eq!(err, 0);
    assert_eq!(result[0], 2);
    assert_eq!(result[1], 2);

    unsafe { rvv_delete_processor(proc) };

    let core_machine = ckb_vm::DefaultCoreMachine::<u64, ckb_vm::SparseMemory<u64>>::new(
        ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_V,
        ckb_vm::machine::VERSION1,
        u64::MAX,
    );
    let machine_builder = ckb_vm::DefaultMachineBuilder::new(core_machine)
        .instruction_cycle_func(Box::new(|_, _, _, _| 1));
    let mut machine = machine_builder.build();

    let insn = rvv_as("vsetivli t0, 8, e64, m1");
    let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
    ckb_vm::instructions::execute_instruction(insn, &mut machine).unwrap();

    machine.element_mut(10, 128, 0).copy_from_slice(unsafe {
        std::slice::from_raw_parts(value_one.as_ptr() as *const u8, 16)
    });
    machine.element_mut(20, 128, 0).copy_from_slice(unsafe {
        std::slice::from_raw_parts(value_one.as_ptr() as *const u8, 16)
    });

    let insn = rvv_as("vadd.vv v2, v10, v20");
    let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
    ckb_vm::instructions::execute_instruction(insn, &mut machine).unwrap();

    println!("{:?}", machine.element_ref(2, 64, 0));
    println!("{:?}", machine.element_ref(2, 64, 1));
}
