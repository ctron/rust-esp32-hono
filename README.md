# Rust ESP32 project template

This is a project template for the use of [ctron/rust-esp-container](https://github.com/ctron/rust-esp-container).

Build with:

    docker run -v $PWD:/home/project:z --rm -ti quay.io/ctron/rust-esp xbuild-project

On Windows:

    docker run -v %CD%:/home/project --rm -ti quay.io/ctron/rust-esp xbuild-project

Flash with:

    esptool.py --chip esp32 --baud 115200 --before default_reset --after hard_reset write_flash -z --flash_mode dio --flash_freq 40m --flash_size detect 0x1000 build/bootloader/bootloader.bin 0x10000 esp-app.bin 0x8000 build/partitions_singleapp.bin
