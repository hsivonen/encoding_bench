#![feature(test)]

extern crate test;
extern crate encoding_rs;
extern crate encoding;
extern crate libc;

use test::Bencher;

#[cfg(target_os = "linux")]
use std::ffi::CString;

macro_rules! decode_bench_impl {
    ($name:ident,
     $encoding:ident,
     $data:expr,
     $max:ident,
     $decode:ident) => (
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
        let (input, _, _) = encoding.encode(utf8);
        let mut decoder = encoding.new_decoder_without_bom_handling();
        let out_len = decoder.$max(input.len());
        let mut output = Vec::with_capacity(out_len);
        output.resize(out_len, 0);
        b.bytes = input.len() as u64;
        b.iter(|| {
            let (result, _, _, _) = decoder.$decode(test::black_box(&input[..]), &mut output[..], false);
            match result {
                encoding_rs::CoderResult::InputEmpty => {}
                encoding_rs::CoderResult::OutputFull => {
                    unreachable!("Output buffer too short.");
                }
            }
            test::black_box(&output);
        });
    });
}

macro_rules! decode_bench_utf8 {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    decode_bench_impl!($name, $encoding, $data, max_utf8_buffer_length, decode_to_utf8);
     );
}

macro_rules! decode_bench_utf16 {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    decode_bench_impl!($name, $encoding, $data, max_utf16_buffer_length, decode_to_utf16);
     );
}

macro_rules! encode_bench_utf8 {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
// Convert back and forth to avoid benching replacement, which other
// libs won't do.
        let (intermediate, _, _) = encoding.encode(utf8);
        let (cow, _) = encoding.decode_without_bom_handling(&intermediate[..]);
        let input = &cow[..];
        let mut encoder = encoding.new_encoder();
        let out_len = intermediate.len() + 10;
        let mut output = Vec::with_capacity(out_len);
        output.resize(out_len, 0);
// Use output length to have something that can be compared
        b.bytes = intermediate.len() as u64;
        b.iter(|| {
            let (result, _, _, _) = encoder.encode_from_utf8(test::black_box(&input[..]), &mut output[..], false);
            match result {
                encoding_rs::CoderResult::InputEmpty => {}
                encoding_rs::CoderResult::OutputFull => {
                    unreachable!("Output buffer too short.");
                }
            }
            test::black_box(&output);
        });
    });
}

macro_rules! encode_bench_utf16 {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
// Convert back and forth to avoid benching replacement, which other
// libs won't do.
        let (intermediate, _, _) = encoding.encode(utf8);
        let mut decoder = encoding.new_decoder_without_bom_handling();
        let mut input: Vec<u16> = Vec::with_capacity(decoder.max_utf16_buffer_length(intermediate.len()));
        let capacity = input.capacity();
        input.resize(capacity, 0u16);
        let (complete, _, written, _) = decoder.decode_to_utf16(&intermediate[..], &mut input[..], true);
        match complete {
            encoding_rs::CoderResult::InputEmpty => {}
            encoding_rs::CoderResult::OutputFull => {
                unreachable!();
            }
        }
        input.truncate(written);
        let mut encoder = encoding.new_encoder();
        let out_len = intermediate.len() + 10;
        let mut output = Vec::with_capacity(out_len);
        output.resize(out_len, 0);
// Use output length to have something that can be compared
        b.bytes = intermediate.len() as u64;
        b.iter(|| {
            let (result, _, _, _) = encoder.encode_from_utf16(test::black_box(&input[..]), &mut output[..], false);
            match result {
                encoding_rs::CoderResult::InputEmpty => {}
                encoding_rs::CoderResult::OutputFull => {
                    unreachable!("Output buffer too short.");
                }
            }
            test::black_box(&output);
        });
    });
}

macro_rules! decode_bench_string {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
        let (input, _, _) = encoding.encode(utf8);
        b.bytes = input.len() as u64;
        b.iter(|| {
            let (output, _) = encoding.decode_without_bom_handling(test::black_box(&input[..]));
            test::black_box(&output);
        });
    });
}

