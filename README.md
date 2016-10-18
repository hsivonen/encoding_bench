# encoding_bench

This is a performance testing framework for understanding the performance
characteristics of [encoding_rs](https://github.com/hsivonen/encoding_rs).

This framework is separate from encoding_rs itself in order to separate
nightly-only Rust features (`#[bench]`) and CC-licensed test data from the main
repository.

## Licensing

Please see the file named COPYRIGHT.

## Building

Currently, this project is tested to build only on Ubuntu 16.04 and with a
custom build of Gecko. Before building this project, make a Firefox optimized
build with the patch `gecko.patch` (from this directory) applied and Rust
enabled (`ac_add_options --enable-rust` in your `mozconfig`). The patch
breaks the ability to run the resulting Firefox build normally and breaks
the packaging that would happen on Mozilla's try server. For the latter reason,
you need to build locally.

Once you have the custom Gecko build available, build this project with
```
LIBRARY_PATH=/path-to-gecko-obj-dir/dist/sdk/lib:/path-to-gecko-obj-dir/dist/bin LD_LIBRARY_PATH=/path-to-gecko-obj-dir/dist/bin cargo bench
```

If the build is successful, it's a good idea to append `2> /dev/null` for the
actual benchmarking runs to hide noise from Gecko.

