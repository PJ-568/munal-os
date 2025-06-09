#!/bin/bash

set -e

#
# Building WASM apps

cd wasm_apps/

cd cube_3d/
cargo build --release
cd ../

cd chronometer/
cargo build --release
cd ../

cd terminal/
cargo build --release
cd ../

cd web_browser/
cargo build --release
cd ../

cd text_editor/
cargo build --release
cd ../

cd ../


#
# Embedding binary data

mkdir -p embedded_data/
cp wasm_apps/cube_3d/target/wasm32-wasip1/release/cube_3d.wasm embedded_data/cube_3d.wasm
cp wasm_apps/chronometer/target/wasm32-wasip1/release/chronometer.wasm embedded_data/chronometer.wasm
cp wasm_apps/terminal/target/wasm32-wasip1/release/terminal.wasm embedded_data/terminal.wasm
cp wasm_apps/web_browser/target/wasm32-wasip1/release/web_browser.wasm embedded_data/web_browser.wasm
cp wasm_apps/text_editor/target/wasm32-wasip1/release/text_editor.wasm embedded_data/text_editor.wasm


#
# Building kernel

cd kernel/
cargo build --release
cd ../


#
# Running QEMU

mkdir -p esp/efi/boot/
cp kernel/target/x86_64-unknown-uefi/release/kernel.efi esp/efi/boot/bootx64.efi

qemu-system-x86_64 \
    -enable-kvm \
    -m 1G \
    -rtc base=utc \
    -display sdl \
    -drive if=pflash,format=raw,readonly=on,file=uefi_firmware/code.fd \
    -drive if=pflash,format=raw,readonly=on,file=uefi_firmware/vars.fd \
    -drive format=raw,file=fat:rw:esp \
    -device virtio-keyboard \
    -device virtio-mouse \
    -device virtio-net-pci,netdev=network0 -netdev user,id=network0 \
    -vga virtio \
    -serial stdio