macro_rules! encode_bench_vec {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
        // Convert back and forth to avoid benching replacement, which other
        // libs won't do.
        let (intermediate, _, _) = encoding.encode(utf8);
        let (cow, _) = encoding.decode_without_bom_handling(&intermediate[..]);
        let input = &cow[..];
        // Use output length to have something that can be compared
        b.bytes = intermediate.len() as u64;
        b.iter(|| {
            let (output, _, _) = encoding.encode(test::black_box(&input[..]));
            test::black_box(&output);
        });
    });
}

// rust-encoding

macro_rules! decode_bench_rust {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
        let (input, _, _) = encoding.encode(utf8);
        let rust_encoding = encoding::label::encoding_from_whatwg_label(encoding.name()).unwrap();
        b.bytes = input.len() as u64;
        b.iter(|| {
            let output = rust_encoding.decode(test::black_box(&input[..]), encoding::DecoderTrap::Replace);
            test::black_box(&output);
        });
    });
}

macro_rules! encode_bench_rust {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
// Convert back and forth to avoid benching replacement, which other
// libs won't do.
        let (intermediate, _, _) = encoding.encode(utf8);
        let (cow, _) = encoding.decode_without_bom_handling(&intermediate[..]);
        let input = &cow[..];
        let rust_encoding = encoding::label::encoding_from_whatwg_label(encoding.name()).unwrap();
// Use output length to have something that can be compared
        b.bytes = intermediate.len() as u64;
        b.iter(|| {
            let output = rust_encoding.encode(test::black_box(&input[..]), encoding::EncoderTrap::Replace);
            test::black_box(&output);
        });
    });
}

// standard library

macro_rules! decode_bench_std {
    ($name:ident,
     $data:expr) => (
    #[bench]
    fn $name(b: &mut Bencher) {
        let mut input = Vec::new();
        input.extend_from_slice(include_bytes!($data));
        b.bytes = input.len() as u64;
        b.iter(|| {
            let output = String::from_utf8_lossy(&input[..]).to_owned();
            test::black_box(&output);
        });
    });
}

macro_rules! copy_bench {
    ($name:ident,
     $data:expr) => (
    #[bench]
    fn $name(b: &mut Bencher) {
        let mut input = Vec::new();
        input.extend_from_slice(include_bytes!($data));
        let mut output = Vec::with_capacity(input.len());
        b.bytes = input.len() as u64;
        b.iter(|| {
            unsafe {
                std::ptr::copy_nonoverlapping(test::black_box(input.as_slice().as_ptr()), output.as_mut_slice().as_mut_ptr(), input.len());
            }
            test::black_box(&output);
        });
    });
}

// iconv

#[cfg(target_os = "linux")]
extern "C" {
    fn iconv_open(tocode: *const std::os::raw::c_char,
                  fromcode: *const std::os::raw::c_char)
                  -> *mut libc::c_void;
    fn iconv_close(cd: *mut libc::c_void) -> libc::c_int;
    fn iconv(cd: *mut libc::c_void,
             inbuf: *mut *mut u8,
             inbytesleft: *mut usize,
             outbuf: *mut *mut u8,
             outbytesleft: *mut usize)
             -> usize;
}

#[cfg(target_os = "linux")]
fn iconv_name(encoding: &'static encoding_rs::Encoding) -> &'static str {
    if encoding_rs::BIG5 == encoding {
        "big5-hkscs"
    } else if encoding_rs::SHIFT_JIS == encoding {
        "windows-31j"
    } else if encoding_rs::EUC_KR == encoding {
        "cp949"
    } else {
        encoding.name()
    }
}

macro_rules! decode_bench_iconv {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[cfg(target_os = "linux")]
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
        let (input, _, _) = encoding.encode(utf8);
        let decoder = encoding.new_decoder_without_bom_handling();
        let out_len = decoder.max_utf8_buffer_length(input.len());
        let mut output: Vec<u8> = Vec::with_capacity(out_len);
        output.resize(out_len, 0);
        let from_label = CString::new(iconv_name(encoding)).unwrap();
        let to_label = CString::new("UTF-8").unwrap();
        let cd = unsafe { iconv_open(to_label.as_ptr(), from_label.as_ptr()) };
        b.bytes = input.len() as u64;
        b.iter(|| {
              unsafe {
                   // Black boxing input doesn't work, but iconv isn't in the
                   // view of the optimizer anyway.
                   let mut input_ptr = input.as_ptr() as *mut u8;
                   let mut output_ptr = output.as_mut_ptr();
                   let input_ptr_ptr = &mut input_ptr as *mut *mut u8;
                   let output_ptr_ptr = &mut output_ptr as *mut *mut u8;
                   let mut input_left = input.len();
                   let mut output_left = output.len();
                   let input_left_ptr = &mut input_left as *mut usize;
                   let output_left_ptr = &mut output_left as *mut usize;
                   iconv(cd, input_ptr_ptr, input_left_ptr, output_ptr_ptr, output_left_ptr);
                   assert_eq!(input_left, 0usize);
                test::black_box(&output);
              }
        });
          unsafe {
              iconv_close(cd);
          }
    });
}

