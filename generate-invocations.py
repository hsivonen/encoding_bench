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
  ("ar", "windows_1256", 1256),
  ("cs", "windows_1250", 1250),
  ("de", "windows_1252", 1252),
  ("el", "windows_1253", 1253),
  ("en", "windows_1252", 1252),
  ("fr", "windows_1252", 1252),
  ("he", "windows_1255", 1255),
  ("ja", "shift_jis", 932),
  ("ko", "euc_kr", 949),
  ("pt", "windows_1252", 1252),
  ("ru", "windows_1251", 1251),
  ("th", "windows_874", 874),
  ("tr", "windows_1254", 1254),
  ("vi", "windows_1258", 1258),
  ("zh_cn", "gb18030", 54936),
  ("zh_tw", "big5", 950),
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

for (lang, enc, cp) in languages:
  lib_file.write('''decode_bench!(bench_copy_{lang},
              bench_decode_to_utf8_{lang},
              bench_decode_to_utf16_{lang},
              bench_decode_to_string_{lang},
              bench_rust_to_string_{lang},
              bench_std_validation_{lang},
              bench_iconv_to_utf8_{lang},
              bench_icu_to_utf16_{lang},
              bench_uconv_to_utf16_{lang},
              bench_windows_to_utf16_{lang},
              bench_decode_to_utf8_{lang}_{enc},
              bench_decode_to_utf16_{lang}_{enc},
              bench_decode_to_string_{lang}_{enc},
              bench_rust_to_string_{lang}_{enc},
              bench_iconv_to_utf8_{lang}_{enc},
              bench_icu_to_utf16_{lang}_{enc},
              bench_uconv_to_utf16_{lang}_{enc},
              bench_windows_to_utf16_{lang}_{enc},
              {upper},
              {cp},
              "wikipedia/{lang}.html");
'''.format(lang=lang, enc=enc, upper=enc.upper(), cp=cp))

  lib_file.write('''encode_bench!(bench_encode_from_utf8_{lang},
              bench_encode_from_utf16_{lang},
              bench_encode_to_vec_{lang},
              bench_rust_to_vec_{lang},
              bench_iconv_from_utf8_{lang},
              bench_icu_from_utf16_{lang},
              bench_uconv_from_utf16_{lang},
              bench_windows_from_utf16_{lang},
              bench_encode_from_utf8_{lang}_{enc},
              bench_encode_from_utf16_{lang}_{enc},
              bench_encode_to_vec_{lang}_{enc},
              bench_rust_to_vec_{lang}_{enc},
              bench_iconv_from_utf8_{lang}_{enc},
              bench_icu_from_utf16_{lang}_{enc},
              bench_uconv_from_utf16_{lang}_{enc},
              bench_windows_from_utf16_{lang}_{enc},
              {upper},
              {cp},
              "wikipedia/{lang}.txt");
'''.format(lang=lang, enc=enc, upper=enc.upper(), cp=cp))

  for (encoding, code_page) in [("utf_16le", 1200), ("utf_16be", 1201),]:
    lib_file.write('''decode_bench_legacy!(bench_decode_to_utf8_{lang}_{enc},
              bench_decode_to_utf16_{lang}_{enc},
              bench_decode_to_string_{lang}_{enc},
              bench_rust_to_string_{lang}_{enc},
              bench_iconv_to_utf8_{lang}_{enc},
              bench_icu_to_utf16_{lang}_{enc},
              bench_uconv_to_utf16_{lang}_{enc},
              bench_windows_to_utf16_{lang}_{enc},
              {upper},
              {cp},
              "wikipedia/{lang}.html");
'''.format(lang=lang, enc=encoding, upper=encoding.upper(), cp=code_page))

lib_file.write('''
''')

