set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd ${DIR}

bash build.sh

make
cargo build

for example in examples/*.rs; do
    cargo run --example `basename $example .rs`
done