macro_rules! encode_bench_iconv {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[cfg(target_os = "linux")]
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
        // Convert back and forth to avoid benching replacement, which other
        // libs won't do.
        let (intermediate, _, _) = encoding.encode(utf8);
        let (cow, _) = encoding.decode_without_bom_handling(&intermediate[..]);
        let input = &cow[..];
        let out_len = intermediate.len() + 10;
        let mut output: Vec<u8> = Vec::with_capacity(out_len);
        output.resize(out_len, 0);
        let from_label = CString::new("UTF-8").unwrap();
        let to_label = CString::new(iconv_name(encoding)).unwrap();
        let cd = unsafe { iconv_open(to_label.as_ptr(), from_label.as_ptr()) };
        // Use output length to have something that can be compared
        b.bytes = intermediate.len() as u64;
        b.iter(|| {
              unsafe {
                   // Black boxing input doesn't work, but iconv isn't in the
                   // view of the optimizer anyway.
                   let mut input_ptr = input.as_ptr() as *mut u8;
                   let mut output_ptr = output.as_mut_ptr();
                   let input_ptr_ptr = &mut input_ptr as *mut *mut u8;
                   let output_ptr_ptr = &mut output_ptr as *mut *mut u8;
                   let mut input_left = input.len();
                   let mut output_left = output.len();
                   let input_left_ptr = &mut input_left as *mut usize;
                   let output_left_ptr = &mut output_left as *mut usize;
                   iconv(cd, input_ptr_ptr, input_left_ptr, output_ptr_ptr, output_left_ptr);
                   assert_eq!(input_left, 0usize);
                test::black_box(&output);
              }
        });
          unsafe {
              iconv_close(cd);
          }
    });
}

// ICU

#[cfg(target_os = "linux")]
#[link(name = "icuuc")]
extern "C" {
    fn ucnv_open_55(label: *const std::os::raw::c_char,
                    error: *mut libc::c_int)
                    -> *mut libc::c_void;
    fn ucnv_close_55(cnv: *mut libc::c_void);
    fn ucnv_toUChars_55(cnv: *mut libc::c_void,
                        dst: *mut u16,
                        dst_len: i32,
                        src: *const u8,
                        src_len: i32,
                        error: *mut libc::c_int)
                        -> i32;
    fn ucnv_fromUChars_55(cnv: *mut libc::c_void,
                          dst: *mut u8,
                          dst_len: i32,
                          src: *const u16,
                          src_len: i32,
                          error: *mut libc::c_int)
                          -> i32;
}

#[cfg(target_os = "linux")]
fn icu_name(encoding: &'static encoding_rs::Encoding) -> &'static str {
    if encoding_rs::BIG5 == encoding {
        "big5-hkscs"
    } else if encoding_rs::SHIFT_JIS == encoding {
        "windows-31j"
    } else if encoding_rs::EUC_KR == encoding {
        "windows-949"
    } else {
        encoding.name()
    }
}

macro_rules! decode_bench_icu {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[cfg(target_os = "linux")]
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
        let (input, _, _) = encoding.encode(utf8);
        let decoder = encoding.new_decoder_without_bom_handling();
        let out_len = decoder.max_utf16_buffer_length(input.len());
        let mut output: Vec<u16> = Vec::with_capacity(out_len);
        output.resize(out_len, 0);
        let label = CString::new(icu_name(encoding)).unwrap();
        let mut error: libc::c_int = 0;
        let cnv = unsafe { ucnv_open_55(label.as_ptr(), &mut error) };
        b.bytes = input.len() as u64;
        b.iter(|| {
              unsafe {
                  ucnv_toUChars_55(cnv, output.as_mut_ptr(), output.len() as i32, input.as_ptr(), input.len() as i32, &mut error);
              }
            test::black_box(&output);
        });
        unsafe {
            ucnv_close_55(cnv);
        }
    });
}