for length in [1, 3, 15, 16, 30, 1000]:
  lib_file.write('''mem_bench_is_u8!(bench_mem_is_ascii_true_{length},
                 bench_safe_mem_is_ascii_true_{length},
                 is_ascii,
                 {length},
                 "jquery/jquery-3.1.1.min.js");
mem_bench_is_u8!(bench_mem_is_ascii_false_{length},
                 bench_safe_mem_is_ascii_false_{length},
                 is_ascii,
                 {length},
                 "wikipedia/de-edit.txt");

mem_bench_is_u8!(bench_mem_is_utf8_latin1_true_{length},
                 bench_safe_mem_is_utf8_latin1_true_{length},
                 is_utf8_latin1,
                 {length},
                 "wikipedia/de-edit.txt");
mem_bench_is_u8!(bench_mem_is_utf8_latin1_false_{length},
                 bench_safe_mem_is_utf8_latin1_false_{length},
                 is_utf8_latin1,
                 {length},
                 "wikipedia/ar.txt");

mem_bench_is_str!(bench_mem_is_str_latin1_true_{length},
                  bench_safe_mem_is_str_latin1_true_{length},
                  is_str_latin1,
                  {length},
                  "wikipedia/de-edit.txt");
mem_bench_is_str!(bench_mem_is_str_latin1_false_{length},
                  bench_safe_mem_is_str_latin1_false_{length},
                  is_str_latin1,
                  {length},
                  "wikipedia/ar.txt");

mem_bench_is_u16!(bench_mem_is_basic_latin_true_{length},
                  bench_safe_mem_is_basic_latin_true_{length},
                  is_basic_latin,
                  {length},
                  "jquery/jquery-3.1.1.min.js");
mem_bench_is_u16!(bench_mem_is_basic_latin_false_{length},
                  bench_safe_mem_is_basic_latin_false_{length},
                  is_basic_latin,
                  {length},
                  "wikipedia/de-edit.txt");

mem_bench_is_u16!(bench_mem_is_utf16_latin1_true_{length},
                  bench_safe_mem_is_utf16_latin1_true_{length},
                  is_utf16_latin1,
                  {length},
                  "wikipedia/de-edit.txt");
mem_bench_is_u16!(bench_mem_is_utf16_latin1_false_{length},
                  bench_safe_mem_is_utf16_latin1_false_{length},
                  is_utf16_latin1,
                  {length},
                  "wikipedia/ar.txt");

mem_bench_is_u16!(bench_mem_utf16_valid_up_to_ascii_{length},
                  bench_safe_mem_utf16_valid_up_to_ascii_{length},
                  utf16_valid_up_to,
                  {length},
                  "jquery/jquery-3.1.1.min.js");
mem_bench_is_u16!(bench_mem_utf16_valid_up_to_latin1_{length},
                  bench_safe_mem_utf16_valid_up_to_latin1_{length},
                  utf16_valid_up_to,
                  {length},
                  "wikipedia/de-edit.txt");
mem_bench_is_u16!(bench_mem_utf16_valid_up_to_bmp_{length},
                  bench_safe_mem_utf16_valid_up_to_bmp_{length},
                  utf16_valid_up_to,
                  {length},
                  "wikipedia/ar.txt");

mem_bench_mut_u16!(bench_mem_ensure_utf16_validity_ascii_{length},
                   bench_safe_mem_ensure_utf16_validity_ascii_{length},
                   ensure_utf16_validity,
                   {length},
                   "jquery/jquery-3.1.1.min.js");
mem_bench_mut_u16!(bench_mem_ensure_utf16_validity_latin1_{length},
                   bench_safe_mem_ensure_utf16_validity_latin1_{length},
                   ensure_utf16_validity,
                   {length},
                   "wikipedia/de-edit.txt");
mem_bench_mut_u16!(bench_mem_ensure_utf16_validity_bmp_{length},
                   bench_safe_mem_ensure_utf16_validity_bmp_{length},
                   ensure_utf16_validity,
                   {length},
                   "wikipedia/ar.txt");

mem_bench_u8_to_u16!(bench_mem_convert_utf8_to_utf16_ascii_{length},
                     bench_safe_mem_convert_utf8_to_utf16_ascii_{length},
                     convert_utf8_to_utf16,
                     {length},
                     "jquery/jquery-3.1.1.min.js");
mem_bench_u8_to_u16!(bench_mem_convert_utf8_to_utf16_bmp_{length},
                     bench_safe_mem_convert_utf8_to_utf16_bmp_{length},
                     convert_utf8_to_utf16,
                     {length},
                     "wikipedia/ar.txt");

mem_bench_u8_to_u16!(bench_mem_convert_latin1_to_utf16_{length},
                     bench_safe_mem_convert_latin1_to_utf16_{length},
                     convert_latin1_to_utf16,
                     {length},
                     "wikipedia/de-edit.txt");

mem_bench_u8_to_u16!(bench_mem_copy_ascii_to_basic_latin_{length},
                     bench_safe_mem_copy_ascii_to_basic_latin_{length},
                     copy_ascii_to_basic_latin,
                     {length},
                     "jquery/jquery-3.1.1.min.js");

mem_bench_str_to_u16!(bench_mem_convert_str_to_utf16_ascii_{length},
                      bench_safe_mem_convert_str_to_utf16_ascii_{length},
                      convert_str_to_utf16,
                      {length},
                      "jquery/jquery-3.1.1.min.js");
mem_bench_str_to_u16!(bench_mem_convert_str_to_utf16_bmp_{length},
                      bench_safe_mem_convert_str_to_utf16_bmp_{length},
                      convert_str_to_utf16,
                      {length},
                      "wikipedia/ar.txt");

mem_bench_u16_to_u8!(bench_mem_convert_utf16_to_utf8_ascii_{length},
                     bench_safe_mem_convert_utf16_to_utf8_ascii_{length},
                     convert_utf16_to_utf8,
                     {length},
                     "jquery/jquery-3.1.1.min.js");
mem_bench_u16_to_u8!(bench_mem_convert_utf16_to_utf8_bmp_{length},
                     bench_safe_mem_convert_utf16_to_utf8_bmp_{length},
                     convert_utf16_to_utf8,
                     {length},
                     "wikipedia/ar.txt");

mem_bench_u16_to_u8!(bench_mem_convert_utf16_to_latin1_lossy_{length},
                     bench_safe_mem_convert_utf16_to_latin1_lossy_{length},
                     convert_utf16_to_latin1_lossy,
                     {length},
                     "wikipedia/de-edit.txt");

mem_bench_u16_to_u8!(bench_mem_copy_basic_latin_to_ascii_{length},
                     bench_safe_mem_copy_basic_latin_to_ascii_{length},
                     copy_basic_latin_to_ascii,
                     {length},
                     "jquery/jquery-3.1.1.min.js");

mem_bench_u16_to_str!(bench_mem_convert_utf16_to_str_ascii_{length},
                      bench_safe_mem_convert_utf16_to_str_ascii_{length},
                      convert_utf16_to_str,
                      {length},
                      "jquery/jquery-3.1.1.min.js");
mem_bench_u16_to_str!(bench_mem_convert_utf16_to_str_bmp_{length},
                      bench_safe_mem_convert_utf16_to_str_bmp_{length},
                      convert_utf16_to_str,
                      {length},
                      "wikipedia/ar.txt");

mem_bench_u8_to_u8!(bench_mem_convert_latin1_to_utf8_{length},
                    bench_safe_mem_convert_latin1_to_utf8_{length},
                    convert_latin1_to_utf8,
                    {length},
                    "wikipedia/de-edit.txt");

mem_bench_u8_to_u8!(bench_mem_convert_utf8_to_latin1_lossy_{length},
                    bench_safe_mem_convert_utf8_to_latin1_lossy_{length},
                    convert_utf8_to_latin1_lossy,
                    {length},
                    "wikipedia/de-edit.txt");

mem_bench_u8_to_u8!(bench_mem_copy_ascii_to_ascii_{length},
                    bench_safe_mem_copy_ascii_to_ascii_{length},
                    copy_ascii_to_ascii,
                    {length},
                    "jquery/jquery-3.1.1.min.js");

mem_bench_u8_to_str!(bench_mem_convert_latin1_to_str_{length},
                     bench_safe_mem_convert_latin1_to_str_{length},
                     convert_latin1_to_str,
                     {length},
                     "wikipedia/de-edit.txt");
'''.format(length=length))


lib_file.write(lib_rs_end)
lib_file.close()

subprocess.call(["cargo", "fmt"])
