#!/usr/bin/python

# Copyright 2016 Mozilla Foundation. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import subprocess

languages = [
  ("ar", "windows_1256"),
  ("el", "windows_1253"),
  ("en", "windows_1252"),
  ("he", "windows_1255"),
  ("ja", "shift_jis"),
  ("ko", "euc_kr"),
  ("ru", "windows_1251"),
  ("zh_cn", "gb18030"),
#  ("zh_tw", "big5"),
]

def read_non_generated(path):
  partially_generated_file = open(path, "r")
  full = partially_generated_file.read()
  partially_generated_file.close()

  generated_begin = "// BEGIN GENERATED CODE. PLEASE DO NOT EDIT."
  generated_end = "// END GENERATED CODE"

  generated_begin_index = full.find(generated_begin)
  if generated_begin_index < 0:
    print "Can't find generated code start marker in %s. Exiting." % path
    sys.exit(-1)
  generated_end_index = full.find(generated_end)
  if generated_end_index < 0:
    print "Can't find generated code end marker in %s. Exiting." % path
    sys.exit(-1)

  return (full[0:generated_begin_index + len(generated_begin)],
          full[generated_end_index:])

(lib_rs_begin, lib_rs_end) = read_non_generated("src/lib.rs")

lib_file = open("src/lib.rs", "w")

lib_file.write(lib_rs_begin)
lib_file.write("""
// Instead, please regenerate using generate-encoding-data.py

""")

for (lang, enc) in languages:
  lib_file.write('''decode_bench!(bench_decode_to_utf8_%s,
              bench_decode_to_utf16_%s,
              bench_decode_to_string_%s,
              bench_rust_to_string_%s,
              bench_std_to_string_%s,
              bench_iconv_to_utf8_%s,
              bench_icu_to_utf16_%s,
              bench_uconv_to_utf16_%s,
              bench_decode_to_utf8_%s,
              bench_decode_to_utf16_%s,
              bench_decode_to_string_%s,
              bench_rust_to_string_%s,
              bench_iconv_to_utf8_%s,
              bench_icu_to_utf16_%s,
              bench_uconv_to_utf16_%s,
              %s,
              "wikipedia/%s.html");
''' % (lang, lang, lang, lang, lang, lang, lang, lang, enc, enc, enc, enc, enc, enc, enc, enc.upper(), lang))

lib_file.write('''
''')
lib_file.write(lib_rs_end)
lib_file.close()

subprocess.call(["cargo", "fmt"])
