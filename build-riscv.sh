#!/bin/bash

cargo +nightly build -Zbuild-std=core,alloc,std,panic_abort -Zbuild-std-features="optimize_for_size" --target riscv64gc-unknown-linux-musl $@