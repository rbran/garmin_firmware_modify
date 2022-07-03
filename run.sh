#!/bin/bash

set -e

SCRIPT=$(readlink -f "$0")
BASEDIR=$(dirname "$SCRIPT")

CC="arm-none-eabi-ld"
TARGET="thumbv7em-none-eabihf"
INPUT_FIRMWARE="${BASEDIR}/../../firmware/edge130apac/380/firmware.gcd"
OUTPUT_FIRMWARE="GUPDATE.GCD"

INJECT_MAX_LEN="0x270"
INJECT_ADDR="0xbf9a0"
cat > ${BASEDIR}/firmware_payload/.cargo/firmware.ld <<EOF
MEMORY
{
  FLASH : ORIGIN = ${INJECT_ADDR}, LENGTH = ${INJECT_MAX_LEN}
}

ENTRY(_start);
SECTIONS
{
  .text :
  {
    *(.text .text.*);
  } > FLASH

  .rodata :
  {
    *(.rodata .rodata.*);
  } > FLASH

  /DISCARD/ :
  {
    *(.ARM.exidx .ARM.exidx.*);
  }
}
EOF

cat > ${BASEDIR}/firmware_payload/.cargo/config.toml <<EOF
[target.${TARGET}]
linker = "arm-none-eabi-ld"
rustflags = [
    "-C", "link-arg=-T${BASEDIR}/firmware_payload/.cargo/firmware.ld",
]

[profile.release]
opt-level = 'z'
lto = true

[stable]
build-std = ["core"]

[build]
target = "${TARGET}"
EOF

cd ${BASEDIR}/firmware_payload
cargo build --release

cd ${BASEDIR}

FW_INJECTS=()
for payload in $@
do
    if [ "xmem_dump" == "x$payload" ]
    then
        INJECT_OFFSET="0x2000"
        INJECT_ADDR="0xbf9a0"
        INJECT_FW_ADDR=$(printf '%#x' "$((${INJECT_ADDR} - ${INJECT_OFFSET}))")
        #INJECT_MAX_LEN="0x270"
        INJECT_FW_ID="0x2bd"
        PAYLOAD_FILE="${BASEDIR}/firmware_payload/target/thumbv7em-none-eabihf/release/mem_dump"
        arm-none-eabi-objcopy -O binary \
            -j .text \
            -j .rodata \
            "${PAYLOAD_FILE}" \
            "${PAYLOAD_FILE}.bin"
        FW_INJECTS+=("${PAYLOAD_FILE}.bin")
        FW_INJECTS+=(${INJECT_FW_ADDR})
        FW_INJECTS+=(${INJECT_FW_ID})
    fi
    if [ "xport_search" == "x$payload" ]
    then
        INJECT_OFFSET="0x2000"
        INJECT_ADDR="0xbf9a0"
        INJECT_FW_ADDR=$(printf '%#x' "$((${INJECT_ADDR} - ${INJECT_OFFSET}))")
        #INJECT_MAX_LEN="0x270"
        INJECT_FW_ID="0x2bd"
        PAYLOAD_FILE="${BASEDIR}/firmware_payload/target/thumbv7em-none-eabihf/release/port_search"
        arm-none-eabi-objcopy -O binary \
            -j .text \
            -j .rodata \
            "${PAYLOAD_FILE}" \
            "${PAYLOAD_FILE}.bin"
        FW_INJECTS+=("${PAYLOAD_FILE}.bin")
        FW_INJECTS+=(${INJECT_FW_ADDR})
        FW_INJECTS+=(${INJECT_FW_ID})
    fi
    if [ "xnrf52_jtag_input" == "x$payload" ]
    then
        # don't enable port_a_21 port_a_22 pull resistor
        INJECT_OFFSET="0x2000"
        INJECT_ADDR="0x6786a"
        INJECT_FW_ADDR=$(printf '%#x' "$((${INJECT_ADDR} - ${INJECT_OFFSET}))")
        INJECT_FW_ID="0x2bd"
        PAYLOAD_FILE="${BASEDIR}/bin/nrf52_jtag_input_p1.payload"
        FW_INJECTS+=(${PAYLOAD_FILE})
        FW_INJECTS+=(${INJECT_FW_ADDR})
        FW_INJECTS+=(${INJECT_FW_ID})

        # don't put port_a_20 into output mode
        INJECT_ADDR="0x679f2"
        INJECT_FW_ADDR=$(printf '%#x' "$((${INJECT_ADDR} - ${INJECT_OFFSET}))")
        PAYLOAD_FILE="${BASEDIR}/bin/nrf52_jtag_input_p2.payload"
        FW_INJECTS+=(${PAYLOAD_FILE})
        FW_INJECTS+=(${INJECT_FW_ADDR})
        FW_INJECTS+=(${INJECT_FW_ID})
    fi
done

# compile and exec the firmware inject
cd ${BASEDIR}/firmware_inject
cargo run -- \
    ${INPUT_FIRMWARE} \
    ${OUTPUT_FIRMWARE} \
    ${FW_INJECTS[@]}

# unpack the firmware, in case a review is needed
#cd ${BASEDIR}/out
#/home/rbran/src/gcd-rs/target/release/examples/gcd_extract ../GUPDATE.GCD firmware.toml

