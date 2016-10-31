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

## Selection of test data

For testing decoding, it's important to have test data that's real-world Web
content in order to have a real-world interleaving of markup and non-markup.

Unicode.org's translations of the Universal Declaration of Human Rights have
less markup than one would expect from Web content in general. (Also, the
copyright status of the _translations_ wasn't obvious at a glance.)

Using Google Translate to synthetize content in various languages doesn't work,
because Google Translate adds its own markup, which messes up the natural
interleaving of ASCII and non-ASCII in real-world Web content.

Reasons for choosing Wikipedia were:

* Wikipedia is an actual top site that's relevant to users.
* Wikipedia has content in all the languages that were relevant for testing.
* Wikipedia content is human-authored.
* Wikipedia content is suitably licensed.

The topic Mars, the planet, was chosen, because it is the most-featured topic
across the different-language Wikipedias and, indeed, had non-trivial articles
in all the languages needed. Trying to choose a typical-length article for each
language separately wasn't feasible in the Wikidata data set.

For x-user-defined, a binary file is used instead of text, because the use case
for x-user-defined is loading binary data using XHR (in pre-`ArrayBuffer` code).

For testing encoders, the relevant cases are URL parsing (almost always ASCII),
form submission (typically mostly human-readable text) and POSTing stuff using
XHR (UTF-16 to UTF-8 encode only). Because it was too troublesome to find
real-world workloads representing POSTing stuff using XHR and because URL
parsing is almost always ASCII, the form submission case is measured even
though (except when encoding to UTF-8), encoding_rs explicitly doesn't attempt
to optimize that case for speed but for size. The test data is a plain-text
extract from each corresponding HTML decoder test file.
