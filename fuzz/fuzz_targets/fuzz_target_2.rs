#![no_main]
use ckb_vm::{CoreMachine, Memory};
use libfuzzer_sys::fuzz_target;
use spike_sys::*;

struct Rand {
    n: [u8; 2048],
    i: usize,
}

impl Rand {
    fn new(n: [u8; 2048]) -> Self {
        Self { n, i: 0 }
    }

    fn u8(&mut self) -> u8 {
        let r = self.n[self.i];
        self.i += 1;
        r
    }

    fn u32(&mut self) -> u32 {
        let mut b = [0; 4];
        b.copy_from_slice(&self.n[self.i..self.i + 4]);
        let r = u32::from_le_bytes(b);
        self.i += 4;
        r
    }

    fn u64(&mut self) -> u64 {
        let mut b = [0; 8];
        b.copy_from_slice(&self.n[self.i..self.i + 8]);
        let r = u64::from_le_bytes(b);
        self.i += 8;
        r
    }

    fn data(&mut self, n: usize) -> &[u8] {
        let r = &self.n[self.i..self.i + n];
        self.i += n;
        r
    }
}

fn fuzz_unit_stride(data: [u8; 2048]) {
    let mut rand = Rand::new(data.clone());
    let spike = Spike::new(128, 64, 8192);
    let mut ckbvm =
        ckb_vm::DefaultMachineBuilder::new(ckb_vm::DefaultCoreMachine::<u64, ckb_vm::SparseMemory<u64>>::new(
            ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_V,
            ckb_vm::machine::VERSION1,
            u64::MAX,
        ))
        .build();

    // Set vtype
    let mut insn: u32 = 0b11_0000000000_00000_111_00101_1010111;
    let insn_sew = rand.u8() as u32 & 0b11;
    let insn_immediate_u = rand.u8() as u32 % (128 / (1 << (insn_sew + 3)) + 1);
    let insn_lmul_mod = match 1 << (insn_sew + 3) {
        64 => 4,
        32 => 5,
        16 => 6,
        _ => 7,
    };
    let insn_lmul_array: [u32; 7] = [0b000, 0b001, 0b010, 0b011, 0b111, 0b110, 0b101]; // [1 2 4 8 0.5 0.25 0.125]
    let insn_lmul: u32 = insn_lmul_array[rand.u8() as usize % insn_lmul_mod];
    insn |= insn_immediate_u << 15;
    insn |= insn_sew << 23;
    insn |= insn_lmul << 20;
    spike.execute(insn as u64).unwrap();
    let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
    ckb_vm::instructions::execute_instruction(insn, &mut ckbvm).unwrap();
    assert_eq!(spike.get_vill(), 0);
    assert_eq!(ckbvm.vill(), false);
    let spike_sew = spike.get_sew();
    let ckbvm_sew = ckbvm.vsew();
    assert_eq!(spike_sew, ckbvm_sew);
    let spike_vl = spike.get_vl();
    let ckbvm_vl = ckbvm.vl();
    assert_eq!(spike_vl, ckbvm_vl);

    // Set memory
    spike.store_mem(4096, 2048, data.as_ptr() as *const u8).unwrap();
    ckbvm.memory_mut().store_bytes(4096, &data[..]).unwrap();
    // Set v register
    for i in 0..32 {
        let buf = rand.data(16);
        spike.set_vreg(16 * i, buf.as_ptr() as *const u8, 16).unwrap();
        ckbvm.element_mut(i as usize, 128, 0).copy_from_slice(buf);
    }
    // Set x register
    for i in 1..32 {
        spike.set_xreg(i, 4096).unwrap();
        ckbvm.set_register(i as usize, 4096);
    }

    for _ in 0..128 {
        #[rustfmt::skip]
        let insn_list = [
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_000_0_00000_00001_000_00000_0000111], // vle8.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_000_0_00000_00001_101_00000_0000111], // vle16.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_000_0_00000_00001_110_00000_0000111], // vle32.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_000_0_00000_00001_111_00000_0000111], // vle64.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_000_0_00000_00001_000_00000_0100111], // vse8.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_000_0_00000_00001_101_00000_0100111], // vse16.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_000_0_00000_00001_110_00000_0100111], // vse32.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_000_0_00000_00001_111_00000_0100111], // vse64.v
            [0b000_000_0_00000_00000_000_11111_0000000, 0b000_000_1_01011_00001_000_00000_0000111], // vlm.v
            [0b000_000_0_00000_00000_000_11111_0000000, 0b000_000_1_01011_00001_000_00000_0100111], // vsm.v
        ];

        // Execute random instruction
        let insn_choose = rand.u8() as usize % insn_list.len();
        let mask = insn_list[insn_choose];
        let insn = rand.u32() & mask[0] | mask[1];
        if std::env::var("LOG").is_ok() {
            println!(
                "sew={:?} lmul={:?} vl={:?} insn_choose=0x{:x} insn=0x{:x}",
                ckbvm_sew,
                ckbvm.vlmul(),
                ckbvm_vl,
                insn_choose,
                insn
            );
        }
        let err = spike.execute(insn as u64);
        let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
        let r = ckb_vm::instructions::execute_instruction(insn, &mut ckbvm);
        assert_eq!(err.is_ok(), r.is_ok());
    }

    // Check result
    let mut spike_vd = [0x00; 16];
    let mut ckbvm_vd = [0x00; 16];
    for i in 0..32 {
        spike.get_vreg(16 * i, (&mut spike_vd).as_mut_ptr() as *mut u8, 16).unwrap();
        ckbvm_vd.copy_from_slice(ckbvm.element_ref(i as usize, 128, 0));
        assert_eq!(spike_vd, ckbvm_vd);
    }
}

