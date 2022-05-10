## Fuzz

```sh
$ cargo fuzz run fuzz_target_1 # arithmetic logic instructions
$ cargo fuzz run fuzz_target_2 # memory store/load instructions
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
$ cargo fuzz run fuzz_target_1
```

Generate coverage report

```sh
$ cargo fuzz coverage fuzz_target_1
$ cargo cov -- show target/x86_64-unknown-linux-gnu/release/fuzz_target_1 --Xdemangler=rustfilt --format=html -instr-profile=coverage/fuzz_target_1/coverage.profdata --name=ckb --line-coverage-gt=1> /tmp/report.html

$ cargo fuzz coverage fuzz_target_2
$ cargo cov -- show target/x86_64-unknown-linux-gnu/release/fuzz_target_2 --Xdemangler=rustfilt --format=html -instr-profile=coverage/fuzz_target_2/coverage.profdata --name=ckb --line-coverage-gt=1> /tmp/report.html
```
