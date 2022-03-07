# spike-sys
A Rust FFI binding for executing instructions in spike (https://github.com/riscv-software-src/riscv-isa-sim).
It's not a full binding to spike.

## How build

* Build spike (riscv-isa-sim) libraries
```bash
bash build.sh
```

* Build spike wrapping interfaces  
```bash
make
```

* Build rust project

```bash
cargo build
```

* Run example (Optional)
```bash
cargo run --example=add
```