fn fuzz_stride(data: [u8; 2048]) {
    let mut rand = Rand::new(data.clone());
    let spike = Spike::new(128, 64, 8182);
    let mut ckbvm =
        ckb_vm::DefaultMachineBuilder::new(ckb_vm::DefaultCoreMachine::<u64, ckb_vm::SparseMemory<u64>>::new(
            ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_V,
            ckb_vm::machine::VERSION1,
            u64::MAX,
        ))
        .build();

    // Set vtype
    let mut insn: u32 = 0b11_0000000000_00000_111_00101_1010111;
    let insn_sew = rand.u8() as u32 & 0b11;
    let insn_immediate_u = rand.u8() as u32 % (128 / (1 << (insn_sew + 3)) + 1);
    let insn_lmul_mod = match 1 << (insn_sew + 3) {
        64 => 4,
        32 => 5,
        16 => 6,
        _ => 7,
    };
    let insn_lmul_array: [u32; 7] = [0b000, 0b001, 0b010, 0b011, 0b111, 0b110, 0b101]; // [1 2 4 8 0.5 0.25 0.125]
    let insn_lmul: u32 = insn_lmul_array[rand.u8() as usize % insn_lmul_mod];
    insn |= insn_immediate_u << 15;
    insn |= insn_sew << 23;
    insn |= insn_lmul << 20;
    spike.execute(insn as u64).unwrap();
    let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
    ckb_vm::instructions::execute_instruction(insn, &mut ckbvm).unwrap();
    assert_eq!(spike.get_vill(), 0);
    assert_eq!(ckbvm.vill(), false);
    let spike_sew = spike.get_sew();
    let ckbvm_sew = ckbvm.vsew();
    assert_eq!(spike_sew, ckbvm_sew);
    let spike_vl = spike.get_vl();
    let ckbvm_vl = ckbvm.vl();
    assert_eq!(spike_vl, ckbvm_vl);

    // Set memory
    spike.store_mem(4096, 2048, data.as_ptr() as *const u8).unwrap();
    ckbvm.memory_mut().store_bytes(4096, &data[..]).unwrap();
    // Set v register
    for i in 0..32 {
        let buf = rand.data(16);
        spike.set_vreg(16 * i, buf.as_ptr() as *const u8, 16).unwrap();
        ckbvm.element_mut(i as usize, 128, 0).copy_from_slice(buf);
    }
    // Set x register
    spike.set_xreg(1, 4096 + 512).unwrap();
    ckbvm.set_register(1, 4096 + 512);
    let rs2 = 0;
    spike.set_xreg(2, rs2).unwrap();
    ckbvm.set_register(2, rs2);

    for _ in 0..128 {
        #[rustfmt::skip]
        let insn_list = [
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_010_0_00010_00001_000_00000_0000111], // vlse8.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_010_0_00010_00001_101_00000_0000111], // vlse16.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_010_0_00010_00001_110_00000_0000111], // vlse32.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_010_0_00010_00001_111_00000_0000111], // vlse64.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_010_0_00010_00001_000_00000_0100111], // vsse8.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_010_0_00010_00001_101_00000_0100111], // vsse16.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_010_0_00010_00001_110_00000_0100111], // vsse32.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_010_0_00010_00001_111_00000_0100111], // vsse64.v
        ];

        // Execute random instruction
        let insn_choose = rand.u8() as usize % insn_list.len();
        let mask = insn_list[insn_choose];
        let insn = rand.u32() & mask[0] | mask[1];
        if std::env::var("LOG").is_ok() {
            println!(
                "sew={:?} lmul={:?} vl={:?} insn_choose=0x{:x} insn=0x{:x}",
                ckbvm_sew,
                ckbvm.vlmul(),
                ckbvm_vl,
                insn_choose,
                insn
            );
        }
        let err = spike.execute(insn as u64);
        let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
        let r = ckb_vm::instructions::execute_instruction(insn, &mut ckbvm);
        assert_eq!(err.is_ok(), r.is_ok());
    }

    // Check result
    let mut spike_vd = [0x00; 16];
    let mut ckbvm_vd = [0x00; 16];
    for i in 0..32 {
        spike.get_vreg(16 * i, (&mut spike_vd).as_mut_ptr() as *mut u8, 16).unwrap();
        ckbvm_vd.copy_from_slice(ckbvm.element_ref(i as usize, 128, 0));
        assert_eq!(spike_vd, ckbvm_vd);
    }
}

