# before building it, run build.sh

CXX := clang++
CXXFLAGS := -fPIC -I./deps/riscv-isa-sim/riscv  -I./deps/riscv-isa-sim/build -I./deps/riscv-isa-sim/softfloat
LDFLAGS := -L'./deps/riscv-isa-sim/build' -lriscv -lsoftfloat -ldisasm -ldl

all: target/libspike-interfaces.a

target/spike-interfaces.o: cpp/spike-interfaces.cc cpp/spike-interfaces.h
	mkdir -p target
	$(CXX) -c $(CXXFLAGS) $< -o $@

target/libspike-interfaces.a: target/spike-interfaces.o
	ar rcs $@ $^

fmt:
	clang-format -i -style="{IndentWidth: 4, ColumnLimit: 140}" cpp/*.cc cpp/*.h

clean:
	rm -f target/libspike-interfaces.a target/spike-interfaces.o
