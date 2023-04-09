run-dev:
    trunk serve
check:
    cargo fmt --check --all
    cargo clippy --all
test:
    cargo test

commit:
    cargo fmt --check
    cargo clippy
    trunk build --release
    git commit