fn fuzz_indexed(data: [u8; 2048]) {
    let mut rand = Rand::new(data.clone());
    let spike = Spike::new(128, 64, 8192);
    let mut ckbvm =
        ckb_vm::DefaultMachineBuilder::new(ckb_vm::DefaultCoreMachine::<u64, ckb_vm::SparseMemory<u64>>::new(
            ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_V,
            ckb_vm::machine::VERSION1,
            u64::MAX,
        ))
        .build();

    // Set vtype
    let mut insn: u32 = 0b11_0000000000_00000_111_00101_1010111;
    let insn_sew = rand.u8() as u32 & 0b11;
    let insn_immediate_u = rand.u8() as u32 % (128 / (1 << (insn_sew + 3)) + 1);
    let insn_lmul_mod = match 1 << (insn_sew + 3) {
        64 => 4,
        32 => 5,
        16 => 6,
        _ => 7,
    };
    let insn_lmul_array: [u32; 7] = [0b000, 0b001, 0b010, 0b011, 0b111, 0b110, 0b101]; // [1 2 4 8 0.5 0.25 0.125]
    let insn_lmul: u32 = insn_lmul_array[rand.u8() as usize % insn_lmul_mod];
    insn |= insn_immediate_u << 15;
    insn |= insn_sew << 23;
    insn |= insn_lmul << 20;
    spike.execute(insn as u64).unwrap();
    let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
    ckb_vm::instructions::execute_instruction(insn, &mut ckbvm).unwrap();
    assert_eq!(spike.get_vill(), 0);
    assert_eq!(ckbvm.vill(), false);
    let spike_sew = spike.get_sew();
    let ckbvm_sew = ckbvm.vsew();
    assert_eq!(spike_sew, ckbvm_sew);
    let spike_vl = spike.get_vl();
    let ckbvm_vl = ckbvm.vl();
    assert_eq!(spike_vl, ckbvm_vl);

    // Set memory
    spike.store_mem(4096, 2048, data.as_ptr() as *const u8).unwrap();
    ckbvm.memory_mut().store_bytes(4096, &data[..]).unwrap();

    for _ in 0..8 {
        // Set x register
        spike.set_xreg(1, 4096).unwrap();
        ckbvm.set_register(1, 4096);

        #[rustfmt::skip]
        let insn_list = [
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_001_0_00010_00001_000_00000_0000111], // vluxei8.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_001_0_00010_00001_101_00000_0000111], // vluxei16.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_001_0_00010_00001_110_00000_0000111], // vluxei32.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_001_0_00010_00001_111_00000_0000111], // vluxei64.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_011_0_00010_00001_000_00000_0000111], // vloxei8.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_011_0_00010_00001_101_00000_0000111], // vloxei16.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_011_0_00010_00001_110_00000_0000111], // vloxei32.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_011_0_00010_00001_111_00000_0000111], // vloxei64.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_001_0_00010_00001_000_00000_0100111], // vsuxei8.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_001_0_00010_00001_101_00000_0100111], // vsuxei16.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_001_0_00010_00001_110_00000_0100111], // vsuxei32.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_001_0_00010_00001_111_00000_0100111], // vsuxei64.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_011_0_00010_00001_000_00000_0100111], // vsoxei8.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_011_0_00010_00001_101_00000_0100111], // vsoxei16.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_011_0_00010_00001_110_00000_0100111], // vsoxei32.v
            [0b000_000_1_00000_00000_000_11111_0000000, 0b000_011_0_00010_00001_111_00000_0100111], // vsoxei64.v
        ];

        let insn_choose = rand.u8() as usize % insn_list.len();
        let mask = insn_list[insn_choose];

        // Set v register
        for i in 0..ckbvm_vl {
            match (mask[1] >> 12) & 0b111 {
                0b000 => {
                    let offset = [0u64, 8][rand.u64() as usize % 2];
                    spike.set_vreg(32 + i * 1, (&offset) as *const u64 as *const u8, 1).unwrap();
                    let buf = unsafe { std::slice::from_raw_parts((&offset) as *const u64 as *const u8, 1) };
                    ckbvm.element_mut(2, 8, i as usize).copy_from_slice(buf);
                }
                0b101 => {
                    let offset = [0u64, 8][rand.u64() as usize % 2];
                    spike.set_vreg(32 + i * 2, (&offset) as *const u64 as *const u8, 2).unwrap();
                    let buf = unsafe { std::slice::from_raw_parts((&offset) as *const u64 as *const u8, 2) };
                    ckbvm.element_mut(2, 16, i as usize).copy_from_slice(buf);
                }
                0b110 => {
                    let offset = [0u64, 8][rand.u64() as usize % 2];
                    spike.set_vreg(32 + i * 4, (&offset) as *const u64 as *const u8, 4).unwrap();
                    let buf = unsafe { std::slice::from_raw_parts((&offset) as *const u64 as *const u8, 4) };
                    ckbvm.element_mut(2, 32, i as usize).copy_from_slice(buf);
                }
                0b111 => {
                    let offset = [0u64, 8][rand.u64() as usize % 2];
                    spike.set_vreg(32 + i * 8, (&offset) as *const u64 as *const u8, 8).unwrap();
                    let buf = unsafe { std::slice::from_raw_parts((&offset) as *const u64 as *const u8, 8) };
                    ckbvm.element_mut(2, 64, i as usize).copy_from_slice(buf);
                }
                _ => unreachable!(),
            }
        }

        // Execute random instruction
        let insn = rand.u32() & mask[0] | mask[1];
        if std::env::var("LOG").is_ok() {
            println!(
                "sew={:?} lmul={:?} vl={:?} insn_choose=0x{:x} insn=0x{:x}",
                ckbvm_sew,
                ckbvm.vlmul(),
                ckbvm_vl,
                insn_choose,
                insn
            );
        }
        let err = spike.execute(insn as u64);
        let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
        let r = ckb_vm::instructions::execute_instruction(insn, &mut ckbvm);
        assert_eq!(err.is_ok(), r.is_ok());
    }

    // Check result
    let mut spike_vd = [0x00; 16];
    let mut ckbvm_vd = [0x00; 16];
    for i in 0..32 {
        spike.get_vreg(16 * i, (&mut spike_vd).as_mut_ptr() as *mut u8, 16).unwrap();
        ckbvm_vd.copy_from_slice(ckbvm.element_ref(i as usize, 128, 0));
        assert_eq!(spike_vd, ckbvm_vd);
    }
}

