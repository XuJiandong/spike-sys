set -e

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd ${DIR}

bash build.sh

make
cargo build
cargo run --example=add
cargo run --example=mem

