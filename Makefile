.PHONY: all clippy debug release flash gnumake-env

include config.mk

BUILD_TIMESTAMP := $(shell date +%s)
.EXPORT_ALL_VARIABLES:

all: clippy debug #release

clippy:
	cargo clippy $(BUILD_FEATURES)

debug:
	cargo build $(BUILD_FEATURES)

release:
	cargo build --release $(BUILD_FEATURES)

flash:
	# 16MB is important. So might partitions.csv be!
	# Select latest build by time using find+ls.
	LATEST=$$(ls -t $$(find target/ -name hellomch) | head -n1) && \
	  printf '\nLatest image is: %s\n\n' "$$LATEST" >&2 && \
	  espflash flash --chip=esp32 --port=/dev/ttyACM0 -M \
	    --partition-table=partitions.csv --flash-size=16mb \
	    --target-app-partition=ota_0 "$$LATEST"