fn fuzz_whole(data: [u8; 2048]) {
    let mut rand = Rand::new(data.clone());
    let spike = Spike::new(128, 64, 8192);
    let mut ckbvm =
        ckb_vm::DefaultMachineBuilder::new(ckb_vm::DefaultCoreMachine::<u64, ckb_vm::SparseMemory<u64>>::new(
            ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_V,
            ckb_vm::machine::VERSION1,
            u64::MAX,
        ))
        .build();

    // Set vtype
    let mut insn: u32 = 0b11_0000000000_00000_111_00101_1010111;
    let insn_sew = rand.u8() as u32 & 0b11;
    let insn_immediate_u = rand.u8() as u32 % (128 / (1 << (insn_sew + 3)) + 1);
    let insn_lmul_mod = match 1 << (insn_sew + 3) {
        64 => 4,
        32 => 5,
        16 => 6,
        _ => 7,
    };
    let insn_lmul_array: [u32; 7] = [0b000, 0b001, 0b010, 0b011, 0b111, 0b110, 0b101]; // [1 2 4 8 0.5 0.25 0.125]
    let insn_lmul: u32 = insn_lmul_array[rand.u8() as usize % insn_lmul_mod];
    insn |= insn_immediate_u << 15;
    insn |= insn_sew << 23;
    insn |= insn_lmul << 20;
    spike.execute(insn as u64).unwrap();
    let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
    ckb_vm::instructions::execute_instruction(insn, &mut ckbvm).unwrap();
    assert_eq!(spike.get_vill(), 0);
    assert_eq!(ckbvm.vill(), false);
    let spike_sew = spike.get_sew();
    let ckbvm_sew = ckbvm.vsew();
    assert_eq!(spike_sew, ckbvm_sew);
    let spike_vl = spike.get_vl();
    let ckbvm_vl = ckbvm.vl();
    assert_eq!(spike_vl, ckbvm_vl);

    // Set memory
    spike.store_mem(4096, 2048, data.as_ptr() as *const u8).unwrap();
    ckbvm.memory_mut().store_bytes(4096, &data[..]).unwrap();
    // Set v register
    for i in 0..32 {
        let buf = rand.data(16);
        spike.set_vreg(16 * i, buf.as_ptr() as *const u8, 16).unwrap();
        ckbvm.element_mut(i as usize, 128, 0).copy_from_slice(buf);
    }
    // Set x register
    spike.set_xreg(1, 4096).unwrap();
    ckbvm.set_register(1, 4096);

    for _ in 0..128 {
        #[rustfmt::skip]
        let insn_list = [
            [0b000_000_0_00000_00000_000_00000_0000000, 0b000_000_1_01000_00001_000_01000_0000111], // vl1re8.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b000_000_1_01000_00001_101_01000_0000111], // vl1re16.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b000_000_1_01000_00001_110_01000_0000111], // vl1re32.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b000_000_1_01000_00001_111_01000_0000111], // vl1re64.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b001_000_1_01000_00001_000_01000_0000111], // vl2re8.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b001_000_1_01000_00001_101_01000_0000111], // vl2re16.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b001_000_1_01000_00001_110_01000_0000111], // vl2re32.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b001_000_1_01000_00001_111_01000_0000111], // vl2re64.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b011_000_1_01000_00001_000_01000_0000111], // vl4re8.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b011_000_1_01000_00001_101_01000_0000111], // vl4re16.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b011_000_1_01000_00001_110_01000_0000111], // vl4re32.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b011_000_1_01000_00001_111_01000_0000111], // vl4re64.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b111_000_1_01000_00001_000_01000_0000111], // vl8re8.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b111_000_1_01000_00001_101_01000_0000111], // vl8re16.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b111_000_1_01000_00001_110_01000_0000111], // vl8re32.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b111_000_1_01000_00001_111_01000_0000111], // vl8re64.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b000_000_1_01000_00001_000_01000_0100111], // vs1r.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b001_000_1_01000_00001_000_01000_0100111], // vs2r.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b011_000_1_01000_00001_000_01000_0100111], // vs4r.v
            [0b000_000_0_00000_00000_000_00000_0000000, 0b111_000_1_01000_00001_000_01000_0100111], // vs8r.v
        ];

        // Execute random instruction
        let insn_choose = rand.u8() as usize % insn_list.len();
        let mask = insn_list[insn_choose];
        let insn = rand.u32() & mask[0] | mask[1];
        if std::env::var("LOG").is_ok() {
            println!(
                "sew={:?} lmul={:?} vl={:?} insn_choose=0x{:x} insn=0x{:x}",
                ckbvm_sew,
                ckbvm.vlmul(),
                ckbvm_vl,
                insn_choose,
                insn
            );
        }
        let err = spike.execute(insn as u64);
        let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
        let r = ckb_vm::instructions::execute_instruction(insn, &mut ckbvm);
        assert_eq!(err.is_ok(), r.is_ok());
    }

    // Check result
    let mut spike_vd = [0x00; 16];
    let mut ckbvm_vd = [0x00; 16];
    for i in 0..32 {
        spike.get_vreg(16 * i, (&mut spike_vd).as_mut_ptr() as *mut u8, 16).unwrap();
        ckbvm_vd.copy_from_slice(ckbvm.element_ref(i as usize, 128, 0));
        assert_eq!(spike_vd, ckbvm_vd);
    }
}

fuzz_target!(|data: [u8; 512]| {
    let mut rand_data = [0u8; 2048];
    rand_data[0x000..0x200].copy_from_slice(&data);
    rand_data[0x200..0x400].copy_from_slice(&data);
    rand_data[0x400..0x600].copy_from_slice(&data);
    rand_data[0x600..0x800].copy_from_slice(&data);
    fuzz_unit_stride(rand_data.clone());
    fuzz_stride(rand_data.clone());
    fuzz_indexed(rand_data.clone());
    fuzz_whole(rand_data.clone());
});
