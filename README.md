st-flash write rom.bin 0x8000000
cargo objcopy -- -O binary target/thumbv7em-none-eabi/release/stm32-ws2812 rom.bin
cargo install cargo-binutils
