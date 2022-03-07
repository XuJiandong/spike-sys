#!/bin/bash

sed -i '/softfloat_install_shared_lib/d' deps/riscv-isa-sim/softfloat/softfloat.mk.in
sed -i 's/private:/public:/g' deps/riscv-isa-sim/riscv/processor.h

mkdir -p deps/riscv-isa-sim/build
cd deps/riscv-isa-sim/build
../configure CXX=clang++ CC=clang
make -j ${JOBS:4}
cd -
