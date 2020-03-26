command -v grcov >/dev/null 2>&1 || { echo >&2 "Please install 'grcov' first.  Exit."; exit 1; }

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads"

cargo clean
cargo build
sh run_tests.sh
grcov ./target/debug/ -s . -t html --llvm --branch --ignore-not-existing -o ./target/debug/coverage/

echo "The report is in: target/debug/coverage/index.html"