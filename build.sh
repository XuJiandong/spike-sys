#!/bin/bash

sed -i '/softfloat_install_shared_lib/d' deps/riscv-isa-sim/softfloat/softfloat.mk.in
sed -i 's/private:/public:/g' deps/riscv-isa-sim/riscv/processor.h
sed -i 's/private:/public:/g' deps/riscv-isa-sim/riscv/mmu.h

mkdir -p deps/riscv-isa-sim/build
cd deps/riscv-isa-sim/build
../configure CXX=clang++ CC=clang CFLAGS="-g -O1" CXXFLAGS="-g -O1"
make -j$(nproc)
cd -