macro_rules! encode_bench_icu {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[cfg(target_os = "linux")]
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
// Convert back and forth to avoid benching replacement, which other
// libs won't do.
        let (intermediate, _, _) = encoding.encode(utf8);
        let mut decoder = encoding.new_decoder_without_bom_handling();
        let mut input: Vec<u16> = Vec::with_capacity(decoder.max_utf16_buffer_length(intermediate.len()));
        let capacity = input.capacity();
        input.resize(capacity, 0u16);
        let (complete, _, written, _) = decoder.decode_to_utf16(&intermediate[..], &mut input[..], true);
        match complete {
            encoding_rs::CoderResult::InputEmpty => {}
            encoding_rs::CoderResult::OutputFull => {
                unreachable!();
            }
        }
        input.truncate(written);
        let mut encoder = encoding.new_encoder();
        let out_len = intermediate.len() + 10;
        let mut output: Vec<u8> = Vec::with_capacity(out_len);
        output.resize(out_len, 0);
        let label = CString::new(icu_name(encoding)).unwrap();
        let mut error: libc::c_int = 0;
        let cnv = unsafe { ucnv_open_55(label.as_ptr(), &mut error) };
// Use output length to have something that can be compared
        b.bytes = intermediate.len() as u64;
        b.iter(|| {
              unsafe {
                  ucnv_fromUChars_55(cnv, output.as_mut_ptr(), output.len() as i32, input.as_ptr(), input.len() as i32, &mut error);
              }
            test::black_box(&output);
        });
        unsafe {
            ucnv_close_55(cnv);
        }
    });
}

// uconv

#[cfg(target_os = "linux")]
#[link(name = "stdc++" )]
extern "C" {}

#[cfg(target_os = "linux")]
#[link(name = "mozglue", kind = "static" )]
extern "C" {}

#[cfg(target_os = "linux")]
#[link(name = "xul")]
extern "C" {
    fn NS_InitXPCOM2(manager: *mut *mut libc::c_void,
                     bin_dir: *mut libc::c_void,
                     provider: *mut libc::c_void)
                     -> libc::c_int;
    fn NS_CreateUnicodeDecoder(name: *const u8, name_len: usize) -> *mut libc::c_void;
    fn NS_ReleaseUnicodeDecoder(dec: *mut libc::c_void);
    fn NS_DecodeWithUnicodeDecoder(dec: *mut libc::c_void,
                                   src: *const u8,
                                   src_len: i32,
                                   dst: *mut u16,
                                   dst_len: i32);
}

#[cfg(target_os = "linux")]
static mut XPCOM_INITIALIZED: bool = false;

#[cfg(target_os = "linux")]
fn init_xpcom() {
    unsafe {
        if !XPCOM_INITIALIZED {
            XPCOM_INITIALIZED = true;
            NS_InitXPCOM2(std::ptr::null_mut(),
                          std::ptr::null_mut(),
                          std::ptr::null_mut());
        }
    }
}

macro_rules! decode_bench_uconv {
    ($name:ident,
     $encoding:ident,
     $data:expr) => (
    #[cfg(target_os = "linux")]
    #[bench]
    fn $name(b: &mut Bencher) {
        init_xpcom();
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
        let (input, _, _) = encoding.encode(utf8);
        let decoder = encoding.new_decoder_without_bom_handling();
        let out_len = decoder.max_utf16_buffer_length(input.len());
        let mut output: Vec<u16> = Vec::with_capacity(out_len);
        output.resize(out_len, 0);
        let name = encoding.name();
        let dec = unsafe { NS_CreateUnicodeDecoder(name.as_ptr(), name.len()) };
        b.bytes = input.len() as u64;
        b.iter(|| {
               unsafe {
                   NS_DecodeWithUnicodeDecoder(dec, input.as_ptr(), input.len() as i32, output.as_mut_ptr(), output.len() as i32);
               }
            test::black_box(&output);
        });
        unsafe {
            NS_ReleaseUnicodeDecoder(dec);
        }
    }
);
}

