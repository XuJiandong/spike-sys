#![no_main]
use ckb_vm::CoreMachine;
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

    fn u16(&mut self) -> u16 {
        let mut b = [0; 2];
        b.copy_from_slice(&self.n[self.i..self.i + 2]);
        let r = u16::from_le_bytes(b);
        self.i += 2;
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

fuzz_target!(|data: [u8; 512]| {
    let mut rand_data = [0u8; 2048];
    rand_data[0x000..0x200].copy_from_slice(&data);
    rand_data[0x200..0x400].copy_from_slice(&data);
    rand_data[0x400..0x600].copy_from_slice(&data);
    rand_data[0x600..0x800].copy_from_slice(&data);

    let mut rand = Rand::new(rand_data);
    let spike = Spike::new(128, 64, 0);
    let mut ckbvm =
        ckb_vm::DefaultMachineBuilder::new(ckb_vm::DefaultCoreMachine::<u64, ckb_vm::SparseMemory<u64>>::new(
            ckb_vm::ISA_IMC | ckb_vm::ISA_B | ckb_vm::ISA_V,
            ckb_vm::machine::VERSION1,
            u64::MAX,
        ))
        .build();

    // Set vtype
    match rand.u8() % 3 {
        // vsetivli
        0 => {
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
        }
        // vsetvli
        1 => {
            let mut insn: u32 = 0b0_00000000000_00100_111_00101_1010111;
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
            spike.set_xreg(4, insn_immediate_u as u64).unwrap();
            ckbvm.set_register(4, insn_immediate_u as u64);
            insn |= insn_sew << 23;
            insn |= insn_lmul << 20;
            spike.execute(insn as u64).unwrap();
            let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
            ckb_vm::instructions::execute_instruction(insn, &mut ckbvm).unwrap();
        }
        // vsetvl
        2 => {
            let insn: u32 = 0b1000000_00011_00100_111_00101_1010111;
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
            spike.set_xreg(4, insn_immediate_u as u64).unwrap();
            ckbvm.set_register(4, insn_immediate_u as u64);
            spike.set_xreg(3, (insn_sew << 3 | insn_lmul) as u64).unwrap();
            ckbvm.set_register(3, (insn_sew << 3 | insn_lmul) as u64);
            spike.execute(insn as u64).unwrap();
            let insn = ckb_vm::instructions::v::factory::<u64>(insn, ckb_vm::machine::VERSION1).unwrap();
            ckb_vm::instructions::execute_instruction(insn, &mut ckbvm).unwrap();
        }
        _ => unreachable!(),
    }
    assert_eq!(spike.get_vill(), 0);
    assert_eq!(ckbvm.vill(), false);
    let spike_sew = spike.get_sew();
    let ckbvm_sew = ckbvm.vsew();
    assert_eq!(spike_sew, ckbvm_sew);
    let spike_vl = spike.get_vl();
    let ckbvm_vl = ckbvm.vl();
    assert_eq!(spike_vl, ckbvm_vl);

    // Set v register
    for i in 0..32 {
        let buf = rand.data(16);
        spike.set_vreg(16 * i, buf.as_ptr() as *const u8, 16).unwrap();
        ckbvm.element_mut(i as usize, 128, 0).copy_from_slice(buf);
    }
    // Set x register
    for i in 1..32 {
        let buf = rand.u64();
        spike.set_xreg(i, buf).unwrap();
        ckbvm.set_register(i as usize, buf);
    }

    #[rustfmt::skip]
    let insn_list = [
        [0b000000_1_11111_11111_000_11111_0000000, 0b000000_0_00000_00000_000_00000_1010111], // vadd.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b000000_0_00000_00000_100_00000_1010111], // vadd.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b000000_0_00000_00000_011_00000_1010111], // vadd.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b000010_0_00000_00000_000_00000_1010111], // vsub.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b000010_0_00000_00000_100_00000_1010111], // vsub.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b000011_0_00000_00000_100_00000_1010111], // vrsub.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b000011_0_00000_00000_011_00000_1010111], // vrsub.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b110000_0_00000_00000_010_00000_1010111], // vwaddu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b110000_0_00000_00000_110_00000_1010111], // vwaddu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b110010_0_00000_00000_010_00000_1010111], // vwsubu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b110010_0_00000_00000_110_00000_1010111], // vwsubu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b110001_0_00000_00000_010_00000_1010111], // vwadd.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b110001_0_00000_00000_110_00000_1010111], // vwadd.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b110011_0_00000_00000_010_00000_1010111], // vwsub.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b110011_0_00000_00000_110_00000_1010111], // vwsub.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b110100_0_00000_00000_010_00000_1010111], // vwaddu.wv
        [0b000000_1_11111_11111_000_11111_0000000, 0b110100_0_00000_00000_110_00000_1010111], // vwaddu.wx
        [0b000000_1_11111_11111_000_11111_0000000, 0b110110_0_00000_00000_010_00000_1010111], // vwsubu.wv
        [0b000000_1_11111_11111_000_11111_0000000, 0b110110_0_00000_00000_110_00000_1010111], // vwsubu.wx
        [0b000000_1_11111_11111_000_11111_0000000, 0b110101_0_00000_00000_010_00000_1010111], // vwadd.wv
        [0b000000_1_11111_11111_000_11111_0000000, 0b110101_0_00000_00000_110_00000_1010111], // vwadd.wx
        [0b000000_1_11111_11111_000_11111_0000000, 0b110111_0_00000_00000_010_00000_1010111], // vwsub.wv
        [0b000000_1_11111_11111_000_11111_0000000, 0b110111_0_00000_00000_110_00000_1010111], // vwsub.wx
        [0b000000_1_11111_00000_000_11111_0000000, 0b010010_0_00000_00110_010_00000_1010111], // vzext.vf2
        [0b000000_1_11111_00000_000_11111_0000000, 0b010010_0_00000_00100_010_00000_1010111], // vzext.vf4
        [0b000000_1_11111_00000_000_11111_0000000, 0b010010_0_00000_00010_010_00000_1010111], // vzext.vf8
        [0b000000_1_11111_00000_000_11111_0000000, 0b010010_0_00000_00111_010_00000_1010111], // vsext.vf2
        [0b000000_1_11111_00000_000_11111_0000000, 0b010010_0_00000_00101_010_00000_1010111], // vsext.vf4
        [0b000000_1_11111_00000_000_11111_0000000, 0b010010_0_00000_00011_010_00000_1010111], // vsext.vf8
        [0b000000_0_11111_11111_000_11111_0000000, 0b010000_0_00000_00000_000_00000_1010111], // vadc.vvm
        [0b000000_0_11111_11111_000_11111_0000000, 0b010000_0_00000_00000_100_00000_1010111], // vadc.vxm
        [0b000000_0_11111_11111_000_11111_0000000, 0b010000_0_00000_00000_011_00000_1010111], // vadc.vim
        [0b000000_1_11111_11111_000_11111_0000000, 0b010001_0_00000_00000_000_00000_1010111], // vmadc.vvm
        [0b000000_1_11111_11111_000_11111_0000000, 0b010001_0_00000_00000_100_00000_1010111], // vmadc.vxm
        [0b000000_1_11111_11111_000_11111_0000000, 0b010001_0_00000_00000_011_00000_1010111], // vmadc.vim
        [0b000000_0_11111_11111_000_11111_0000000, 0b010001_1_00000_00000_000_00000_1010111], // vmadc.vv
        [0b000000_0_11111_11111_000_11111_0000000, 0b010001_1_00000_00000_100_00000_1010111], // vmadc.vx
        [0b000000_0_11111_11111_000_11111_0000000, 0b010001_1_00000_00000_011_00000_1010111], // vmadc.vi
        [0b000000_0_11111_11111_000_11111_0000000, 0b010010_0_00000_00000_000_00000_1010111], // vsbc.vvm
        [0b000000_0_11111_11111_000_11111_0000000, 0b010010_0_00000_00000_100_00000_1010111], // vsbc.vxm
        [0b000000_1_11111_11111_000_11111_0000000, 0b010011_0_00000_00000_000_00000_1010111], // vmsbc.vvm
        [0b000000_1_11111_11111_000_11111_0000000, 0b010011_0_00000_00000_100_00000_1010111], // vmsbc.vxm
        [0b000000_0_11111_11111_000_11111_0000000, 0b010010_0_00000_00000_000_00000_1010111], // vsbc.vvm
        [0b000000_0_11111_11111_000_11111_0000000, 0b010010_0_00000_00000_100_00000_1010111], // vsbc.vxm
        [0b000000_1_11111_11111_000_11111_0000000, 0b010011_0_00000_00000_000_00000_1010111], // vmsbc.vvm
        [0b000000_1_11111_11111_000_11111_0000000, 0b010011_0_00000_00000_100_00000_1010111], // vmsbc.vxm
        [0b000000_0_11111_11111_000_11111_0000000, 0b010011_1_00000_00000_000_00000_1010111], // vmsbc.vv
        [0b000000_0_11111_11111_000_11111_0000000, 0b010011_1_00000_00000_100_00000_1010111], // vmsbc.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001001_0_00000_00000_000_00000_1010111], // vand.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b001001_0_00000_00000_100_00000_1010111], // vand.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001001_0_00000_00000_011_00000_1010111], // vand.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b001010_0_00000_00000_000_00000_1010111], // vor.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b001010_0_00000_00000_100_00000_1010111], // vor.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001010_0_00000_00000_011_00000_1010111], // vor.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b001011_0_00000_00000_000_00000_1010111], // vxor.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b001011_0_00000_00000_100_00000_1010111], // vxor.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001011_0_00000_00000_011_00000_1010111], // vxor.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b100101_0_00000_00000_000_00000_1010111], // vsll.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100101_0_00000_00000_100_00000_1010111], // vsll.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100101_0_00000_00000_011_00000_1010111], // vsll.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b101000_0_00000_00000_000_00000_1010111], // vsrl.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101000_0_00000_00000_100_00000_1010111], // vsrl.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101000_0_00000_00000_011_00000_1010111], // vsrl.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b101001_0_00000_00000_000_00000_1010111], // vsra.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101001_0_00000_00000_100_00000_1010111], // vsra.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101001_0_00000_00000_011_00000_1010111], // vsra.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b101100_0_00000_00000_000_00000_1010111], // vnsrl.wv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101100_0_00000_00000_100_00000_1010111], // vnsrl.wx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101100_0_00000_00000_011_00000_1010111], // vnsrl.wi
        [0b000000_1_11111_11111_000_11111_0000000, 0b101101_0_00000_00000_000_00000_1010111], // vnsra.wv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101101_0_00000_00000_100_00000_1010111], // vnsra.wx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101101_0_00000_00000_011_00000_1010111], // vnsra.wi
        [0b000000_1_11111_11111_000_11111_0000000, 0b011000_0_00000_00000_000_00000_1010111], // vmseq.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b011000_0_00000_00000_100_00000_1010111], // vmseq.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b011000_0_00000_00000_011_00000_1010111], // vmseq.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b011001_0_00000_00000_000_00000_1010111], // vmsne.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b011001_0_00000_00000_100_00000_1010111], // vmsne.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b011001_0_00000_00000_011_00000_1010111], // vmsne.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b011010_0_00000_00000_000_00000_1010111], // vmsltu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b011010_0_00000_00000_100_00000_1010111], // vmsltu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b011011_0_00000_00000_000_00000_1010111], // vmslt.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b011011_0_00000_00000_100_00000_1010111], // vmslt.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b011100_0_00000_00000_000_00000_1010111], // vmsleu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b011100_0_00000_00000_100_00000_1010111], // vmsleu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b011100_0_00000_00000_011_00000_1010111], // vmsleu.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b011101_0_00000_00000_000_00000_1010111], // vmsle.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b011101_0_00000_00000_100_00000_1010111], // vmsle.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b011101_0_00000_00000_011_00000_1010111], // vmsle.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b011110_0_00000_00000_100_00000_1010111], // vmsgtu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b011110_0_00000_00000_011_00000_1010111], // vmsgtu.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b011111_0_00000_00000_100_00000_1010111], // vmsgt.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b011111_0_00000_00000_011_00000_1010111], // vmsgt.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b000100_0_00000_00000_000_00000_1010111], // vminu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b000100_0_00000_00000_100_00000_1010111], // vminu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b000101_0_00000_00000_000_00000_1010111], // vmin.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b000101_0_00000_00000_100_00000_1010111], // vmin.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b000110_0_00000_00000_000_00000_1010111], // vmaxu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b000110_0_00000_00000_100_00000_1010111], // vmaxu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b000111_0_00000_00000_000_00000_1010111], // vmax.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b000111_0_00000_00000_100_00000_1010111], // vmax.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100101_0_00000_00000_010_00000_1010111], // vmul.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100101_0_00000_00000_110_00000_1010111], // vmul.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100111_0_00000_00000_010_00000_1010111], // vmulh.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100111_0_00000_00000_110_00000_1010111], // vmulh.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100100_0_00000_00000_010_00000_1010111], // vmulhu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100100_0_00000_00000_110_00000_1010111], // vmulhu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100110_0_00000_00000_010_00000_1010111], // vmulhsu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100110_0_00000_00000_110_00000_1010111], // vmulhsu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100000_0_00000_00000_010_00000_1010111], // vdivu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100000_0_00000_00000_110_00000_1010111], // vdivu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100001_0_00000_00000_010_00000_1010111], // vdiv.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100001_0_00000_00000_110_00000_1010111], // vdiv.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100010_0_00000_00000_010_00000_1010111], // vremu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100010_0_00000_00000_110_00000_1010111], // vremu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100011_0_00000_00000_010_00000_1010111], // vrem.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100011_0_00000_00000_110_00000_1010111], // vrem.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b111000_0_00000_00000_010_00000_1010111], // vwmulu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b111000_0_00000_00000_110_00000_1010111], // vwmulu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b111010_0_00000_00000_010_00000_1010111], // vwmulsu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b111010_0_00000_00000_110_00000_1010111], // vwmulsu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b111011_0_00000_00000_010_00000_1010111], // vwmul.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b111011_0_00000_00000_110_00000_1010111], // vwmul.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101101_0_00000_00000_010_00000_1010111], // vmacc.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101101_0_00000_00000_110_00000_1010111], // vmacc.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101111_0_00000_00000_010_00000_1010111], // vnmsac.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101111_0_00000_00000_110_00000_1010111], // vnmsac.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101001_0_00000_00000_010_00000_1010111], // vmadd.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101001_0_00000_00000_110_00000_1010111], // vmadd.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101011_0_00000_00000_010_00000_1010111], // vnmsub.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101011_0_00000_00000_110_00000_1010111], // vnmsub.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b111100_0_00000_00000_010_00000_1010111], // vwmaccu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b111100_0_00000_00000_110_00000_1010111], // vwmaccu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b111101_0_00000_00000_010_00000_1010111], // vwmacc.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b111101_0_00000_00000_110_00000_1010111], // vwmacc.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b111111_0_00000_00000_010_00000_1010111], // vwmaccsu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b111111_0_00000_00000_110_00000_1010111], // vwmaccsu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b111110_0_00000_00000_110_00000_1010111], // vwmaccus.vx
        [0b000000_0_11111_11111_000_11111_0000000, 0b010111_0_00000_00000_000_00000_1010111], // vmerge.vvm
        [0b000000_0_11111_11111_000_11111_0000000, 0b010111_0_00000_00000_100_00000_1010111], // vmerge.vxm
        [0b000000_0_11111_11111_000_11111_0000000, 0b010111_0_00000_00000_011_00000_1010111], // vmerge.vim
        [0b000000_0_00000_11111_000_11111_0000000, 0b010111_1_00000_00000_000_00000_1010111], // vmv.v.v
        [0b000000_0_00000_11111_000_11111_0000000, 0b010111_1_00000_00000_100_00000_1010111], // vmv.v.x
        [0b000000_0_00000_11111_000_11111_0000000, 0b010111_1_00000_00000_011_00000_1010111], // vmv.v.i
        [0b000000_1_11111_11111_000_11111_0000000, 0b100000_0_00000_00000_000_00000_1010111], // vsaddu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100000_0_00000_00000_100_00000_1010111], // vsaddu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100000_0_00000_00000_011_00000_1010111], // vsaddu.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b100001_0_00000_00000_000_00000_1010111], // vsadd.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100001_0_00000_00000_100_00000_1010111], // vsadd.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100001_0_00000_00000_011_00000_1010111], // vsadd.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b100010_0_00000_00000_000_00000_1010111], // vssubu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100010_0_00000_00000_100_00000_1010111], // vssubu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100011_0_00000_00000_000_00000_1010111], // vssub.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100011_0_00000_00000_100_00000_1010111], // vssub.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001001_0_00000_00000_010_00000_1010111], // vaadd.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b001001_0_00000_00000_110_00000_1010111], // vaadd.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001000_0_00000_00000_010_00000_1010111], // vaaddu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b001000_0_00000_00000_110_00000_1010111], // vaaddu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001011_0_00000_00000_010_00000_1010111], // vasub.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b001011_0_00000_00000_110_00000_1010111], // vasub.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001010_0_00000_00000_010_00000_1010111], // vasubu.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b001010_0_00000_00000_110_00000_1010111], // vasubu.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b100111_0_00000_00000_000_00000_1010111], // vsmul.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b100111_0_00000_00000_100_00000_1010111], // vsmul.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101010_0_00000_00000_000_00000_1010111], // vssrl.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101010_0_00000_00000_100_00000_1010111], // vssrl.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101010_0_00000_00000_011_00000_1010111], // vssrl.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b101011_0_00000_00000_000_00000_1010111], // vssra.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101011_0_00000_00000_100_00000_1010111], // vssra.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101011_0_00000_00000_011_00000_1010111], // vssra.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b101110_0_00000_00000_000_00000_1010111], // vnclipu.wv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101110_0_00000_00000_100_00000_1010111], // vnclipu.wx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101110_0_00000_00000_011_00000_1010111], // vnclipu.wi
        [0b000000_1_11111_11111_000_11111_0000000, 0b101111_0_00000_00000_000_00000_1010111], // vnclip.wv
        [0b000000_1_11111_11111_000_11111_0000000, 0b101111_0_00000_00000_100_00000_1010111], // vnclip.wx
        [0b000000_1_11111_11111_000_11111_0000000, 0b101111_0_00000_00000_011_00000_1010111], // vnclip.wi
        [0b000000_1_11111_11111_000_11111_0000000, 0b000000_0_00000_00000_010_00000_1010111], // vredsum.vs
        [0b000000_1_11111_11111_000_11111_0000000, 0b000001_0_00000_00000_010_00000_1010111], // vredand.vs
        [0b000000_1_11111_11111_000_11111_0000000, 0b000010_0_00000_00000_010_00000_1010111], // vredor.vs
        [0b000000_1_11111_11111_000_11111_0000000, 0b000011_0_00000_00000_010_00000_1010111], // vredxor.vs
        [0b000000_1_11111_11111_000_11111_0000000, 0b000100_0_00000_00000_010_00000_1010111], // vredminu.vs
        [0b000000_1_11111_11111_000_11111_0000000, 0b000101_0_00000_00000_010_00000_1010111], // vredmin.vs
        [0b000000_1_11111_11111_000_11111_0000000, 0b000110_0_00000_00000_010_00000_1010111], // vredmaxu.vs
        [0b000000_1_11111_11111_000_11111_0000000, 0b000111_0_00000_00000_010_00000_1010111], // vredmax.vs
        [0b000000_1_11111_11111_000_11111_0000000, 0b110000_0_00000_00000_000_00000_1010111], // vwredsumu.vs
        [0b000000_1_11111_11111_000_11111_0000000, 0b110001_0_00000_00000_000_00000_1010111], // vwredsum.vs
        [0b000000_1_11111_11111_000_11111_0000000, 0b011001_0_00000_00000_010_00000_1010111], // vmand.mm
        [0b000000_1_11111_11111_000_11111_0000000, 0b011101_0_00000_00000_010_00000_1010111], // vmnand.mm
        [0b000000_1_11111_11111_000_11111_0000000, 0b011000_0_00000_00000_010_00000_1010111], // vmandnot.mm
        [0b000000_1_11111_11111_000_11111_0000000, 0b011011_0_00000_00000_010_00000_1010111], // vmxor.mm
        [0b000000_1_11111_11111_000_11111_0000000, 0b011010_0_00000_00000_010_00000_1010111], // vmor.mm
        [0b000000_1_11111_11111_000_11111_0000000, 0b011110_0_00000_00000_010_00000_1010111], // vmnor.mm
        [0b000000_1_11111_11111_000_11111_0000000, 0b011100_0_00000_00000_010_00000_1010111], // vmornot.mm
        [0b000000_1_11111_11111_000_11111_0000000, 0b011111_0_00000_00000_010_00000_1010111], // vmxnor.mm
        [0b000000_1_11111_00000_000_11111_0000000, 0b010000_0_00000_10000_010_00000_1010111], // vpopc.m
        [0b000000_1_11111_00000_000_11111_0000000, 0b010000_0_00000_10001_010_00000_1010111], // vfirst.m
        [0b000000_1_11111_00000_000_11111_0000000, 0b010100_0_00000_00001_010_00000_1010111], // vmsbf.m
        [0b000000_1_11111_00000_000_11111_0000000, 0b010100_0_00000_00011_010_00000_1010111], // vmsif.m
        [0b000000_1_11111_00000_000_11111_0000000, 0b010100_0_00000_00010_010_00000_1010111], // vmsof.m
        [0b000000_1_11111_00000_000_11111_0000000, 0b010100_0_00000_10000_010_00000_1010111], // viota.m
        [0b000000_1_00000_00000_000_11111_0000000, 0b010100_0_00000_10001_010_00000_1010111], // vid.v
        [0b000000_0_11111_00000_000_11111_0000000, 0b010000_1_00000_00000_010_00000_1010111], // vmv.x.s
        [0b000000_0_00000_11111_000_11111_0000000, 0b010000_1_00000_00000_110_00000_1010111], // vmv.s.x
        [0b000000_1_11111_11111_000_11111_0000000, 0b001110_0_00000_00000_100_00000_1010111], // vslideup.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001110_0_00000_00000_011_00000_1010111], // vslideup.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b001110_0_00000_00000_110_00000_1010111], // vslide1up.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001111_0_00000_00000_100_00000_1010111], // vslidedown.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001111_0_00000_00000_011_00000_1010111], // vslidedown.vi
        [0b000000_1_11111_11111_000_11111_0000000, 0b001111_0_00000_00000_110_00000_1010111], // vslide1down.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001100_0_00000_00000_000_00000_1010111], // vrgather.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b001110_0_00000_00000_000_00000_1010111], // vrgatherei16.vv
        [0b000000_1_11111_11111_000_11111_0000000, 0b001100_0_00000_00000_100_00000_1010111], // vrgather.vx
        [0b000000_1_11111_11111_000_11111_0000000, 0b001100_0_00000_00000_011_00000_1010111], // vrgather.vi
        [0b000000_0_11111_11111_000_11111_0000000, 0b010111_1_00000_00000_010_00000_1010111], // vcompress.vm
        [0b000000_0_11111_00000_000_11111_0000000, 0b100111_1_00000_00000_011_00000_1010111], // vmv1r.v
        [0b000000_0_11111_00000_000_11111_0000000, 0b100111_1_00000_00001_011_00000_1010111], // vmv2r.v
        [0b000000_0_11111_00000_000_11111_0000000, 0b100111_1_00000_00011_011_00000_1010111], // vmv4r.v
        [0b000000_0_11111_00000_000_11111_0000000, 0b100111_1_00000_00111_011_00000_1010111], // vmv8r.v
    ];

    for _ in 0..128 {
        // Execute random instruction
        let insn_choose = rand.u16() as usize % insn_list.len();
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
    for i in 0..32 {
        let spike_xreg = spike.get_xreg(i).unwrap();
        let ckbvm_xreg = ckbvm.registers()[i as usize];
        assert_eq!(spike_xreg, ckbvm_xreg);
    }
});
