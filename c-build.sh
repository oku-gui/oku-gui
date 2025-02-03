set -e

cargo build --package oku_c
cbindgen --crate oku_c --output examples/c/oku_c.h --lang c
gcc -o target/debug/oku_c examples/c/main.c -L "target\debug" -l oku_c -lpthread && ./target/debug/oku_c.exe