Files in this directory have this naming pattern:

revision_computer_system_bits.txt

## Revisions

* baseline: encoding_rs tag [bench_baseline][1]
* alu: encoding_rs tag [bench_before_utf8_polish][2]
* simd: encoding_rs tag [bench_before_utf8_polish][2] with `--features simd-accel`
* unaligned: encoding_rs tag [bench_only_unaligned_sse2][3] with `--features simd-accel`

[1]: https://github.com/hsivonen/encoding_rs/releases/tag/bench_baseline
[2]: https://github.com/hsivonen/encoding_rs/releases/tag/bench_before_utf8_polish
[3]: https://github.com/hsivonen/encoding_rs/releases/tag/bench_only_unaligned_sse2

## Computers

* dell: Intel Core i7-4770 @ 3.40 GHz
* carbon: Intel Core i7-3667U @ 2.00 GHz (max 2.50 GHz)
* intense: Intel Core i3-4010U @ 1.70GHz
* shuttle: Intel Core i7-950 @ 3.07GHz
* macbook: Intel Core2 Duo T8300 @ 2.40 GHz
* rpi3: ARM Cortex-A53 @ 1.2 GHz

## Systems

* windows: Windows 10 1607, MSVC ABI for Rust
* linux: Ubuntu 16.04.1, Linux 4.4.x

## Bits

* 64: 64-bit operating system running 64-bit benchmark process
* 32: 32-bit operating system running 32-bit benchmark process
