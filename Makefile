.PHONY: all clippy debug release flash gnumake-env

BUILD_TIMESTAMP := $(shell date +%s)
.EXPORT_ALL_VARIABLES:

all: clippy debug release

clippy:
	cargo clippy

debug:
	cargo build

release:
	cargo build --release

flash:
	# 16MB is important. So might partitions.csv be!
	# Also, we need manual esp_println or we see nothing.
	espflash flash --chip=esp32 --port=/dev/ttyACM0 -M --partition-table=partitions.csv --flash-size=16mb --target-app-partition=ota_0 ./target/xtensa-esp32-none-elf/debug/hellomch
