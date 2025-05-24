#just manual: https://github.com/casey/just/#readme

_default:
    @just --list

watch: 
    watchexec -r -e rs -- cargo run -- --help

run *args='list':
    cargo build --release && USERNAME="lau" PROJECT_NAME="Gulfi" ./target/x86_64-unknown-linux-musl/release/backlogr {{args}}

udeps:
    RUSTC_BOOTSTRAP=1 cargo udeps --all-targets --backend depinfo

check:
    cargo clippy --locked -- -D warnings -D clippy::unwrap_used
