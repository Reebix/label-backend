sudo --preserve-env=HOME,PATH,RUSTUP_HOME,CARGO_HOME,RUSTUP_TOOLCHAIN \
env RUSTUP_TOOLCHAIN=stable \
/home/rebix/.cargo/bin/cross build --release --target armv7-unknown-linux-gnueabihf