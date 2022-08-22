## Fuzz

```sh
$ cargo fuzz run fuzz_alu     # arithmetic logic instructions
$ cargo fuzz run fuzz_mem     # memory store/load instructions
$ cargo fuzz run fuzz_encoder # encoder
```

## Fuzz coverage

Install component and tools (require rust nighlty)

```sh
$ rustup component add llvm-tools-preview
$ cargo install cargo-binutils
$ cargo install rustfilt
```

Run fuzz test

```sh
$ cargo fuzz run fuzz_alu
```

Generate coverage report

```sh
$ cargo fuzz coverage fuzz_alu
$ cargo cov -- show target/x86_64-unknown-linux-gnu/release/fuzz_alu --Xdemangler=rustfilt --format=html -instr-profile=coverage/fuzz_alu/coverage.profdata --name=ckb --line-coverage-gt=1> /tmp/report.html

$ cargo fuzz coverage fuzz_alu
$ cargo cov -- show target/x86_64-unknown-linux-gnu/release/fuzz_alu --Xdemangler=rustfilt --format=html -instr-profile=coverage/fuzz_alu/coverage.profdata --name=ckb --line-coverage-gt=1> /tmp/report.html
```