// Windows built-in

#[cfg(target_os = "windows")]
#[link(name = "Kernel32")]
extern "system" {
    fn MultiByteToWideChar(code_page: libc::c_uint,
                           flags: libc::c_ulong,
                           src: *const u8,
                           src_len: libc::c_int,
                           dst: *mut u16,
                           dst_len: libc::c_int)
                           -> libc::c_int;
}

macro_rules! decode_bench_windows {
    ($name:ident,
     $encoding:ident,
     $cp:expr,
     $data:expr) => (
    #[cfg(target_os = "windows")]
    #[bench]
    fn $name(b: &mut Bencher) {
        let encoding = encoding_rs::$encoding;
        let utf8 = include_str!($data);
        let (input, _, _) = encoding.encode(utf8);
        let decoder = encoding.new_decoder_without_bom_handling();
        let out_len = decoder.max_utf16_buffer_length(input.len());
        let mut output: Vec<u16> = Vec::with_capacity(out_len);
        output.resize(out_len, 0);
        b.bytes = input.len() as u64;
        b.iter(|| {
               unsafe {
                   MultiByteToWideChar($cp, 0, input.as_ptr(), input.len() as libc::c_int, output.as_mut_ptr(), output.len() as libc::c_int);
               }
            test::black_box(&output);
        });
    }
);
}

macro_rules! decode_bench {
    ($copy_name:ident,
     $name8:ident,
     $name16:ident,
     $string_name:ident,
     $rust_name:ident,
     $std_name:ident,
     $iconv_name:ident,
     $icu_name:ident,
     $uconv_name:ident,
     $windows_name:ident,
     $legacy_name8:ident,
     $legacy_name16:ident,
     $legacy_string_name:ident,
     $legacy_rust_name:ident,
     $legacy_iconv_name:ident,
     $legacy_icu_name:ident,
     $legacy_uconv_name:ident,
     $legacy_windows_name:ident,
     $encoding:ident,
     $cp:expr,
     $data:expr) => (
    copy_bench!($copy_name, $data);
    decode_bench_utf8!($name8, UTF_8, $data);
    decode_bench_utf16!($name16, UTF_8, $data);
    decode_bench_string!($string_name, UTF_8, $data);
    decode_bench_rust!($rust_name, UTF_8, $data);
    decode_bench_std!($std_name, $data);
    decode_bench_iconv!($iconv_name, UTF_8, $data);
    decode_bench_icu!($icu_name, UTF_8, $data);
    decode_bench_uconv!($uconv_name, UTF_8, $data);
    decode_bench_windows!($windows_name, UTF_8, 65001, $data);
    decode_bench_utf8!($legacy_name8, $encoding, $data);
    decode_bench_utf16!($legacy_name16, $encoding, $data);
    decode_bench_string!($legacy_string_name, $encoding, $data);
    decode_bench_rust!($legacy_rust_name, $encoding, $data);
    decode_bench_iconv!($legacy_iconv_name, $encoding, $data);
    decode_bench_icu!($legacy_icu_name, $encoding, $data);
    decode_bench_uconv!($legacy_uconv_name, $encoding, $data);
    decode_bench_windows!($legacy_windows_name, $encoding, $cp, $data);
     );
}

// BEGIN GENERATED CODE. PLEASE DO NOT EDIT.
// Instead, please regenerate using generate-encoding-data.py

encode_bench_utf8!(bench_encode_from_utf8_shift_jis,
                   SHIFT_JIS,
                   "wikipedia/ja.html");
encode_bench_utf16!(bench_encode_from_utf16_shift_jis,
                    SHIFT_JIS,
                    "wikipedia/ja.html");
encode_bench_vec!(bench_encode_vec_shift_jis, SHIFT_JIS, "wikipedia/ja.html");
encode_bench_rust!(bench_encode_rust_shift_jis, SHIFT_JIS, "wikipedia/ja.html");
encode_bench_iconv!(bench_encode_iconv_shift_jis, SHIFT_JIS, "wikipedia/ja.html");
encode_bench_icu!(bench_encode_icu_shift_jis, SHIFT_JIS, "wikipedia/ja.html");

// END GENERATED CODE
