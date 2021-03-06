#![feature(test)]

extern crate encoding_rs;
extern crate test;

#[cfg(feature = "safe_mem")]
extern crate safe_encoding_rs_mem;

#[cfg(feature = "rust-encoding")]
extern crate encoding;

#[cfg(any(
    feature = "icu",
    feature = "iconv",
    feature = "uconv",
    feature = "webkit"
))]
extern crate libc;

use test::Bencher;

#[cfg(any(
    feature = "icu",
    feature = "iconv",
    feature = "uconv",
    feature = "webkit"
))]
use std::ffi::CString;

#[allow(unused_imports)]
use std::borrow::Cow;

#[allow(dead_code)]
fn encode_utf16(str: &str, big_endian: bool) -> Vec<u8> {
    let mut vec = Vec::new();
    let mut iter = str.encode_utf16();
    loop {
        match iter.next() {
            None => {
                return vec;
            }
            Some(code_unit) => {
                let hi = (code_unit >> 8) as u8;
                let lo = (code_unit & 0xFF) as u8;
                if big_endian {
                    vec.push(hi);
                    vec.push(lo);
                } else {
                    vec.push(lo);
                    vec.push(hi);
                }
            }
        }
    }
}

#[allow(dead_code)]
fn encode(encoding: &'static encoding_rs::Encoding, str: &str) -> Vec<u8> {
    if encoding == encoding_rs::UTF_16BE {
        return encode_utf16(str, true);
    } else if encoding == encoding_rs::UTF_16LE {
        return encode_utf16(str, false);
    } else {
        let (cow, _, _) = encoding.encode(str);
        return cow.into_owned();
    }
}

macro_rules! decode_bench_user_defined {
    ($name:ident,
     $data:expr,
     $max:ident,
     $decode:ident) => {
        #[cfg(feature = "self")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let bytes = include_bytes!($data);
            let mut input = Vec::with_capacity(bytes.len());
            input.extend_from_slice(bytes);
            let mut decoder = encoding_rs::X_USER_DEFINED.new_decoder_without_bom_handling();
            let out_len = decoder.$max(input.len()).unwrap();
            let mut output = Vec::with_capacity(out_len);
            output.resize(out_len, 0);
            b.bytes = input.len() as u64;
            b.iter(|| {
                let (result, _, _, _) =
                    decoder.$decode(test::black_box(&input[..]), &mut output[..], false);
                // match result {
                //     encoding_rs::CoderResult::InputEmpty => {}
                //     encoding_rs::CoderResult::OutputFull => {
                //         unreachable!("Output buffer too short.");
                //     }
                // }
                test::black_box(&output);
            });
        }
    };
}

macro_rules! decode_bench_impl {
    ($name:ident,
     $encoding:ident,
     $data:expr,
     $max:ident,
     $decode:ident) => {
        #[cfg(feature = "self")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            let input = encode(encoding, utf8);
            let mut decoder = encoding.new_decoder_without_bom_handling();
            let out_len = decoder.$max(input.len()).unwrap();
            let mut output = Vec::with_capacity(out_len);
            output.resize(out_len, 0);
            b.bytes = input.len() as u64;
            b.iter(|| {
                let (result, _, _, _) =
                    decoder.$decode(test::black_box(&input[..]), &mut output[..], false);
                // match result {
                //     encoding_rs::CoderResult::InputEmpty => {}
                //     encoding_rs::CoderResult::OutputFull => {
                //         unreachable!("Output buffer too short.");
                //     }
                // }
                test::black_box(&output);
            });
        }
    };
}

macro_rules! decode_bench_utf8 {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        decode_bench_impl!(
            $name,
            $encoding,
            $data,
            max_utf8_buffer_length,
            decode_to_utf8
        );
    };
}

macro_rules! decode_bench_utf16 {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        decode_bench_impl!(
            $name,
            $encoding,
            $data,
            max_utf16_buffer_length,
            decode_to_utf16
        );
    };
}

macro_rules! encode_bench_utf8 {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "self")]
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
            let out_len = intermediate.len() + 20;
            let mut output = Vec::with_capacity(out_len);
            output.resize(out_len, 0);
            // Use output length to have something that can be compared
            b.bytes = intermediate.len() as u64;
            b.iter(|| {
                let (result, _, _, _) =
                    encoder.encode_from_utf8(test::black_box(&input[..]), &mut output[..], false);
                // match result {
                //     encoding_rs::CoderResult::InputEmpty => {}
                //     encoding_rs::CoderResult::OutputFull => {
                //         unreachable!("Output buffer too short.");
                //     }
                // }
                test::black_box(&output);
            });
        }
    };
}

macro_rules! encode_bench_utf16 {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "self")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            // Convert back and forth to avoid benching replacement, which other
            // libs won't do.
            let (intermediate, _, _) = encoding.encode(utf8);
            let mut decoder = encoding.new_decoder_without_bom_handling();
            let mut input: Vec<u16> =
                Vec::with_capacity(decoder.max_utf16_buffer_length(intermediate.len()).unwrap());
            let capacity = input.capacity();
            input.resize(capacity, 0u16);
            let (complete, _, written, _) =
                decoder.decode_to_utf16(&intermediate[..], &mut input[..], true);
            match complete {
                encoding_rs::CoderResult::InputEmpty => {}
                encoding_rs::CoderResult::OutputFull => {
                    unreachable!();
                }
            }
            input.truncate(written);
            let mut encoder = encoding.new_encoder();
            let out_len = intermediate.len() + 20;
            let mut output = Vec::with_capacity(out_len);
            output.resize(out_len, 0);
            // Use output length to have something that can be compared
            b.bytes = intermediate.len() as u64;
            b.iter(|| {
                let (result, _, _, _) =
                    encoder.encode_from_utf16(test::black_box(&input[..]), &mut output[..], false);
                // match result {
                //     encoding_rs::CoderResult::InputEmpty => {}
                //     encoding_rs::CoderResult::OutputFull => {
                //         unreachable!("Output buffer too short.");
                //     }
                // }
                test::black_box(&output);
            });
        }
    };
}

macro_rules! decode_bench_string {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "self")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            let input = encode(encoding, utf8);
            b.bytes = input.len() as u64;
            b.iter(|| {
                let (output, _) = encoding.decode_without_bom_handling(test::black_box(&input[..]));
                // if encoding == encoding_rs::UTF_8 {
                //     if let Cow::Owned(_) = output {
                //         unreachable!();
                //     }
                // }
                test::black_box(&output);
            });
        }
    };
}

#[cfg(feature = "self")]
#[bench]
fn bench_decode_to_string_user_defined(b: &mut Bencher) {
    let bytes = include_bytes!("wikipedia/binary.jpg");
    let mut input = Vec::with_capacity(bytes.len());
    input.extend_from_slice(bytes);
    b.bytes = input.len() as u64;
    b.iter(|| {
        let (output, _) = encoding_rs::X_USER_DEFINED.decode_without_bom_handling(test::black_box(&input[..]));
        // if encoding == encoding_rs::UTF_8 {
        //     if let Cow::Owned(_) = output {
        //         unreachable!();
        //     }
        // }
        test::black_box(&output);
    });
}

macro_rules! encode_bench_vec {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "self")]
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
        }
    };
}

macro_rules! label_bench_rs {
    ($name:ident,
     $label:expr) => {
        #[cfg(feature = "self")]
        #[bench]
        fn $name(b: &mut Bencher) {
            b.iter(|| {
                test::black_box(encoding_rs::Encoding::for_label($label.as_bytes()));
            });
        }
    };
}

// rust-encoding

macro_rules! decode_bench_rust {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "rust-encoding")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            let input = encode(encoding, utf8);
            let rust_encoding =
                encoding::label::encoding_from_whatwg_label(encoding.name()).unwrap();
            b.bytes = input.len() as u64;
            b.iter(|| {
                let output = rust_encoding
                    .decode(test::black_box(&input[..]), encoding::DecoderTrap::Replace);
                test::black_box(&output);
            });
        }
    };
}

macro_rules! encode_bench_rust {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "rust-encoding")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            // Convert back and forth to avoid benching replacement, which other
            // libs won't do.
            let (intermediate, _, _) = encoding.encode(utf8);
            let (cow, _) = encoding.decode_without_bom_handling(&intermediate[..]);
            let input = &cow[..];
            let rust_encoding =
                encoding::label::encoding_from_whatwg_label(encoding.name()).unwrap();
            // Use output length to have something that can be compared
            b.bytes = intermediate.len() as u64;
            b.iter(|| {
                let output = rust_encoding
                    .encode(test::black_box(&input[..]), encoding::EncoderTrap::Replace);
                test::black_box(&output);
            });
        }
    };
}

#[cfg(feature = "rust-encoding")]
#[bench]
fn bench_rust_to_string_user_defined(b: &mut Bencher) {
    let bytes = include_bytes!("wikipedia/binary.jpg");
    let mut input = Vec::with_capacity(bytes.len());
    input.extend_from_slice(bytes);
    let rust_encoding =
        encoding::label::encoding_from_whatwg_label("x-user-defined").unwrap();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let output = rust_encoding
            .decode(test::black_box(&input[..]), encoding::DecoderTrap::Replace);
        test::black_box(&output);
    });
}

macro_rules! label_bench_rust {
    ($name:ident,
     $label:expr) => {
        #[cfg(feature = "rust-encoding")]
        #[bench]
        fn $name(b: &mut Bencher) {
            b.iter(|| {
                test::black_box(encoding::label::encoding_from_whatwg_label($label));
            });
        }
    };
}

// standard library

macro_rules! decode_bench_std {
    ($name:ident,
     $data:expr) => {
        #[cfg(feature = "standard-library")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut input = Vec::new();
            input.extend_from_slice(include_bytes!($data));
            b.bytes = input.len() as u64;
            b.iter(|| {
                let output = std::str::from_utf8(&input[..]);
                assert!(output.is_ok());
                test::black_box(&output);
            });
        }
    };
}

macro_rules! copy_bench {
    ($name:ident,
     $data:expr) => {
        #[cfg(feature = "memcpy")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut input = Vec::new();
            input.extend_from_slice(include_bytes!($data));
            let mut output = Vec::with_capacity(input.len());
            b.bytes = input.len() as u64;
            b.iter(|| {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        test::black_box(input.as_slice().as_ptr()),
                        output.as_mut_slice().as_mut_ptr(),
                        input.len(),
                    );
                }
                test::black_box(&output);
            });
        }
    };
}

// iconv

#[cfg(feature = "iconv")]
extern "C" {
    fn iconv_open(
        tocode: *const std::os::raw::c_char,
        fromcode: *const std::os::raw::c_char,
    ) -> *mut libc::c_void;
    fn iconv_close(cd: *mut libc::c_void) -> libc::c_int;
    fn iconv(
        cd: *mut libc::c_void,
        inbuf: *mut *mut u8,
        inbytesleft: *mut usize,
        outbuf: *mut *mut u8,
        outbytesleft: *mut usize,
    ) -> usize;
}

#[cfg(feature = "iconv")]
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
     $data:expr) => {
        #[cfg(feature = "iconv")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            let input = encode(encoding, utf8);
            let decoder = encoding.new_decoder_without_bom_handling();
            let out_len = decoder.max_utf8_buffer_length(input.len()).unwrap();
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
                    let mut input_ptr = test::black_box(input.as_ptr()) as *mut u8;
                    let mut output_ptr = output.as_mut_ptr();
                    let input_ptr_ptr = &mut input_ptr as *mut *mut u8;
                    let output_ptr_ptr = &mut output_ptr as *mut *mut u8;
                    let mut input_left = input.len();
                    let mut output_left = output.len();
                    let input_left_ptr = &mut input_left as *mut usize;
                    let output_left_ptr = &mut output_left as *mut usize;
                    iconv(
                        cd,
                        input_ptr_ptr,
                        input_left_ptr,
                        output_ptr_ptr,
                        output_left_ptr,
                    );
                    assert_eq!(input_left, 0usize);
                    test::black_box(&output);
                }
            });
            unsafe {
                iconv_close(cd);
            }
        }
    };
}

macro_rules! encode_bench_iconv {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "iconv")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            // Convert back and forth to avoid benching replacement, which other
            // libs won't do.
            let (intermediate, _, _) = encoding.encode(utf8);
            let (cow, _) = encoding.decode_without_bom_handling(&intermediate[..]);
            let input = &cow[..];
            let out_len = intermediate.len() + 20;
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
                    let mut input_ptr = test::black_box(input.as_ptr()) as *mut u8;
                    let mut output_ptr = output.as_mut_ptr();
                    let input_ptr_ptr = &mut input_ptr as *mut *mut u8;
                    let output_ptr_ptr = &mut output_ptr as *mut *mut u8;
                    let mut input_left = input.len();
                    let mut output_left = output.len();
                    let input_left_ptr = &mut input_left as *mut usize;
                    let output_left_ptr = &mut output_left as *mut usize;
                    iconv(
                        cd,
                        input_ptr_ptr,
                        input_left_ptr,
                        output_ptr_ptr,
                        output_left_ptr,
                    );
                    assert_eq!(input_left, 0usize);
                    test::black_box(&output);
                }
            });
            unsafe {
                iconv_close(cd);
            }
        }
    };
}

// ICU

#[cfg(feature = "icu")]
#[link(name = "icuuc")]
extern "C" {
    fn ucnv_open_60(
        label: *const std::os::raw::c_char,
        error: *mut libc::c_int,
    ) -> *mut libc::c_void;
    fn ucnv_close_60(cnv: *mut libc::c_void);
    fn ucnv_toUChars_60(
        cnv: *mut libc::c_void,
        dst: *mut u16,
        dst_len: i32,
        src: *const u8,
        src_len: i32,
        error: *mut libc::c_int,
    ) -> i32;
    fn ucnv_fromUChars_60(
        cnv: *mut libc::c_void,
        dst: *mut u8,
        dst_len: i32,
        src: *const u16,
        src_len: i32,
        error: *mut libc::c_int,
    ) -> i32;
}

#[cfg(feature = "icu")]
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
     $data:expr) => {
        #[cfg(feature = "icu")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            let input = encode(encoding, utf8);
            let decoder = encoding.new_decoder_without_bom_handling();
            let out_len = decoder.max_utf16_buffer_length(input.len()).unwrap();
            let mut output: Vec<u16> = Vec::with_capacity(out_len);
            output.resize(out_len, 0);
            let label = CString::new(icu_name(encoding)).unwrap();
            let mut error: libc::c_int = 0;
            let cnv = unsafe { ucnv_open_60(label.as_ptr(), &mut error) };
            b.bytes = input.len() as u64;
            b.iter(|| {
                unsafe {
                    ucnv_toUChars_60(
                        cnv,
                        output.as_mut_ptr(),
                        output.len() as i32,
                        test::black_box(input.as_ptr()),
                        input.len() as i32,
                        &mut error,
                    );
                }
                test::black_box(&output);
            });
            unsafe {
                ucnv_close_60(cnv);
            }
        }
    };
}

macro_rules! encode_bench_icu {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "icu")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            // Convert back and forth to avoid benching replacement, which other
            // libs won't do.
            let (intermediate, _, _) = encoding.encode(utf8);
            let mut decoder = encoding.new_decoder_without_bom_handling();
            let mut input: Vec<u16> =
                Vec::with_capacity(decoder.max_utf16_buffer_length(intermediate.len()).unwrap());
            let capacity = input.capacity();
            input.resize(capacity, 0u16);
            let (complete, _, written, _) =
                decoder.decode_to_utf16(&intermediate[..], &mut input[..], true);
            match complete {
                encoding_rs::CoderResult::InputEmpty => {}
                encoding_rs::CoderResult::OutputFull => {
                    unreachable!();
                }
            }
            input.truncate(written);
            let out_len = intermediate.len() + 20;
            let mut output: Vec<u8> = Vec::with_capacity(out_len);
            output.resize(out_len, 0);
            let label = CString::new(icu_name(encoding)).unwrap();
            let mut error: libc::c_int = 0;
            let cnv = unsafe { ucnv_open_60(label.as_ptr(), &mut error) };
            // Use output length to have something that can be compared
            b.bytes = intermediate.len() as u64;
            b.iter(|| {
                unsafe {
                    ucnv_fromUChars_60(
                        cnv,
                        output.as_mut_ptr(),
                        output.len() as i32,
                        test::black_box(input.as_ptr()),
                        input.len() as i32,
                        &mut error,
                    );
                }
                test::black_box(&output);
            });
            unsafe {
                ucnv_close_60(cnv);
            }
        }
    };
}

// kewb

#[cfg(feature = "kewb")]
#[link(name = "kewb_glue", kind = "static")]
extern "C" {
    fn kewb_sse2_utf8_to_utf16(src: *const u8, src_len: usize, dst: *mut u16) -> usize;
}

macro_rules! decode_bench_kewb {
    ($name:ident,
     $data:expr) => {
        #[cfg(feature = "kewb")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let utf8 = include_str!($data);
            let input = String::from(utf8);
            let mut output: Vec<u16> = Vec::with_capacity(input.len());
            output.resize(input.len(), 0);
            b.bytes = input.len() as u64;
            b.iter(|| {
                let black_boxed_input = test::black_box(&input[..]);
                let out_len = unsafe {
                    kewb_sse2_utf8_to_utf16(
                        black_boxed_input.as_ptr(),
                        black_boxed_input.len(),
                        output.as_mut_ptr(),
                    )
                };
                test::black_box(&output);
                test::black_box(out_len);
            });
        }
    };
}

// webkit

#[cfg(feature = "webkit")]
#[link(name = "webkit2gtk-4.0")]
extern "C" {
    fn webkit_new_text_codec(name: *const std::os::raw::c_char) -> *mut libc::c_void;
    fn webkit_text_codec_decode(
        codec: *mut libc::c_void,
        src: *const u8,
        src_len: usize,
    ) -> *mut libc::c_void;
    fn webkit_text_codec_encode(
        codec: *mut libc::c_void,
        src: *const u16,
        src_len: usize,
    ) -> *mut libc::c_void;

    fn webkit_delete_string_holder(holder: *mut libc::c_void);
    fn webkit_delete_vector_holder(holder: *mut libc::c_void);
    fn webkit_delete_text_codec(codec: *mut libc::c_void);
}

macro_rules! decode_bench_webkit {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "webkit")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            let input = encode(encoding, utf8);
            let label = CString::new(encoding.name()).unwrap();
            let codec = unsafe { webkit_new_text_codec(label.as_ptr()) };
            b.bytes = input.len() as u64;
            b.iter(|| unsafe {
                let holder =
                    webkit_text_codec_decode(codec, test::black_box(input.as_ptr()), input.len());
                webkit_delete_string_holder(test::black_box(holder));
            });
            unsafe {
                webkit_delete_text_codec(codec);
            }
        }
    };
}

macro_rules! encode_bench_webkit {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "webkit")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            // Convert back and forth to avoid benching replacement, which other
            // libs won't do.
            let (intermediate, _, _) = encoding.encode(utf8);
            let mut decoder = encoding.new_decoder_without_bom_handling();
            let mut input: Vec<u16> =
                Vec::with_capacity(decoder.max_utf16_buffer_length(intermediate.len()).unwrap());
            let capacity = input.capacity();
            input.resize(capacity, 0u16);
            let (complete, _, written, _) =
                decoder.decode_to_utf16(&intermediate[..], &mut input[..], true);
            match complete {
                encoding_rs::CoderResult::InputEmpty => {}
                encoding_rs::CoderResult::OutputFull => {
                    unreachable!();
                }
            }
            input.truncate(written);
            let label = CString::new(encoding.name()).unwrap();
            let codec = unsafe { webkit_new_text_codec(label.as_ptr()) };
            b.bytes = intermediate.len() as u64;
            b.iter(|| unsafe {
                let holder =
                    webkit_text_codec_encode(codec, test::black_box(input.as_ptr()), input.len());
                webkit_delete_vector_holder(test::black_box(holder));
            });
            unsafe {
                webkit_delete_text_codec(codec);
            }
        }
    };
}

#[cfg(feature = "webkit")]
#[bench]
fn bench_webkit_to_utf16_user_defined(b: &mut Bencher) {
    let bytes = include_bytes!("wikipedia/binary.jpg");
    let mut input = Vec::with_capacity(bytes.len());
    input.extend_from_slice(bytes);
    let name = "x-user-defined";

    let label = CString::new(name).unwrap();
    let codec = unsafe { webkit_new_text_codec(label.as_ptr()) };
    b.bytes = input.len() as u64;
    b.iter(|| unsafe {
        let holder = webkit_text_codec_decode(codec, test::black_box(input.as_ptr()), input.len());
        webkit_delete_string_holder(test::black_box(holder));
    });
    unsafe {
        webkit_delete_text_codec(codec);
    }
}

// uconv

#[cfg(feature = "uconv")]
#[link(name = "stdc++")]
extern "C" {}

#[cfg(feature = "uconv")]
#[link(name = "mozglue", kind = "static")]
extern "C" {}

#[cfg(feature = "uconv")]
#[link(name = "xul")]
extern "C" {
    fn NS_InitXPCOM2(
        manager: *mut *mut libc::c_void,
        bin_dir: *mut libc::c_void,
        provider: *mut libc::c_void,
    ) -> libc::c_int;
    fn NS_CreateUnicodeDecoder(name: *const u8, name_len: usize) -> *mut libc::c_void;
    fn NS_ReleaseUnicodeDecoder(dec: *mut libc::c_void);
    fn NS_DecodeWithUnicodeDecoder(
        dec: *mut libc::c_void,
        src: *const u8,
        src_len: i32,
        dst: *mut u16,
        dst_len: i32,
    );
    fn NS_CreateUnicodeEncoder(name: *const u8, name_len: usize) -> *mut libc::c_void;
    fn NS_ReleaseUnicodeEncoder(enc: *mut libc::c_void);
    fn NS_EncodeWithUnicodeEncoder(
        enc: *mut libc::c_void,
        src: *const u16,
        src_len: i32,
        dst: *mut u8,
        dst_len: i32,
    );
    fn NS_FindEncodingForLabel(name: *const u8, name_len: usize) -> i32;
}

#[cfg(feature = "uconv")]
static mut XPCOM_INITIALIZED: bool = false;

#[cfg(feature = "uconv")]
fn init_xpcom() {
    unsafe {
        if !XPCOM_INITIALIZED {
            XPCOM_INITIALIZED = true;
            NS_InitXPCOM2(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        }
    }
}

#[cfg(feature = "uconv")]
#[bench]
fn bench_uconv_to_utf16_user_defined(b: &mut Bencher) {
    init_xpcom();
    let bytes = include_bytes!("wikipedia/binary.jpg");
    let mut input = Vec::with_capacity(bytes.len());
    input.extend_from_slice(bytes);
    let out_len = input.len() + 2;
    let mut output: Vec<u16> = Vec::with_capacity(out_len);
    output.resize(out_len, 0);
    let name = "x-user-defined";
    let dec = unsafe { NS_CreateUnicodeDecoder(name.as_ptr(), name.len()) };
    b.bytes = input.len() as u64;
    b.iter(|| {
        unsafe {
            NS_DecodeWithUnicodeDecoder(
                dec,
                input.as_ptr(),
                input.len() as i32,
                output.as_mut_ptr(),
                output.len() as i32,
            );
        }
        test::black_box(&output);
    });
    unsafe {
        NS_ReleaseUnicodeDecoder(dec);
    }
}

macro_rules! decode_bench_uconv {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "uconv")]
        #[bench]
        fn $name(b: &mut Bencher) {
            init_xpcom();
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            let input = encode(encoding, utf8);
            let decoder = encoding.new_decoder_without_bom_handling();
            let out_len = decoder.max_utf16_buffer_length(input.len()).unwrap();
            let mut output: Vec<u16> = Vec::with_capacity(out_len);
            output.resize(out_len, 0);
            let name = encoding.name();
            let dec = unsafe { NS_CreateUnicodeDecoder(name.as_ptr(), name.len()) };
            b.bytes = input.len() as u64;
            b.iter(|| {
                unsafe {
                    NS_DecodeWithUnicodeDecoder(
                        dec,
                        input.as_ptr(),
                        input.len() as i32,
                        output.as_mut_ptr(),
                        output.len() as i32,
                    );
                }
                test::black_box(&output);
            });
            unsafe {
                NS_ReleaseUnicodeDecoder(dec);
            }
        }
    };
}

macro_rules! encode_bench_uconv {
    ($name:ident,
     $encoding:ident,
     $data:expr) => {
        #[cfg(feature = "uconv")]
        #[bench]
        fn $name(b: &mut Bencher) {
            init_xpcom();
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            // Convert back and forth to avoid benching replacement, which other
            // libs won't do.
            let (intermediate, _, _) = encoding.encode(utf8);
            let mut decoder = encoding.new_decoder_without_bom_handling();
            let mut input: Vec<u16> =
                Vec::with_capacity(decoder.max_utf16_buffer_length(intermediate.len()).unwrap());
            let capacity = input.capacity();
            input.resize(capacity, 0u16);
            let (complete, _, written, _) =
                decoder.decode_to_utf16(&intermediate[..], &mut input[..], true);
            match complete {
                encoding_rs::CoderResult::InputEmpty => {}
                encoding_rs::CoderResult::OutputFull => {
                    unreachable!();
                }
            }
            input.truncate(written);
            let out_len = intermediate.len() + 20;
            let mut output: Vec<u8> = Vec::with_capacity(out_len);
            output.resize(out_len, 0);
            let name = encoding.name();
            let enc = unsafe { NS_CreateUnicodeEncoder(name.as_ptr(), name.len()) };
            // Use output length to have something that can be compared
            b.bytes = intermediate.len() as u64;
            b.iter(|| {
                unsafe {
                    NS_EncodeWithUnicodeEncoder(
                        enc,
                        input.as_ptr(),
                        input.len() as i32,
                        output.as_mut_ptr(),
                        output.len() as i32,
                    );
                }
                test::black_box(&output);
            });
            unsafe {
                NS_ReleaseUnicodeEncoder(enc);
            }
        }
    };
}

macro_rules! label_bench_uconv {
    ($name:ident,
     $label:expr) => {
        #[cfg(feature = "uconv")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let label = $label;
            b.iter(|| {
                test::black_box(unsafe { NS_FindEncodingForLabel(label.as_ptr(), label.len()) });
            });
        }
    };
}

// Windows built-in

#[cfg(feature = "win32")]
#[link(name = "Kernel32")]
extern "system" {
    fn MultiByteToWideChar(
        code_page: libc::c_uint,
        flags: libc::c_ulong,
        src: *const u8,
        src_len: libc::c_int,
        dst: *mut u16,
        dst_len: libc::c_int,
    ) -> libc::c_int;
    fn WideCharToMultiByte(
        code_page: libc::c_uint,
        flags: libc::c_ulong,
        src: *const u16,
        src_len: libc::c_int,
        dst: *mut u8,
        dst_len: libc::c_int,
        replacement: *const u8,
        used_replacement: *mut bool,
    ) -> libc::c_int;
}

macro_rules! decode_bench_windows {
    ($name:ident,
     $encoding:ident,
     $cp:expr,
     $data:expr) => {
        #[cfg(feature = "win32")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            let input = encode(encoding, utf8);
            let decoder = encoding.new_decoder_without_bom_handling();
            let out_len = decoder.max_utf16_buffer_length(input.len()).unwrap();
            let mut output: Vec<u16> = Vec::with_capacity(out_len);
            output.resize(out_len, 0);
            b.bytes = input.len() as u64;
            b.iter(|| {
                unsafe {
                    assert!(
                        MultiByteToWideChar(
                            $cp,
                            0,
                            test::black_box(input.as_ptr()),
                            input.len() as libc::c_int,
                            output.as_mut_ptr(),
                            output.len() as libc::c_int
                        ) != 0
                    );
                }
                test::black_box(&output);
            });
        }
    };
}

macro_rules! encode_bench_windows {
    ($name:ident,
     $encoding:ident,
     $cp:expr,
     $data:expr) => {
        #[cfg(feature = "win32")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let encoding = encoding_rs::$encoding;
            let utf8 = include_str!($data);
            // Convert back and forth to avoid benching replacement, which other
            // libs won't do.
            let (intermediate, _, _) = encoding.encode(utf8);
            let mut decoder = encoding.new_decoder_without_bom_handling();
            let mut input: Vec<u16> =
                Vec::with_capacity(decoder.max_utf16_buffer_length(intermediate.len()).unwrap());
            let capacity = input.capacity();
            input.resize(capacity, 0u16);
            let (complete, _, written, _) =
                decoder.decode_to_utf16(&intermediate[..], &mut input[..], true);
            match complete {
                encoding_rs::CoderResult::InputEmpty => {}
                encoding_rs::CoderResult::OutputFull => {
                    unreachable!();
                }
            }
            input.truncate(written);
            let out_len = intermediate.len() + 20;
            let mut output: Vec<u8> = Vec::with_capacity(out_len);
            output.resize(out_len, 0);
            // Use output length to have something that can be compared
            b.bytes = intermediate.len() as u64;
            b.iter(|| {
                unsafe {
                    assert!(
                        WideCharToMultiByte(
                            $cp,
                            0,
                            test::black_box(input.as_ptr()),
                            input.len() as libc::c_int,
                            output.as_mut_ptr(),
                            output.len() as libc::c_int,
                            std::ptr::null(),
                            std::ptr::null_mut()
                        ) != 0
                    );
                }
                test::black_box(&output);
            });
        }
    };
}

// mem

// Note that the benches below always copy static data to a Vec/String in order to
// test malloc-provided alignment.

fn round_str_index_up(s: &str, i: usize) -> usize {
    let b = s.as_bytes();
    let mut idx = i;
    while (idx < b.len()) && ((b[idx] & 0xC0) == 0x80) {
        idx += 1;
    }
    idx
}

macro_rules! mem_bench_is_u8 {
    ($name:ident,
     $safe_name:ident,
     $func:ident,
     $len:expr,
     $data:expr) => {
        #[cfg(feature = "mem")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let bytes = include_bytes!($data);
            let mut v = Vec::with_capacity(bytes.len());
            v.extend_from_slice(bytes);
            let truncated = &v[..$len];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(encoding_rs::mem::$func(test::black_box(truncated)));
                }
            });
        }

        #[cfg(feature = "safe_mem")]
        #[bench]
        fn $safe_name(b: &mut Bencher) {
            let bytes = include_bytes!($data);
            let mut v = Vec::with_capacity(bytes.len());
            v.extend_from_slice(bytes);
            let truncated = &v[..$len];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(safe_encoding_rs_mem::$func(test::black_box(truncated)));
                }
            });
        }
    };
}

const MEM_INNER_LOOP: u64 = 500;

macro_rules! mem_bench_is_str {
    ($name:ident,
     $safe_name:ident,
     $func:ident,
     $len:expr,
     $data:expr) => {
        #[cfg(feature = "mem")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let s = include_str!($data);
            let mut string = String::with_capacity(s.len());
            string.push_str(s);
            let truncated = &string[..round_str_index_up(&string[..], $len)];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(encoding_rs::mem::$func(test::black_box(truncated)));
                }
            });
        }

        #[cfg(feature = "safe_mem")]
        #[bench]
        fn $safe_name(b: &mut Bencher) {
            let s = include_str!($data);
            let mut string = String::with_capacity(s.len());
            string.push_str(s);
            let truncated = &string[..round_str_index_up(&string[..], $len)];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(safe_encoding_rs_mem::$func(test::black_box(truncated)));
                }
            });
        }
    };
}

macro_rules! mem_bench_is_u16 {
    ($name:ident,
     $safe_name:ident,
     $func:ident,
     $len:expr,
     $data:expr) => {
        #[cfg(feature = "mem")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let s = include_str!($data);
            let u: Vec<u16> = s.encode_utf16().collect();
            let truncated = &u[..$len];
            b.bytes = ((truncated.len() * 2) as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(encoding_rs::mem::$func(test::black_box(truncated)));
                }
            });
        }

        #[cfg(feature = "safe_mem")]
        #[bench]
        fn $safe_name(b: &mut Bencher) {
            let s = include_str!($data);
            let u: Vec<u16> = s.encode_utf16().collect();
            let truncated = &u[..$len];
            b.bytes = ((truncated.len() * 2) as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(safe_encoding_rs_mem::$func(test::black_box(truncated)));
                }
            });
        }
    };
}

macro_rules! mem_bench_mut_u16 {
    ($name:ident,
     $safe_name:ident,
     $func:ident,
     $len:expr,
     $data:expr) => {
        #[cfg(feature = "mem")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let s = include_str!($data);
            let mut u: Vec<u16> = s.encode_utf16().collect();
            let truncated = &mut u[..$len];
            b.bytes = ((truncated.len() * 2) as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(encoding_rs::mem::$func(test::black_box(truncated)));
                }
            });
        }

        #[cfg(feature = "safe_mem")]
        #[bench]
        fn $safe_name(b: &mut Bencher) {
            let s = include_str!($data);
            let mut u: Vec<u16> = s.encode_utf16().collect();
            let truncated = &mut u[..$len];
            b.bytes = ((truncated.len() * 2) as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(safe_encoding_rs_mem::$func(test::black_box(truncated)));
                }
            });
        }
    };
}

macro_rules! mem_bench_u8_to_u16 {
    ($name:ident,
     $safe_name:ident,
     $func:ident,
     $len:expr,
     $data:expr) => {
        #[cfg(feature = "mem")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let bytes = include_bytes!($data);
            let mut v = Vec::with_capacity(bytes.len());
            v.extend_from_slice(bytes);
            let truncated = &v[..$len];
            let capacity = $len * 4;
            let mut vec = Vec::with_capacity(capacity);
            vec.resize(capacity, 0u16);
            let dst = &mut vec[..];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(encoding_rs::mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }

        #[cfg(feature = "safe_mem")]
        #[bench]
        fn $safe_name(b: &mut Bencher) {
            let bytes = include_bytes!($data);
            let mut v = Vec::with_capacity(bytes.len());
            v.extend_from_slice(bytes);
            let truncated = &v[..$len];
            let capacity = $len * 4;
            let mut vec = Vec::with_capacity(capacity);
            vec.resize(capacity, 0u16);
            let dst = &mut vec[..];
            b.bytes = truncated.len() as u64;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(safe_encoding_rs_mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }
    };
}

macro_rules! mem_bench_str_to_u16 {
    ($name:ident,
     $safe_name:ident,
     $func:ident,
     $len:expr,
     $data:expr) => {
        #[cfg(feature = "mem")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let s = include_str!($data);
            let mut string = String::with_capacity(s.len());
            string.push_str(s);
            let truncated = &string[..round_str_index_up(&string[..], $len)];
            let capacity = $len * 4;
            let mut vec = Vec::with_capacity(capacity);
            vec.resize(capacity, 0u16);
            let dst = &mut vec[..];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(encoding_rs::mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }

        #[cfg(feature = "safe_mem")]
        #[bench]
        fn $safe_name(b: &mut Bencher) {
            let s = include_str!($data);
            let mut string = String::with_capacity(s.len());
            string.push_str(s);
            let truncated = &string[..round_str_index_up(&string[..], $len)];
            let capacity = $len * 4;
            let mut vec = Vec::with_capacity(capacity);
            vec.resize(capacity, 0u16);
            let dst = &mut vec[..];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(safe_encoding_rs_mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }
    };
}

macro_rules! mem_bench_u16_to_u8 {
    ($name:ident,
     $safe_name:ident,
     $func:ident,
     $len:expr,
     $data:expr) => {
        #[cfg(feature = "mem")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let s = include_str!($data);
            let u: Vec<u16> = s.encode_utf16().collect();
            let truncated = &u[..$len];
            let capacity = $len * 4;
            let mut vec = Vec::with_capacity(capacity);
            vec.resize(capacity, 0u8);
            let dst = &mut vec[..];
            b.bytes = ((truncated.len() * 2) as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(encoding_rs::mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }

        #[cfg(feature = "safe_mem")]
        #[bench]
        fn $safe_name(b: &mut Bencher) {
            let s = include_str!($data);
            let u: Vec<u16> = s.encode_utf16().collect();
            let truncated = &u[..$len];
            let capacity = $len * 4;
            let mut vec = Vec::with_capacity(capacity);
            vec.resize(capacity, 0u8);
            let dst = &mut vec[..];
            b.bytes = ((truncated.len() * 2) as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(safe_encoding_rs_mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }
    };
}

macro_rules! mem_bench_u16_to_str {
    ($name:ident,
     $safe_name:ident,
     $func:ident,
     $len:expr,
     $data:expr) => {
        #[cfg(feature = "mem")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let s = include_str!($data);
            let u: Vec<u16> = s.encode_utf16().collect();
            let truncated = &u[..$len];
            let capacity = $len * 4;
            let mut string = String::with_capacity(capacity);
            for _ in 0..capacity {
                string.push('\u{0000}');
            }
            let dst = &mut string[..];
            b.bytes = ((truncated.len() * 2) as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(encoding_rs::mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }

        #[cfg(feature = "safe_mem")]
        #[bench]
        fn $safe_name(b: &mut Bencher) {
            let s = include_str!($data);
            let u: Vec<u16> = s.encode_utf16().collect();
            let truncated = &u[..$len];
            let capacity = $len * 4;
            let mut string = String::with_capacity(capacity);
            for _ in 0..capacity {
                string.push('\u{0000}');
            }
            let dst = &mut string[..];
            b.bytes = ((truncated.len() * 2) as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(safe_encoding_rs_mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }
    };
}

macro_rules! mem_bench_u8_to_u8 {
    ($name:ident,
     $safe_name:ident,
     $func:ident,
     $len:expr,
     $data:expr) => {
        #[cfg(feature = "mem")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let bytes = include_bytes!($data);
            let mut v = Vec::with_capacity(bytes.len());
            v.extend_from_slice(bytes);
            let truncated = &v[..$len];
            let capacity = $len * 4;
            let mut vec = Vec::with_capacity(capacity);
            vec.resize(capacity, 0u8);
            let dst = &mut vec[..];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(encoding_rs::mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }

        #[cfg(feature = "safe_mem")]
        #[bench]
        fn $safe_name(b: &mut Bencher) {
            let bytes = include_bytes!($data);
            let mut v = Vec::with_capacity(bytes.len());
            v.extend_from_slice(bytes);
            let truncated = &v[..$len];
            let capacity = $len * 4;
            let mut vec = Vec::with_capacity(capacity);
            vec.resize(capacity, 0u8);
            let dst = &mut vec[..];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(safe_encoding_rs_mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }
    };
}

macro_rules! mem_bench_u8_to_str {
    ($name:ident,
     $safe_name:ident,
     $func:ident,
     $len:expr,
     $data:expr) => {
        #[cfg(feature = "mem")]
        #[bench]
        fn $name(b: &mut Bencher) {
            let bytes = include_bytes!($data);
            let mut v = Vec::with_capacity(bytes.len());
            v.extend_from_slice(bytes);
            let truncated = &v[..$len];
            let capacity = $len * 4;
            let mut string = String::with_capacity(capacity);
            for _ in 0..capacity {
                string.push('\u{0000}');
            }
            let dst = &mut string[..];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(encoding_rs::mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }

        #[cfg(feature = "safe_mem")]
        #[bench]
        fn $safe_name(b: &mut Bencher) {
            let bytes = include_bytes!($data);
            let mut v = Vec::with_capacity(bytes.len());
            v.extend_from_slice(bytes);
            let truncated = &v[..$len];
            let capacity = $len * 4;
            let mut string = String::with_capacity(capacity);
            for _ in 0..capacity {
                string.push('\u{0000}');
            }
            let dst = &mut string[..];
            b.bytes = (truncated.len() as u64) * MEM_INNER_LOOP;
            b.iter(|| {
                for _ in 0..MEM_INNER_LOOP {
                    test::black_box(safe_encoding_rs_mem::$func(
                        test::black_box(truncated),
                        test::black_box(dst),
                    ));
                }
            });
        }
    };
}

// Invocations

macro_rules! label_bench {
    ($name:ident,
     $rust_name:ident,
     $uconv_name:ident,
     $label:expr) => {
        label_bench_rs!($name, $label);
        label_bench_rust!($rust_name, $label);
        label_bench_uconv!($uconv_name, $label);
    };
}

macro_rules! decode_bench {
    ($copy_name:ident,
     $name8:ident,
     $name16:ident,
     $string_name:ident,
     $rust_name:ident,
     $std_name:ident,
     $kewb_name:ident,
     $iconv_name:ident,
     $icu_name:ident,
     $uconv_name:ident,
     $webkit_name:ident,
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
     $data:expr) => {
        copy_bench!($copy_name, $data);
        decode_bench_utf8!($name8, UTF_8, $data);
        decode_bench_utf16!($name16, UTF_8, $data);
        decode_bench_string!($string_name, UTF_8, $data);
        decode_bench_rust!($rust_name, UTF_8, $data);
        decode_bench_std!($std_name, $data);
        decode_bench_kewb!($kewb_name, $data);
        decode_bench_iconv!($iconv_name, UTF_8, $data);
        decode_bench_icu!($icu_name, UTF_8, $data);
        decode_bench_uconv!($uconv_name, UTF_8, $data);
        decode_bench_webkit!($webkit_name, UTF_8, $data);
        decode_bench_windows!($windows_name, UTF_8, 65001, $data);
        decode_bench_legacy!(
            $legacy_name8,
            $legacy_name16,
            $legacy_string_name,
            $legacy_rust_name,
            $legacy_iconv_name,
            $legacy_icu_name,
            $legacy_uconv_name,
            $legacy_windows_name,
            $encoding,
            $cp,
            $data
        );
    };
}

macro_rules! decode_bench_legacy {
    ($legacy_name8:ident,
     $legacy_name16:ident,
     $legacy_string_name:ident,
     $legacy_rust_name:ident,
     $legacy_iconv_name:ident,
     $legacy_icu_name:ident,
     $legacy_uconv_name:ident,
     $legacy_windows_name:ident,
     $encoding:ident,
     $cp:expr,
     $data:expr) => {
        decode_bench_utf8!($legacy_name8, $encoding, $data);
        decode_bench_utf16!($legacy_name16, $encoding, $data);
        decode_bench_string!($legacy_string_name, $encoding, $data);
        decode_bench_rust!($legacy_rust_name, $encoding, $data);
        decode_bench_iconv!($legacy_iconv_name, $encoding, $data);
        decode_bench_icu!($legacy_icu_name, $encoding, $data);
        decode_bench_uconv!($legacy_uconv_name, $encoding, $data);
        decode_bench_windows!($legacy_windows_name, $encoding, $cp, $data);
    };
}

macro_rules! encode_bench {
    ($name8:ident,
     $name16:ident,
     $vec_name:ident,
     $rust_name:ident,
     $iconv_name:ident,
     $icu_name:ident,
     $uconv_name:ident,
     $webkit_name:ident,
     $windows_name:ident,
     $legacy_name8:ident,
     $legacy_name16:ident,
     $legacy_vec_name:ident,
     $legacy_rust_name:ident,
     $legacy_iconv_name:ident,
     $legacy_icu_name:ident,
     $legacy_uconv_name:ident,
     $legacy_windows_name:ident,
     $encoding:ident,
     $cp:expr,
     $data:expr) => {
        encode_bench_utf8!($name8, UTF_8, $data);
        encode_bench_utf16!($name16, UTF_8, $data);
        encode_bench_vec!($vec_name, UTF_8, $data);
        encode_bench_rust!($rust_name, UTF_8, $data);
        encode_bench_iconv!($iconv_name, UTF_8, $data);
        encode_bench_icu!($icu_name, UTF_8, $data);
        encode_bench_uconv!($uconv_name, UTF_8, $data);
        encode_bench_webkit!($webkit_name, UTF_8, $data);
        encode_bench_windows!($windows_name, UTF_8, 65001, $data);
        encode_bench_legacy!(
            $legacy_name8,
            $legacy_name16,
            $legacy_vec_name,
            $legacy_rust_name,
            $legacy_iconv_name,
            $legacy_icu_name,
            $legacy_uconv_name,
            $legacy_windows_name,
            $encoding,
            $cp,
            $data
        );
    };
}

macro_rules! encode_bench_legacy {
    ($legacy_name8:ident,
     $legacy_name16:ident,
     $legacy_vec_name:ident,
     $legacy_rust_name:ident,
     $legacy_iconv_name:ident,
     $legacy_icu_name:ident,
     $legacy_uconv_name:ident,
     $legacy_windows_name:ident,
     $encoding:ident,
     $cp:expr,
     $data:expr) => {
        encode_bench_utf8!($legacy_name8, $encoding, $data);
        encode_bench_utf16!($legacy_name16, $encoding, $data);
        encode_bench_vec!($legacy_vec_name, $encoding, $data);
        encode_bench_rust!($legacy_rust_name, $encoding, $data);
        encode_bench_iconv!($legacy_iconv_name, $encoding, $data);
        encode_bench_icu!($legacy_icu_name, $encoding, $data);
        encode_bench_uconv!($legacy_uconv_name, $encoding, $data);
        encode_bench_windows!($legacy_windows_name, $encoding, $cp, $data);
    };
}

label_bench!(
    bench_label_rs_utf_8,
    bench_label_rust_utf_8,
    bench_label_uconv_utf_8,
    "utf-8"
);
label_bench!(
    bench_label_rs_utf_8_upper,
    bench_label_rust_utf_8_upper,
    bench_label_uconv_utf_8_upper,
    "UTF-8"
);
label_bench!(
    bench_label_rs_cseucpkdfmtjapanesx,
    bench_label_rust_cseucpkdfmtjapanesx,
    bench_label_uconv_cseucpkdfmtjapanesx,
    "CSEUCPKDFMTJAPANESX"
);
label_bench!(
    bench_label_rs_xseucpkdfmtjapanese,
    bench_label_rust_xseucpkdfmtjapanese,
    bench_label_uconv_xseucpkdfmtjapanese,
    "XSEUCPKDFMTJAPANESE"
);

decode_bench_string!(
    bench_decode_to_string_jquerycat,
    UTF_8,
    "jquery/jquery-cat.js"
);

copy_bench!(bench_copy_jquery, "jquery/jquery-3.1.1.min.js");
decode_bench_utf8!(
    bench_decode_to_utf8_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_utf16!(
    bench_decode_to_utf16_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_string!(
    bench_decode_to_string_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_rust!(
    bench_rust_to_string_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_std!(bench_std_validation_jquery, "jquery/jquery-3.1.1.min.js");
decode_bench_kewb!(bench_kewb_to_utf16_jquery, "jquery/jquery-3.1.1.min.js");
decode_bench_iconv!(
    bench_iconv_to_utf8_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_icu!(
    bench_icu_to_utf16_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_uconv!(
    bench_uconv_to_utf16_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_windows!(
    bench_windows_to_utf16_jquery,
    UTF_8,
    65001,
    "jquery/jquery-3.1.1.min.js"
);

decode_bench_utf8!(
    bench_decode_to_utf8_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_utf16!(
    bench_decode_to_utf16_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_string!(
    bench_decode_to_string_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_rust!(
    bench_rust_to_string_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_iconv!(
    bench_iconv_to_utf8_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_icu!(
    bench_icu_to_utf16_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
decode_bench_uconv!(
    bench_uconv_to_utf16_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
// decode_bench_webkit!(bench_webkit_to_utf16_jquery_windows_1252,
//                     WINDOWS_1252,
//                     "jquery/jquery-3.1.1.min.js");
decode_bench_windows!(
    bench_windows_to_utf16_jquery_windows_1252,
    WINDOWS_1252,
    1252,
    "jquery/jquery-3.1.1.min.js"
);

encode_bench_utf8!(
    bench_encode_from_utf8_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_utf16!(
    bench_encode_from_utf16_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_vec!(
    bench_encode_to_vec_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_rust!(
    bench_rust_to_vec_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_iconv!(
    bench_iconv_from_utf8_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_icu!(
    bench_icu_from_utf16_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_uconv!(
    bench_uconv_from_utf16_jquery,
    UTF_8,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_webkit!(bench_webkit_from_utf16_jquery,
                    UTF_8,
                    "jquery/jquery-3.1.1.min.js");
encode_bench_windows!(
    bench_windows_from_utf16_jquery,
    UTF_8,
    65001,
    "jquery/jquery-3.1.1.min.js"
);

encode_bench_utf8!(
    bench_encode_from_utf8_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_utf16!(
    bench_encode_from_utf16_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_vec!(
    bench_encode_to_vec_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_rust!(
    bench_rust_to_vec_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_iconv!(
    bench_iconv_from_utf8_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_icu!(
    bench_icu_from_utf16_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
encode_bench_uconv!(
    bench_uconv_from_utf16_jquery_windows_1252,
    WINDOWS_1252,
    "jquery/jquery-3.1.1.min.js"
);
// encode_bench_webkit!(bench_webkit_from_utf16_jquery_windows_1252,
//                     WINDOWS_1252,
//                     "jquery/jquery-3.1.1.min.js");
encode_bench_windows!(
    bench_windows_from_utf16_jquery_windows_1252,
    WINDOWS_1252,
    1252,
    "jquery/jquery-3.1.1.min.js"
);

decode_bench_user_defined!(
    bench_decode_to_utf8_user_defined,
    "wikipedia/binary.jpg",
    max_utf8_buffer_length,
    decode_to_utf8
);
decode_bench_user_defined!(
    bench_decode_to_utf16_user_defined,
    "wikipedia/binary.jpg",
    max_utf16_buffer_length,
    decode_to_utf16
);

decode_bench_legacy!(
    bench_decode_to_utf8_ja_euc_jp,
    bench_decode_to_utf16_ja_euc_jp,
    bench_decode_to_string_ja_euc_jp,
    bench_rust_to_string_ja_euc_jp,
    bench_iconv_to_utf8_ja_euc_jp,
    bench_icu_to_utf16_ja_euc_jp,
    bench_uconv_to_utf16_ja_euc_jp,
    bench_windows_to_utf16_ja_euc_jp,
    EUC_JP,
    20932,
    "wikipedia/ja.html"
);
encode_bench_legacy!(
    bench_encode_from_utf8_ja_euc_jp,
    bench_encode_from_utf16_ja_euc_jp,
    bench_encode_to_vec_ja_euc_jp,
    bench_rust_to_vec_ja_euc_jp,
    bench_iconv_from_utf8_ja_euc_jp,
    bench_icu_from_utf16_ja_euc_jp,
    bench_uconv_from_utf16_ja_euc_jp,
    bench_windows_from_utf16_ja_euc_jp,
    EUC_JP,
    20932,
    "wikipedia/ja.txt"
);

decode_bench_legacy!(
    bench_decode_to_utf8_ja_iso_2022_jp,
    bench_decode_to_utf16_ja_iso_2022_jp,
    bench_decode_to_string_ja_iso_2022_jp,
    bench_rust_to_string_ja_iso_2022_jp,
    bench_iconv_to_utf8_ja_iso_2022_jp,
    bench_icu_to_utf16_ja_iso_2022_jp,
    bench_uconv_to_utf16_ja_iso_2022_jp,
    bench_windows_to_utf16_ja_iso_2022_jp,
    ISO_2022_JP,
    50220,
    "wikipedia/ja.html"
);
encode_bench_legacy!(
    bench_encode_from_utf8_ja_iso_2022_jp,
    bench_encode_from_utf16_ja_iso_2022_jp,
    bench_encode_to_vec_ja_iso_2022_jp,
    bench_rust_to_vec_ja_iso_2022_jp,
    bench_iconv_from_utf8_ja_iso_2022_jp,
    bench_icu_from_utf16_ja_iso_2022_jp,
    bench_uconv_from_utf16_ja_iso_2022_jp,
    bench_windows_from_utf16_ja_iso_2022_jp,
    ISO_2022_JP,
    50220,
    "wikipedia/ja.txt"
);

mem_bench_is_u8!(
    bench_mem_is_utf8_bidi_de_1000,
    bench_safe_mem_is_utf8_bidi_de_1000,
    is_utf8_bidi,
    1000,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u8!(
    bench_mem_is_utf8_bidi_ru_1000,
    bench_safe_mem_is_utf8_bidi_ru_1000,
    is_utf8_bidi,
    1000,
    "wikipedia/ru.txt"
);

mem_bench_is_u8!(
    bench_mem_is_utf8_bidi_ja_1000,
    bench_safe_mem_is_utf8_bidi_ja_1000,
    is_utf8_bidi,
    1000,
    "wikipedia/ja.txt"
);

mem_bench_is_u8!(
    bench_mem_is_utf8_bidi_th_1000,
    bench_safe_mem_is_utf8_bidi_th_1000,
    is_utf8_bidi,
    1000,
    "wikipedia/th.txt"
);

mem_bench_is_str!(
    bench_mem_is_str_bidi_de_1000,
    bench_safe_mem_is_str_bidi_de_1000,
    is_str_bidi,
    1000,
    "wikipedia/de-edit.txt"
);

mem_bench_is_str!(
    bench_mem_is_str_bidi_ru_1000,
    bench_safe_mem_is_str_bidi_ru_1000,
    is_str_bidi,
    1000,
    "wikipedia/ru.txt"
);

mem_bench_is_str!(
    bench_mem_is_str_bidi_ja_1000,
    bench_safe_mem_is_str_bidi_ja_1000,
    is_str_bidi,
    1000,
    "wikipedia/ja.txt"
);

mem_bench_is_str!(
    bench_mem_is_str_bidi_th_1000,
    bench_safe_mem_is_str_bidi_th_1000,
    is_str_bidi,
    1000,
    "wikipedia/th.txt"
);

mem_bench_is_u16!(
    bench_mem_is_utf16_bidi_de_1000,
    bench_safe_mem_is_utf16_bidi_de_1000,
    is_utf16_bidi,
    1000,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u16!(
    bench_mem_is_utf16_bidi_ru_1000,
    bench_safe_mem_is_utf16_bidi_ru_1000,
    is_utf16_bidi,
    1000,
    "wikipedia/ru.txt"
);

mem_bench_is_u16!(
    bench_mem_is_utf16_bidi_ja_1000,
    bench_safe_mem_is_utf16_bidi_ja_1000,
    is_utf16_bidi,
    1000,
    "wikipedia/ja.txt"
);

mem_bench_is_u16!(
    bench_mem_is_utf16_bidi_th_1000,
    bench_safe_mem_is_utf16_bidi_th_1000,
    is_utf16_bidi,
    1000,
    "wikipedia/th.txt"
);

mem_bench_is_u16!(
    bench_mem_check_utf16_for_latin1_and_bidi_de_1000,
    bench_safe_mem_check_utf16_for_latin1_and_bidi_de_1000,
    check_utf16_for_latin1_and_bidi,
    1000,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u16!(
    bench_mem_check_utf16_for_latin1_and_bidi_ru_1000,
    bench_safe_mem_check_utf16_for_latin1_and_bidi_ru_1000,
    check_utf16_for_latin1_and_bidi,
    1000,
    "wikipedia/ru.txt"
);

mem_bench_is_u16!(
    bench_mem_check_utf16_for_latin1_and_bidi_ja_1000,
    bench_safe_mem_check_utf16_for_latin1_and_bidi_ja_1000,
    is_utf16_bidi,
    1000,
    "wikipedia/ja.txt"
);

mem_bench_is_u16!(
    bench_mem_check_utf16_for_latin1_and_bidi_th_1000,
    bench_safe_mem_check_utf16_for_latin1_and_bidi_th_1000,
    is_utf16_bidi,
    1000,
    "wikipedia/th.txt"
);

// BEGIN GENERATED CODE. PLEASE DO NOT EDIT.
// Instead, please regenerate using generate-encoding-data.py

decode_bench!(
    bench_copy_ar,
    bench_decode_to_utf8_ar,
    bench_decode_to_utf16_ar,
    bench_decode_to_string_ar,
    bench_rust_to_string_ar,
    bench_std_validation_ar,
    bench_kewb_to_utf16_ar,
    bench_iconv_to_utf8_ar,
    bench_icu_to_utf16_ar,
    bench_uconv_to_utf16_ar,
    bench_webkit_to_utf16_ar,
    bench_windows_to_utf16_ar,
    bench_decode_to_utf8_ar_windows_1256,
    bench_decode_to_utf16_ar_windows_1256,
    bench_decode_to_string_ar_windows_1256,
    bench_rust_to_string_ar_windows_1256,
    bench_iconv_to_utf8_ar_windows_1256,
    bench_icu_to_utf16_ar_windows_1256,
    bench_uconv_to_utf16_ar_windows_1256,
    bench_windows_to_utf16_ar_windows_1256,
    WINDOWS_1256,
    1256,
    "wikipedia/ar.html"
);
encode_bench!(
    bench_encode_from_utf8_ar,
    bench_encode_from_utf16_ar,
    bench_encode_to_vec_ar,
    bench_rust_to_vec_ar,
    bench_iconv_from_utf8_ar,
    bench_icu_from_utf16_ar,
    bench_uconv_from_utf16_ar,
    bench_webkit_from_utf16_ar,
    bench_windows_from_utf16_ar,
    bench_encode_from_utf8_ar_windows_1256,
    bench_encode_from_utf16_ar_windows_1256,
    bench_encode_to_vec_ar_windows_1256,
    bench_rust_to_vec_ar_windows_1256,
    bench_iconv_from_utf8_ar_windows_1256,
    bench_icu_from_utf16_ar_windows_1256,
    bench_uconv_from_utf16_ar_windows_1256,
    bench_windows_from_utf16_ar_windows_1256,
    WINDOWS_1256,
    1256,
    "wikipedia/ar.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_ar_utf_16le,
    bench_decode_to_utf16_ar_utf_16le,
    bench_decode_to_string_ar_utf_16le,
    bench_rust_to_string_ar_utf_16le,
    bench_iconv_to_utf8_ar_utf_16le,
    bench_icu_to_utf16_ar_utf_16le,
    bench_uconv_to_utf16_ar_utf_16le,
    bench_windows_to_utf16_ar_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/ar.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_ar_utf_16be,
    bench_decode_to_utf16_ar_utf_16be,
    bench_decode_to_string_ar_utf_16be,
    bench_rust_to_string_ar_utf_16be,
    bench_iconv_to_utf8_ar_utf_16be,
    bench_icu_to_utf16_ar_utf_16be,
    bench_uconv_to_utf16_ar_utf_16be,
    bench_windows_to_utf16_ar_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/ar.html"
);
decode_bench!(
    bench_copy_cs,
    bench_decode_to_utf8_cs,
    bench_decode_to_utf16_cs,
    bench_decode_to_string_cs,
    bench_rust_to_string_cs,
    bench_std_validation_cs,
    bench_kewb_to_utf16_cs,
    bench_iconv_to_utf8_cs,
    bench_icu_to_utf16_cs,
    bench_uconv_to_utf16_cs,
    bench_webkit_to_utf16_cs,
    bench_windows_to_utf16_cs,
    bench_decode_to_utf8_cs_windows_1250,
    bench_decode_to_utf16_cs_windows_1250,
    bench_decode_to_string_cs_windows_1250,
    bench_rust_to_string_cs_windows_1250,
    bench_iconv_to_utf8_cs_windows_1250,
    bench_icu_to_utf16_cs_windows_1250,
    bench_uconv_to_utf16_cs_windows_1250,
    bench_windows_to_utf16_cs_windows_1250,
    WINDOWS_1250,
    1250,
    "wikipedia/cs.html"
);
encode_bench!(
    bench_encode_from_utf8_cs,
    bench_encode_from_utf16_cs,
    bench_encode_to_vec_cs,
    bench_rust_to_vec_cs,
    bench_iconv_from_utf8_cs,
    bench_icu_from_utf16_cs,
    bench_uconv_from_utf16_cs,
    bench_webkit_from_utf16_cs,
    bench_windows_from_utf16_cs,
    bench_encode_from_utf8_cs_windows_1250,
    bench_encode_from_utf16_cs_windows_1250,
    bench_encode_to_vec_cs_windows_1250,
    bench_rust_to_vec_cs_windows_1250,
    bench_iconv_from_utf8_cs_windows_1250,
    bench_icu_from_utf16_cs_windows_1250,
    bench_uconv_from_utf16_cs_windows_1250,
    bench_windows_from_utf16_cs_windows_1250,
    WINDOWS_1250,
    1250,
    "wikipedia/cs.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_cs_utf_16le,
    bench_decode_to_utf16_cs_utf_16le,
    bench_decode_to_string_cs_utf_16le,
    bench_rust_to_string_cs_utf_16le,
    bench_iconv_to_utf8_cs_utf_16le,
    bench_icu_to_utf16_cs_utf_16le,
    bench_uconv_to_utf16_cs_utf_16le,
    bench_windows_to_utf16_cs_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/cs.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_cs_utf_16be,
    bench_decode_to_utf16_cs_utf_16be,
    bench_decode_to_string_cs_utf_16be,
    bench_rust_to_string_cs_utf_16be,
    bench_iconv_to_utf8_cs_utf_16be,
    bench_icu_to_utf16_cs_utf_16be,
    bench_uconv_to_utf16_cs_utf_16be,
    bench_windows_to_utf16_cs_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/cs.html"
);
decode_bench!(
    bench_copy_de,
    bench_decode_to_utf8_de,
    bench_decode_to_utf16_de,
    bench_decode_to_string_de,
    bench_rust_to_string_de,
    bench_std_validation_de,
    bench_kewb_to_utf16_de,
    bench_iconv_to_utf8_de,
    bench_icu_to_utf16_de,
    bench_uconv_to_utf16_de,
    bench_webkit_to_utf16_de,
    bench_windows_to_utf16_de,
    bench_decode_to_utf8_de_windows_1252,
    bench_decode_to_utf16_de_windows_1252,
    bench_decode_to_string_de_windows_1252,
    bench_rust_to_string_de_windows_1252,
    bench_iconv_to_utf8_de_windows_1252,
    bench_icu_to_utf16_de_windows_1252,
    bench_uconv_to_utf16_de_windows_1252,
    bench_windows_to_utf16_de_windows_1252,
    WINDOWS_1252,
    1252,
    "wikipedia/de.html"
);
encode_bench!(
    bench_encode_from_utf8_de,
    bench_encode_from_utf16_de,
    bench_encode_to_vec_de,
    bench_rust_to_vec_de,
    bench_iconv_from_utf8_de,
    bench_icu_from_utf16_de,
    bench_uconv_from_utf16_de,
    bench_webkit_from_utf16_de,
    bench_windows_from_utf16_de,
    bench_encode_from_utf8_de_windows_1252,
    bench_encode_from_utf16_de_windows_1252,
    bench_encode_to_vec_de_windows_1252,
    bench_rust_to_vec_de_windows_1252,
    bench_iconv_from_utf8_de_windows_1252,
    bench_icu_from_utf16_de_windows_1252,
    bench_uconv_from_utf16_de_windows_1252,
    bench_windows_from_utf16_de_windows_1252,
    WINDOWS_1252,
    1252,
    "wikipedia/de.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_de_utf_16le,
    bench_decode_to_utf16_de_utf_16le,
    bench_decode_to_string_de_utf_16le,
    bench_rust_to_string_de_utf_16le,
    bench_iconv_to_utf8_de_utf_16le,
    bench_icu_to_utf16_de_utf_16le,
    bench_uconv_to_utf16_de_utf_16le,
    bench_windows_to_utf16_de_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/de.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_de_utf_16be,
    bench_decode_to_utf16_de_utf_16be,
    bench_decode_to_string_de_utf_16be,
    bench_rust_to_string_de_utf_16be,
    bench_iconv_to_utf8_de_utf_16be,
    bench_icu_to_utf16_de_utf_16be,
    bench_uconv_to_utf16_de_utf_16be,
    bench_windows_to_utf16_de_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/de.html"
);
decode_bench!(
    bench_copy_el,
    bench_decode_to_utf8_el,
    bench_decode_to_utf16_el,
    bench_decode_to_string_el,
    bench_rust_to_string_el,
    bench_std_validation_el,
    bench_kewb_to_utf16_el,
    bench_iconv_to_utf8_el,
    bench_icu_to_utf16_el,
    bench_uconv_to_utf16_el,
    bench_webkit_to_utf16_el,
    bench_windows_to_utf16_el,
    bench_decode_to_utf8_el_windows_1253,
    bench_decode_to_utf16_el_windows_1253,
    bench_decode_to_string_el_windows_1253,
    bench_rust_to_string_el_windows_1253,
    bench_iconv_to_utf8_el_windows_1253,
    bench_icu_to_utf16_el_windows_1253,
    bench_uconv_to_utf16_el_windows_1253,
    bench_windows_to_utf16_el_windows_1253,
    WINDOWS_1253,
    1253,
    "wikipedia/el.html"
);
encode_bench!(
    bench_encode_from_utf8_el,
    bench_encode_from_utf16_el,
    bench_encode_to_vec_el,
    bench_rust_to_vec_el,
    bench_iconv_from_utf8_el,
    bench_icu_from_utf16_el,
    bench_uconv_from_utf16_el,
    bench_webkit_from_utf16_el,
    bench_windows_from_utf16_el,
    bench_encode_from_utf8_el_windows_1253,
    bench_encode_from_utf16_el_windows_1253,
    bench_encode_to_vec_el_windows_1253,
    bench_rust_to_vec_el_windows_1253,
    bench_iconv_from_utf8_el_windows_1253,
    bench_icu_from_utf16_el_windows_1253,
    bench_uconv_from_utf16_el_windows_1253,
    bench_windows_from_utf16_el_windows_1253,
    WINDOWS_1253,
    1253,
    "wikipedia/el.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_el_utf_16le,
    bench_decode_to_utf16_el_utf_16le,
    bench_decode_to_string_el_utf_16le,
    bench_rust_to_string_el_utf_16le,
    bench_iconv_to_utf8_el_utf_16le,
    bench_icu_to_utf16_el_utf_16le,
    bench_uconv_to_utf16_el_utf_16le,
    bench_windows_to_utf16_el_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/el.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_el_utf_16be,
    bench_decode_to_utf16_el_utf_16be,
    bench_decode_to_string_el_utf_16be,
    bench_rust_to_string_el_utf_16be,
    bench_iconv_to_utf8_el_utf_16be,
    bench_icu_to_utf16_el_utf_16be,
    bench_uconv_to_utf16_el_utf_16be,
    bench_windows_to_utf16_el_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/el.html"
);
decode_bench!(
    bench_copy_en,
    bench_decode_to_utf8_en,
    bench_decode_to_utf16_en,
    bench_decode_to_string_en,
    bench_rust_to_string_en,
    bench_std_validation_en,
    bench_kewb_to_utf16_en,
    bench_iconv_to_utf8_en,
    bench_icu_to_utf16_en,
    bench_uconv_to_utf16_en,
    bench_webkit_to_utf16_en,
    bench_windows_to_utf16_en,
    bench_decode_to_utf8_en_windows_1252,
    bench_decode_to_utf16_en_windows_1252,
    bench_decode_to_string_en_windows_1252,
    bench_rust_to_string_en_windows_1252,
    bench_iconv_to_utf8_en_windows_1252,
    bench_icu_to_utf16_en_windows_1252,
    bench_uconv_to_utf16_en_windows_1252,
    bench_windows_to_utf16_en_windows_1252,
    WINDOWS_1252,
    1252,
    "wikipedia/en.html"
);
encode_bench!(
    bench_encode_from_utf8_en,
    bench_encode_from_utf16_en,
    bench_encode_to_vec_en,
    bench_rust_to_vec_en,
    bench_iconv_from_utf8_en,
    bench_icu_from_utf16_en,
    bench_uconv_from_utf16_en,
    bench_webkit_from_utf16_en,
    bench_windows_from_utf16_en,
    bench_encode_from_utf8_en_windows_1252,
    bench_encode_from_utf16_en_windows_1252,
    bench_encode_to_vec_en_windows_1252,
    bench_rust_to_vec_en_windows_1252,
    bench_iconv_from_utf8_en_windows_1252,
    bench_icu_from_utf16_en_windows_1252,
    bench_uconv_from_utf16_en_windows_1252,
    bench_windows_from_utf16_en_windows_1252,
    WINDOWS_1252,
    1252,
    "wikipedia/en.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_en_utf_16le,
    bench_decode_to_utf16_en_utf_16le,
    bench_decode_to_string_en_utf_16le,
    bench_rust_to_string_en_utf_16le,
    bench_iconv_to_utf8_en_utf_16le,
    bench_icu_to_utf16_en_utf_16le,
    bench_uconv_to_utf16_en_utf_16le,
    bench_windows_to_utf16_en_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/en.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_en_utf_16be,
    bench_decode_to_utf16_en_utf_16be,
    bench_decode_to_string_en_utf_16be,
    bench_rust_to_string_en_utf_16be,
    bench_iconv_to_utf8_en_utf_16be,
    bench_icu_to_utf16_en_utf_16be,
    bench_uconv_to_utf16_en_utf_16be,
    bench_windows_to_utf16_en_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/en.html"
);
decode_bench!(
    bench_copy_fr,
    bench_decode_to_utf8_fr,
    bench_decode_to_utf16_fr,
    bench_decode_to_string_fr,
    bench_rust_to_string_fr,
    bench_std_validation_fr,
    bench_kewb_to_utf16_fr,
    bench_iconv_to_utf8_fr,
    bench_icu_to_utf16_fr,
    bench_uconv_to_utf16_fr,
    bench_webkit_to_utf16_fr,
    bench_windows_to_utf16_fr,
    bench_decode_to_utf8_fr_windows_1252,
    bench_decode_to_utf16_fr_windows_1252,
    bench_decode_to_string_fr_windows_1252,
    bench_rust_to_string_fr_windows_1252,
    bench_iconv_to_utf8_fr_windows_1252,
    bench_icu_to_utf16_fr_windows_1252,
    bench_uconv_to_utf16_fr_windows_1252,
    bench_windows_to_utf16_fr_windows_1252,
    WINDOWS_1252,
    1252,
    "wikipedia/fr.html"
);
encode_bench!(
    bench_encode_from_utf8_fr,
    bench_encode_from_utf16_fr,
    bench_encode_to_vec_fr,
    bench_rust_to_vec_fr,
    bench_iconv_from_utf8_fr,
    bench_icu_from_utf16_fr,
    bench_uconv_from_utf16_fr,
    bench_webkit_from_utf16_fr,
    bench_windows_from_utf16_fr,
    bench_encode_from_utf8_fr_windows_1252,
    bench_encode_from_utf16_fr_windows_1252,
    bench_encode_to_vec_fr_windows_1252,
    bench_rust_to_vec_fr_windows_1252,
    bench_iconv_from_utf8_fr_windows_1252,
    bench_icu_from_utf16_fr_windows_1252,
    bench_uconv_from_utf16_fr_windows_1252,
    bench_windows_from_utf16_fr_windows_1252,
    WINDOWS_1252,
    1252,
    "wikipedia/fr.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_fr_utf_16le,
    bench_decode_to_utf16_fr_utf_16le,
    bench_decode_to_string_fr_utf_16le,
    bench_rust_to_string_fr_utf_16le,
    bench_iconv_to_utf8_fr_utf_16le,
    bench_icu_to_utf16_fr_utf_16le,
    bench_uconv_to_utf16_fr_utf_16le,
    bench_windows_to_utf16_fr_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/fr.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_fr_utf_16be,
    bench_decode_to_utf16_fr_utf_16be,
    bench_decode_to_string_fr_utf_16be,
    bench_rust_to_string_fr_utf_16be,
    bench_iconv_to_utf8_fr_utf_16be,
    bench_icu_to_utf16_fr_utf_16be,
    bench_uconv_to_utf16_fr_utf_16be,
    bench_windows_to_utf16_fr_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/fr.html"
);
decode_bench!(
    bench_copy_he,
    bench_decode_to_utf8_he,
    bench_decode_to_utf16_he,
    bench_decode_to_string_he,
    bench_rust_to_string_he,
    bench_std_validation_he,
    bench_kewb_to_utf16_he,
    bench_iconv_to_utf8_he,
    bench_icu_to_utf16_he,
    bench_uconv_to_utf16_he,
    bench_webkit_to_utf16_he,
    bench_windows_to_utf16_he,
    bench_decode_to_utf8_he_windows_1255,
    bench_decode_to_utf16_he_windows_1255,
    bench_decode_to_string_he_windows_1255,
    bench_rust_to_string_he_windows_1255,
    bench_iconv_to_utf8_he_windows_1255,
    bench_icu_to_utf16_he_windows_1255,
    bench_uconv_to_utf16_he_windows_1255,
    bench_windows_to_utf16_he_windows_1255,
    WINDOWS_1255,
    1255,
    "wikipedia/he.html"
);
encode_bench!(
    bench_encode_from_utf8_he,
    bench_encode_from_utf16_he,
    bench_encode_to_vec_he,
    bench_rust_to_vec_he,
    bench_iconv_from_utf8_he,
    bench_icu_from_utf16_he,
    bench_uconv_from_utf16_he,
    bench_webkit_from_utf16_he,
    bench_windows_from_utf16_he,
    bench_encode_from_utf8_he_windows_1255,
    bench_encode_from_utf16_he_windows_1255,
    bench_encode_to_vec_he_windows_1255,
    bench_rust_to_vec_he_windows_1255,
    bench_iconv_from_utf8_he_windows_1255,
    bench_icu_from_utf16_he_windows_1255,
    bench_uconv_from_utf16_he_windows_1255,
    bench_windows_from_utf16_he_windows_1255,
    WINDOWS_1255,
    1255,
    "wikipedia/he.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_he_utf_16le,
    bench_decode_to_utf16_he_utf_16le,
    bench_decode_to_string_he_utf_16le,
    bench_rust_to_string_he_utf_16le,
    bench_iconv_to_utf8_he_utf_16le,
    bench_icu_to_utf16_he_utf_16le,
    bench_uconv_to_utf16_he_utf_16le,
    bench_windows_to_utf16_he_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/he.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_he_utf_16be,
    bench_decode_to_utf16_he_utf_16be,
    bench_decode_to_string_he_utf_16be,
    bench_rust_to_string_he_utf_16be,
    bench_iconv_to_utf8_he_utf_16be,
    bench_icu_to_utf16_he_utf_16be,
    bench_uconv_to_utf16_he_utf_16be,
    bench_windows_to_utf16_he_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/he.html"
);
decode_bench!(
    bench_copy_ja,
    bench_decode_to_utf8_ja,
    bench_decode_to_utf16_ja,
    bench_decode_to_string_ja,
    bench_rust_to_string_ja,
    bench_std_validation_ja,
    bench_kewb_to_utf16_ja,
    bench_iconv_to_utf8_ja,
    bench_icu_to_utf16_ja,
    bench_uconv_to_utf16_ja,
    bench_webkit_to_utf16_ja,
    bench_windows_to_utf16_ja,
    bench_decode_to_utf8_ja_shift_jis,
    bench_decode_to_utf16_ja_shift_jis,
    bench_decode_to_string_ja_shift_jis,
    bench_rust_to_string_ja_shift_jis,
    bench_iconv_to_utf8_ja_shift_jis,
    bench_icu_to_utf16_ja_shift_jis,
    bench_uconv_to_utf16_ja_shift_jis,
    bench_windows_to_utf16_ja_shift_jis,
    SHIFT_JIS,
    932,
    "wikipedia/ja.html"
);
encode_bench!(
    bench_encode_from_utf8_ja,
    bench_encode_from_utf16_ja,
    bench_encode_to_vec_ja,
    bench_rust_to_vec_ja,
    bench_iconv_from_utf8_ja,
    bench_icu_from_utf16_ja,
    bench_uconv_from_utf16_ja,
    bench_webkit_from_utf16_ja,
    bench_windows_from_utf16_ja,
    bench_encode_from_utf8_ja_shift_jis,
    bench_encode_from_utf16_ja_shift_jis,
    bench_encode_to_vec_ja_shift_jis,
    bench_rust_to_vec_ja_shift_jis,
    bench_iconv_from_utf8_ja_shift_jis,
    bench_icu_from_utf16_ja_shift_jis,
    bench_uconv_from_utf16_ja_shift_jis,
    bench_windows_from_utf16_ja_shift_jis,
    SHIFT_JIS,
    932,
    "wikipedia/ja.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_ja_utf_16le,
    bench_decode_to_utf16_ja_utf_16le,
    bench_decode_to_string_ja_utf_16le,
    bench_rust_to_string_ja_utf_16le,
    bench_iconv_to_utf8_ja_utf_16le,
    bench_icu_to_utf16_ja_utf_16le,
    bench_uconv_to_utf16_ja_utf_16le,
    bench_windows_to_utf16_ja_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/ja.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_ja_utf_16be,
    bench_decode_to_utf16_ja_utf_16be,
    bench_decode_to_string_ja_utf_16be,
    bench_rust_to_string_ja_utf_16be,
    bench_iconv_to_utf8_ja_utf_16be,
    bench_icu_to_utf16_ja_utf_16be,
    bench_uconv_to_utf16_ja_utf_16be,
    bench_windows_to_utf16_ja_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/ja.html"
);
decode_bench!(
    bench_copy_ko,
    bench_decode_to_utf8_ko,
    bench_decode_to_utf16_ko,
    bench_decode_to_string_ko,
    bench_rust_to_string_ko,
    bench_std_validation_ko,
    bench_kewb_to_utf16_ko,
    bench_iconv_to_utf8_ko,
    bench_icu_to_utf16_ko,
    bench_uconv_to_utf16_ko,
    bench_webkit_to_utf16_ko,
    bench_windows_to_utf16_ko,
    bench_decode_to_utf8_ko_euc_kr,
    bench_decode_to_utf16_ko_euc_kr,
    bench_decode_to_string_ko_euc_kr,
    bench_rust_to_string_ko_euc_kr,
    bench_iconv_to_utf8_ko_euc_kr,
    bench_icu_to_utf16_ko_euc_kr,
    bench_uconv_to_utf16_ko_euc_kr,
    bench_windows_to_utf16_ko_euc_kr,
    EUC_KR,
    949,
    "wikipedia/ko.html"
);
encode_bench!(
    bench_encode_from_utf8_ko,
    bench_encode_from_utf16_ko,
    bench_encode_to_vec_ko,
    bench_rust_to_vec_ko,
    bench_iconv_from_utf8_ko,
    bench_icu_from_utf16_ko,
    bench_uconv_from_utf16_ko,
    bench_webkit_from_utf16_ko,
    bench_windows_from_utf16_ko,
    bench_encode_from_utf8_ko_euc_kr,
    bench_encode_from_utf16_ko_euc_kr,
    bench_encode_to_vec_ko_euc_kr,
    bench_rust_to_vec_ko_euc_kr,
    bench_iconv_from_utf8_ko_euc_kr,
    bench_icu_from_utf16_ko_euc_kr,
    bench_uconv_from_utf16_ko_euc_kr,
    bench_windows_from_utf16_ko_euc_kr,
    EUC_KR,
    949,
    "wikipedia/ko.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_ko_utf_16le,
    bench_decode_to_utf16_ko_utf_16le,
    bench_decode_to_string_ko_utf_16le,
    bench_rust_to_string_ko_utf_16le,
    bench_iconv_to_utf8_ko_utf_16le,
    bench_icu_to_utf16_ko_utf_16le,
    bench_uconv_to_utf16_ko_utf_16le,
    bench_windows_to_utf16_ko_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/ko.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_ko_utf_16be,
    bench_decode_to_utf16_ko_utf_16be,
    bench_decode_to_string_ko_utf_16be,
    bench_rust_to_string_ko_utf_16be,
    bench_iconv_to_utf8_ko_utf_16be,
    bench_icu_to_utf16_ko_utf_16be,
    bench_uconv_to_utf16_ko_utf_16be,
    bench_windows_to_utf16_ko_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/ko.html"
);
decode_bench!(
    bench_copy_pt,
    bench_decode_to_utf8_pt,
    bench_decode_to_utf16_pt,
    bench_decode_to_string_pt,
    bench_rust_to_string_pt,
    bench_std_validation_pt,
    bench_kewb_to_utf16_pt,
    bench_iconv_to_utf8_pt,
    bench_icu_to_utf16_pt,
    bench_uconv_to_utf16_pt,
    bench_webkit_to_utf16_pt,
    bench_windows_to_utf16_pt,
    bench_decode_to_utf8_pt_windows_1252,
    bench_decode_to_utf16_pt_windows_1252,
    bench_decode_to_string_pt_windows_1252,
    bench_rust_to_string_pt_windows_1252,
    bench_iconv_to_utf8_pt_windows_1252,
    bench_icu_to_utf16_pt_windows_1252,
    bench_uconv_to_utf16_pt_windows_1252,
    bench_windows_to_utf16_pt_windows_1252,
    WINDOWS_1252,
    1252,
    "wikipedia/pt.html"
);
encode_bench!(
    bench_encode_from_utf8_pt,
    bench_encode_from_utf16_pt,
    bench_encode_to_vec_pt,
    bench_rust_to_vec_pt,
    bench_iconv_from_utf8_pt,
    bench_icu_from_utf16_pt,
    bench_uconv_from_utf16_pt,
    bench_webkit_from_utf16_pt,
    bench_windows_from_utf16_pt,
    bench_encode_from_utf8_pt_windows_1252,
    bench_encode_from_utf16_pt_windows_1252,
    bench_encode_to_vec_pt_windows_1252,
    bench_rust_to_vec_pt_windows_1252,
    bench_iconv_from_utf8_pt_windows_1252,
    bench_icu_from_utf16_pt_windows_1252,
    bench_uconv_from_utf16_pt_windows_1252,
    bench_windows_from_utf16_pt_windows_1252,
    WINDOWS_1252,
    1252,
    "wikipedia/pt.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_pt_utf_16le,
    bench_decode_to_utf16_pt_utf_16le,
    bench_decode_to_string_pt_utf_16le,
    bench_rust_to_string_pt_utf_16le,
    bench_iconv_to_utf8_pt_utf_16le,
    bench_icu_to_utf16_pt_utf_16le,
    bench_uconv_to_utf16_pt_utf_16le,
    bench_windows_to_utf16_pt_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/pt.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_pt_utf_16be,
    bench_decode_to_utf16_pt_utf_16be,
    bench_decode_to_string_pt_utf_16be,
    bench_rust_to_string_pt_utf_16be,
    bench_iconv_to_utf8_pt_utf_16be,
    bench_icu_to_utf16_pt_utf_16be,
    bench_uconv_to_utf16_pt_utf_16be,
    bench_windows_to_utf16_pt_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/pt.html"
);
decode_bench!(
    bench_copy_ru,
    bench_decode_to_utf8_ru,
    bench_decode_to_utf16_ru,
    bench_decode_to_string_ru,
    bench_rust_to_string_ru,
    bench_std_validation_ru,
    bench_kewb_to_utf16_ru,
    bench_iconv_to_utf8_ru,
    bench_icu_to_utf16_ru,
    bench_uconv_to_utf16_ru,
    bench_webkit_to_utf16_ru,
    bench_windows_to_utf16_ru,
    bench_decode_to_utf8_ru_windows_1251,
    bench_decode_to_utf16_ru_windows_1251,
    bench_decode_to_string_ru_windows_1251,
    bench_rust_to_string_ru_windows_1251,
    bench_iconv_to_utf8_ru_windows_1251,
    bench_icu_to_utf16_ru_windows_1251,
    bench_uconv_to_utf16_ru_windows_1251,
    bench_windows_to_utf16_ru_windows_1251,
    WINDOWS_1251,
    1251,
    "wikipedia/ru.html"
);
encode_bench!(
    bench_encode_from_utf8_ru,
    bench_encode_from_utf16_ru,
    bench_encode_to_vec_ru,
    bench_rust_to_vec_ru,
    bench_iconv_from_utf8_ru,
    bench_icu_from_utf16_ru,
    bench_uconv_from_utf16_ru,
    bench_webkit_from_utf16_ru,
    bench_windows_from_utf16_ru,
    bench_encode_from_utf8_ru_windows_1251,
    bench_encode_from_utf16_ru_windows_1251,
    bench_encode_to_vec_ru_windows_1251,
    bench_rust_to_vec_ru_windows_1251,
    bench_iconv_from_utf8_ru_windows_1251,
    bench_icu_from_utf16_ru_windows_1251,
    bench_uconv_from_utf16_ru_windows_1251,
    bench_windows_from_utf16_ru_windows_1251,
    WINDOWS_1251,
    1251,
    "wikipedia/ru.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_ru_utf_16le,
    bench_decode_to_utf16_ru_utf_16le,
    bench_decode_to_string_ru_utf_16le,
    bench_rust_to_string_ru_utf_16le,
    bench_iconv_to_utf8_ru_utf_16le,
    bench_icu_to_utf16_ru_utf_16le,
    bench_uconv_to_utf16_ru_utf_16le,
    bench_windows_to_utf16_ru_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/ru.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_ru_utf_16be,
    bench_decode_to_utf16_ru_utf_16be,
    bench_decode_to_string_ru_utf_16be,
    bench_rust_to_string_ru_utf_16be,
    bench_iconv_to_utf8_ru_utf_16be,
    bench_icu_to_utf16_ru_utf_16be,
    bench_uconv_to_utf16_ru_utf_16be,
    bench_windows_to_utf16_ru_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/ru.html"
);
decode_bench!(
    bench_copy_th,
    bench_decode_to_utf8_th,
    bench_decode_to_utf16_th,
    bench_decode_to_string_th,
    bench_rust_to_string_th,
    bench_std_validation_th,
    bench_kewb_to_utf16_th,
    bench_iconv_to_utf8_th,
    bench_icu_to_utf16_th,
    bench_uconv_to_utf16_th,
    bench_webkit_to_utf16_th,
    bench_windows_to_utf16_th,
    bench_decode_to_utf8_th_windows_874,
    bench_decode_to_utf16_th_windows_874,
    bench_decode_to_string_th_windows_874,
    bench_rust_to_string_th_windows_874,
    bench_iconv_to_utf8_th_windows_874,
    bench_icu_to_utf16_th_windows_874,
    bench_uconv_to_utf16_th_windows_874,
    bench_windows_to_utf16_th_windows_874,
    WINDOWS_874,
    874,
    "wikipedia/th.html"
);
encode_bench!(
    bench_encode_from_utf8_th,
    bench_encode_from_utf16_th,
    bench_encode_to_vec_th,
    bench_rust_to_vec_th,
    bench_iconv_from_utf8_th,
    bench_icu_from_utf16_th,
    bench_uconv_from_utf16_th,
    bench_webkit_from_utf16_th,
    bench_windows_from_utf16_th,
    bench_encode_from_utf8_th_windows_874,
    bench_encode_from_utf16_th_windows_874,
    bench_encode_to_vec_th_windows_874,
    bench_rust_to_vec_th_windows_874,
    bench_iconv_from_utf8_th_windows_874,
    bench_icu_from_utf16_th_windows_874,
    bench_uconv_from_utf16_th_windows_874,
    bench_windows_from_utf16_th_windows_874,
    WINDOWS_874,
    874,
    "wikipedia/th.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_th_utf_16le,
    bench_decode_to_utf16_th_utf_16le,
    bench_decode_to_string_th_utf_16le,
    bench_rust_to_string_th_utf_16le,
    bench_iconv_to_utf8_th_utf_16le,
    bench_icu_to_utf16_th_utf_16le,
    bench_uconv_to_utf16_th_utf_16le,
    bench_windows_to_utf16_th_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/th.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_th_utf_16be,
    bench_decode_to_utf16_th_utf_16be,
    bench_decode_to_string_th_utf_16be,
    bench_rust_to_string_th_utf_16be,
    bench_iconv_to_utf8_th_utf_16be,
    bench_icu_to_utf16_th_utf_16be,
    bench_uconv_to_utf16_th_utf_16be,
    bench_windows_to_utf16_th_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/th.html"
);
decode_bench!(
    bench_copy_tr,
    bench_decode_to_utf8_tr,
    bench_decode_to_utf16_tr,
    bench_decode_to_string_tr,
    bench_rust_to_string_tr,
    bench_std_validation_tr,
    bench_kewb_to_utf16_tr,
    bench_iconv_to_utf8_tr,
    bench_icu_to_utf16_tr,
    bench_uconv_to_utf16_tr,
    bench_webkit_to_utf16_tr,
    bench_windows_to_utf16_tr,
    bench_decode_to_utf8_tr_windows_1254,
    bench_decode_to_utf16_tr_windows_1254,
    bench_decode_to_string_tr_windows_1254,
    bench_rust_to_string_tr_windows_1254,
    bench_iconv_to_utf8_tr_windows_1254,
    bench_icu_to_utf16_tr_windows_1254,
    bench_uconv_to_utf16_tr_windows_1254,
    bench_windows_to_utf16_tr_windows_1254,
    WINDOWS_1254,
    1254,
    "wikipedia/tr.html"
);
encode_bench!(
    bench_encode_from_utf8_tr,
    bench_encode_from_utf16_tr,
    bench_encode_to_vec_tr,
    bench_rust_to_vec_tr,
    bench_iconv_from_utf8_tr,
    bench_icu_from_utf16_tr,
    bench_uconv_from_utf16_tr,
    bench_webkit_from_utf16_tr,
    bench_windows_from_utf16_tr,
    bench_encode_from_utf8_tr_windows_1254,
    bench_encode_from_utf16_tr_windows_1254,
    bench_encode_to_vec_tr_windows_1254,
    bench_rust_to_vec_tr_windows_1254,
    bench_iconv_from_utf8_tr_windows_1254,
    bench_icu_from_utf16_tr_windows_1254,
    bench_uconv_from_utf16_tr_windows_1254,
    bench_windows_from_utf16_tr_windows_1254,
    WINDOWS_1254,
    1254,
    "wikipedia/tr.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_tr_utf_16le,
    bench_decode_to_utf16_tr_utf_16le,
    bench_decode_to_string_tr_utf_16le,
    bench_rust_to_string_tr_utf_16le,
    bench_iconv_to_utf8_tr_utf_16le,
    bench_icu_to_utf16_tr_utf_16le,
    bench_uconv_to_utf16_tr_utf_16le,
    bench_windows_to_utf16_tr_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/tr.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_tr_utf_16be,
    bench_decode_to_utf16_tr_utf_16be,
    bench_decode_to_string_tr_utf_16be,
    bench_rust_to_string_tr_utf_16be,
    bench_iconv_to_utf8_tr_utf_16be,
    bench_icu_to_utf16_tr_utf_16be,
    bench_uconv_to_utf16_tr_utf_16be,
    bench_windows_to_utf16_tr_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/tr.html"
);
decode_bench!(
    bench_copy_vi,
    bench_decode_to_utf8_vi,
    bench_decode_to_utf16_vi,
    bench_decode_to_string_vi,
    bench_rust_to_string_vi,
    bench_std_validation_vi,
    bench_kewb_to_utf16_vi,
    bench_iconv_to_utf8_vi,
    bench_icu_to_utf16_vi,
    bench_uconv_to_utf16_vi,
    bench_webkit_to_utf16_vi,
    bench_windows_to_utf16_vi,
    bench_decode_to_utf8_vi_windows_1258,
    bench_decode_to_utf16_vi_windows_1258,
    bench_decode_to_string_vi_windows_1258,
    bench_rust_to_string_vi_windows_1258,
    bench_iconv_to_utf8_vi_windows_1258,
    bench_icu_to_utf16_vi_windows_1258,
    bench_uconv_to_utf16_vi_windows_1258,
    bench_windows_to_utf16_vi_windows_1258,
    WINDOWS_1258,
    1258,
    "wikipedia/vi.html"
);
encode_bench!(
    bench_encode_from_utf8_vi,
    bench_encode_from_utf16_vi,
    bench_encode_to_vec_vi,
    bench_rust_to_vec_vi,
    bench_iconv_from_utf8_vi,
    bench_icu_from_utf16_vi,
    bench_uconv_from_utf16_vi,
    bench_webkit_from_utf16_vi,
    bench_windows_from_utf16_vi,
    bench_encode_from_utf8_vi_windows_1258,
    bench_encode_from_utf16_vi_windows_1258,
    bench_encode_to_vec_vi_windows_1258,
    bench_rust_to_vec_vi_windows_1258,
    bench_iconv_from_utf8_vi_windows_1258,
    bench_icu_from_utf16_vi_windows_1258,
    bench_uconv_from_utf16_vi_windows_1258,
    bench_windows_from_utf16_vi_windows_1258,
    WINDOWS_1258,
    1258,
    "wikipedia/vi.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_vi_utf_16le,
    bench_decode_to_utf16_vi_utf_16le,
    bench_decode_to_string_vi_utf_16le,
    bench_rust_to_string_vi_utf_16le,
    bench_iconv_to_utf8_vi_utf_16le,
    bench_icu_to_utf16_vi_utf_16le,
    bench_uconv_to_utf16_vi_utf_16le,
    bench_windows_to_utf16_vi_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/vi.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_vi_utf_16be,
    bench_decode_to_utf16_vi_utf_16be,
    bench_decode_to_string_vi_utf_16be,
    bench_rust_to_string_vi_utf_16be,
    bench_iconv_to_utf8_vi_utf_16be,
    bench_icu_to_utf16_vi_utf_16be,
    bench_uconv_to_utf16_vi_utf_16be,
    bench_windows_to_utf16_vi_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/vi.html"
);
decode_bench!(
    bench_copy_zh_cn,
    bench_decode_to_utf8_zh_cn,
    bench_decode_to_utf16_zh_cn,
    bench_decode_to_string_zh_cn,
    bench_rust_to_string_zh_cn,
    bench_std_validation_zh_cn,
    bench_kewb_to_utf16_zh_cn,
    bench_iconv_to_utf8_zh_cn,
    bench_icu_to_utf16_zh_cn,
    bench_uconv_to_utf16_zh_cn,
    bench_webkit_to_utf16_zh_cn,
    bench_windows_to_utf16_zh_cn,
    bench_decode_to_utf8_zh_cn_gb18030,
    bench_decode_to_utf16_zh_cn_gb18030,
    bench_decode_to_string_zh_cn_gb18030,
    bench_rust_to_string_zh_cn_gb18030,
    bench_iconv_to_utf8_zh_cn_gb18030,
    bench_icu_to_utf16_zh_cn_gb18030,
    bench_uconv_to_utf16_zh_cn_gb18030,
    bench_windows_to_utf16_zh_cn_gb18030,
    GB18030,
    54936,
    "wikipedia/zh_cn.html"
);
encode_bench!(
    bench_encode_from_utf8_zh_cn,
    bench_encode_from_utf16_zh_cn,
    bench_encode_to_vec_zh_cn,
    bench_rust_to_vec_zh_cn,
    bench_iconv_from_utf8_zh_cn,
    bench_icu_from_utf16_zh_cn,
    bench_uconv_from_utf16_zh_cn,
    bench_webkit_from_utf16_zh_cn,
    bench_windows_from_utf16_zh_cn,
    bench_encode_from_utf8_zh_cn_gb18030,
    bench_encode_from_utf16_zh_cn_gb18030,
    bench_encode_to_vec_zh_cn_gb18030,
    bench_rust_to_vec_zh_cn_gb18030,
    bench_iconv_from_utf8_zh_cn_gb18030,
    bench_icu_from_utf16_zh_cn_gb18030,
    bench_uconv_from_utf16_zh_cn_gb18030,
    bench_windows_from_utf16_zh_cn_gb18030,
    GB18030,
    54936,
    "wikipedia/zh_cn.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_zh_cn_utf_16le,
    bench_decode_to_utf16_zh_cn_utf_16le,
    bench_decode_to_string_zh_cn_utf_16le,
    bench_rust_to_string_zh_cn_utf_16le,
    bench_iconv_to_utf8_zh_cn_utf_16le,
    bench_icu_to_utf16_zh_cn_utf_16le,
    bench_uconv_to_utf16_zh_cn_utf_16le,
    bench_windows_to_utf16_zh_cn_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/zh_cn.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_zh_cn_utf_16be,
    bench_decode_to_utf16_zh_cn_utf_16be,
    bench_decode_to_string_zh_cn_utf_16be,
    bench_rust_to_string_zh_cn_utf_16be,
    bench_iconv_to_utf8_zh_cn_utf_16be,
    bench_icu_to_utf16_zh_cn_utf_16be,
    bench_uconv_to_utf16_zh_cn_utf_16be,
    bench_windows_to_utf16_zh_cn_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/zh_cn.html"
);
decode_bench!(
    bench_copy_zh_tw,
    bench_decode_to_utf8_zh_tw,
    bench_decode_to_utf16_zh_tw,
    bench_decode_to_string_zh_tw,
    bench_rust_to_string_zh_tw,
    bench_std_validation_zh_tw,
    bench_kewb_to_utf16_zh_tw,
    bench_iconv_to_utf8_zh_tw,
    bench_icu_to_utf16_zh_tw,
    bench_uconv_to_utf16_zh_tw,
    bench_webkit_to_utf16_zh_tw,
    bench_windows_to_utf16_zh_tw,
    bench_decode_to_utf8_zh_tw_big5,
    bench_decode_to_utf16_zh_tw_big5,
    bench_decode_to_string_zh_tw_big5,
    bench_rust_to_string_zh_tw_big5,
    bench_iconv_to_utf8_zh_tw_big5,
    bench_icu_to_utf16_zh_tw_big5,
    bench_uconv_to_utf16_zh_tw_big5,
    bench_windows_to_utf16_zh_tw_big5,
    BIG5,
    950,
    "wikipedia/zh_tw.html"
);
encode_bench!(
    bench_encode_from_utf8_zh_tw,
    bench_encode_from_utf16_zh_tw,
    bench_encode_to_vec_zh_tw,
    bench_rust_to_vec_zh_tw,
    bench_iconv_from_utf8_zh_tw,
    bench_icu_from_utf16_zh_tw,
    bench_uconv_from_utf16_zh_tw,
    bench_webkit_from_utf16_zh_tw,
    bench_windows_from_utf16_zh_tw,
    bench_encode_from_utf8_zh_tw_big5,
    bench_encode_from_utf16_zh_tw_big5,
    bench_encode_to_vec_zh_tw_big5,
    bench_rust_to_vec_zh_tw_big5,
    bench_iconv_from_utf8_zh_tw_big5,
    bench_icu_from_utf16_zh_tw_big5,
    bench_uconv_from_utf16_zh_tw_big5,
    bench_windows_from_utf16_zh_tw_big5,
    BIG5,
    950,
    "wikipedia/zh_tw.txt"
);
decode_bench_legacy!(
    bench_decode_to_utf8_zh_tw_utf_16le,
    bench_decode_to_utf16_zh_tw_utf_16le,
    bench_decode_to_string_zh_tw_utf_16le,
    bench_rust_to_string_zh_tw_utf_16le,
    bench_iconv_to_utf8_zh_tw_utf_16le,
    bench_icu_to_utf16_zh_tw_utf_16le,
    bench_uconv_to_utf16_zh_tw_utf_16le,
    bench_windows_to_utf16_zh_tw_utf_16le,
    UTF_16LE,
    1200,
    "wikipedia/zh_tw.html"
);
decode_bench_legacy!(
    bench_decode_to_utf8_zh_tw_utf_16be,
    bench_decode_to_utf16_zh_tw_utf_16be,
    bench_decode_to_string_zh_tw_utf_16be,
    bench_rust_to_string_zh_tw_utf_16be,
    bench_iconv_to_utf8_zh_tw_utf_16be,
    bench_icu_to_utf16_zh_tw_utf_16be,
    bench_uconv_to_utf16_zh_tw_utf_16be,
    bench_windows_to_utf16_zh_tw_utf_16be,
    UTF_16BE,
    1201,
    "wikipedia/zh_tw.html"
);

decode_bench_webkit!(
    bench_webkit_to_utf16_ar_utf_16le,
    UTF_16LE,
    "wikipedia/ar.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_ar_utf_16be,
    UTF_16BE,
    "wikipedia/ar.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_cs_utf_16le,
    UTF_16LE,
    "wikipedia/cs.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_cs_utf_16be,
    UTF_16BE,
    "wikipedia/cs.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_de_utf_16le,
    UTF_16LE,
    "wikipedia/de.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_de_utf_16be,
    UTF_16BE,
    "wikipedia/de.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_el_utf_16le,
    UTF_16LE,
    "wikipedia/el.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_el_utf_16be,
    UTF_16BE,
    "wikipedia/el.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_en_utf_16le,
    UTF_16LE,
    "wikipedia/en.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_en_utf_16be,
    UTF_16BE,
    "wikipedia/en.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_fr_utf_16le,
    UTF_16LE,
    "wikipedia/fr.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_fr_utf_16be,
    UTF_16BE,
    "wikipedia/fr.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_he_utf_16le,
    UTF_16LE,
    "wikipedia/he.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_he_utf_16be,
    UTF_16BE,
    "wikipedia/he.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_ja_utf_16le,
    UTF_16LE,
    "wikipedia/ja.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_ja_utf_16be,
    UTF_16BE,
    "wikipedia/ja.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_ko_utf_16le,
    UTF_16LE,
    "wikipedia/ko.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_ko_utf_16be,
    UTF_16BE,
    "wikipedia/ko.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_pt_utf_16le,
    UTF_16LE,
    "wikipedia/pt.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_pt_utf_16be,
    UTF_16BE,
    "wikipedia/pt.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_ru_utf_16le,
    UTF_16LE,
    "wikipedia/ru.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_ru_utf_16be,
    UTF_16BE,
    "wikipedia/ru.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_th_utf_16le,
    UTF_16LE,
    "wikipedia/th.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_th_utf_16be,
    UTF_16BE,
    "wikipedia/th.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_tr_utf_16le,
    UTF_16LE,
    "wikipedia/tr.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_tr_utf_16be,
    UTF_16BE,
    "wikipedia/tr.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_vi_utf_16le,
    UTF_16LE,
    "wikipedia/vi.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_vi_utf_16be,
    UTF_16BE,
    "wikipedia/vi.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_zh_cn_utf_16le,
    UTF_16LE,
    "wikipedia/vi.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_zh_cn_utf_16be,
    UTF_16BE,
    "wikipedia/vi.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_zh_tw_utf_16le,
    UTF_16LE,
    "wikipedia/vi.html"
);
decode_bench_webkit!(
    bench_webkit_to_utf16_zh_tw_utf_16be,
    UTF_16BE,
    "wikipedia/vi.html"
);

mem_bench_is_u8!(
    bench_mem_is_ascii_true_1,
    bench_safe_mem_is_ascii_true_1,
    is_ascii,
    1,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_false_1,
    bench_safe_mem_is_ascii_false_1,
    is_ascii,
    1,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_true_1,
    bench_safe_mem_is_utf8_latin1_true_1,
    is_utf8_latin1,
    1,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_false_1,
    bench_safe_mem_is_utf8_latin1_false_1,
    is_utf8_latin1,
    1,
    "wikipedia/ar.txt"
);

mem_bench_is_str!(
    bench_mem_is_str_latin1_true_1,
    bench_safe_mem_is_str_latin1_true_1,
    is_str_latin1,
    1,
    "wikipedia/de-edit.txt"
);
mem_bench_is_str!(
    bench_mem_is_str_latin1_false_1,
    bench_safe_mem_is_str_latin1_false_1,
    is_str_latin1,
    1,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_is_basic_latin_true_1,
    bench_safe_mem_is_basic_latin_true_1,
    is_basic_latin,
    1,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_is_basic_latin_false_1,
    bench_safe_mem_is_basic_latin_false_1,
    is_basic_latin,
    1,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_true_1,
    bench_safe_mem_is_utf16_latin1_true_1,
    is_utf16_latin1,
    1,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_false_1,
    bench_safe_mem_is_utf16_latin1_false_1,
    is_utf16_latin1,
    1,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_ascii_1,
    bench_safe_mem_utf16_valid_up_to_ascii_1,
    utf16_valid_up_to,
    1,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_latin1_1,
    bench_safe_mem_utf16_valid_up_to_latin1_1,
    utf16_valid_up_to,
    1,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_bmp_1,
    bench_safe_mem_utf16_valid_up_to_bmp_1,
    utf16_valid_up_to,
    1,
    "wikipedia/ar.txt"
);

mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_ascii_1,
    bench_safe_mem_ensure_utf16_validity_ascii_1,
    ensure_utf16_validity,
    1,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_latin1_1,
    bench_safe_mem_ensure_utf16_validity_latin1_1,
    ensure_utf16_validity,
    1,
    "wikipedia/de-edit.txt"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_bmp_1,
    bench_safe_mem_ensure_utf16_validity_bmp_1,
    ensure_utf16_validity,
    1,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_ascii_1,
    bench_safe_mem_convert_utf8_to_utf16_ascii_1,
    convert_utf8_to_utf16,
    1,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_bmp_1,
    bench_safe_mem_convert_utf8_to_utf16_bmp_1,
    convert_utf8_to_utf16,
    1,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_latin1_to_utf16_1,
    bench_safe_mem_convert_latin1_to_utf16_1,
    convert_latin1_to_utf16,
    1,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_copy_ascii_to_basic_latin_1,
    bench_safe_mem_copy_ascii_to_basic_latin_1,
    copy_ascii_to_basic_latin,
    1,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_ascii_1,
    bench_safe_mem_convert_str_to_utf16_ascii_1,
    convert_str_to_utf16,
    1,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_bmp_1,
    bench_safe_mem_convert_str_to_utf16_bmp_1,
    convert_str_to_utf16,
    1,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_ascii_1,
    bench_safe_mem_convert_utf16_to_utf8_ascii_1,
    convert_utf16_to_utf8,
    1,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_bmp_1,
    bench_safe_mem_convert_utf16_to_utf8_bmp_1,
    convert_utf16_to_utf8,
    1,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_latin1_lossy_1,
    bench_safe_mem_convert_utf16_to_latin1_lossy_1,
    convert_utf16_to_latin1_lossy,
    1,
    "wikipedia/de-edit.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_copy_basic_latin_to_ascii_1,
    bench_safe_mem_copy_basic_latin_to_ascii_1,
    copy_basic_latin_to_ascii,
    1,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_ascii_1,
    bench_safe_mem_convert_utf16_to_str_ascii_1,
    convert_utf16_to_str,
    1,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_bmp_1,
    bench_safe_mem_convert_utf16_to_str_bmp_1,
    convert_utf16_to_str,
    1,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_latin1_to_utf8_1,
    bench_safe_mem_convert_latin1_to_utf8_1,
    convert_latin1_to_utf8,
    1,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_utf8_to_latin1_lossy_1,
    bench_safe_mem_convert_utf8_to_latin1_lossy_1,
    convert_utf8_to_latin1_lossy,
    1,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_copy_ascii_to_ascii_1,
    bench_safe_mem_copy_ascii_to_ascii_1,
    copy_ascii_to_ascii,
    1,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u8_to_str!(
    bench_mem_convert_latin1_to_str_1,
    bench_safe_mem_convert_latin1_to_str_1,
    convert_latin1_to_str,
    1,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_true_3,
    bench_safe_mem_is_ascii_true_3,
    is_ascii,
    3,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_false_3,
    bench_safe_mem_is_ascii_false_3,
    is_ascii,
    3,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_true_3,
    bench_safe_mem_is_utf8_latin1_true_3,
    is_utf8_latin1,
    3,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_false_3,
    bench_safe_mem_is_utf8_latin1_false_3,
    is_utf8_latin1,
    3,
    "wikipedia/ar.txt"
);

mem_bench_is_str!(
    bench_mem_is_str_latin1_true_3,
    bench_safe_mem_is_str_latin1_true_3,
    is_str_latin1,
    3,
    "wikipedia/de-edit.txt"
);
mem_bench_is_str!(
    bench_mem_is_str_latin1_false_3,
    bench_safe_mem_is_str_latin1_false_3,
    is_str_latin1,
    3,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_is_basic_latin_true_3,
    bench_safe_mem_is_basic_latin_true_3,
    is_basic_latin,
    3,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_is_basic_latin_false_3,
    bench_safe_mem_is_basic_latin_false_3,
    is_basic_latin,
    3,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_true_3,
    bench_safe_mem_is_utf16_latin1_true_3,
    is_utf16_latin1,
    3,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_false_3,
    bench_safe_mem_is_utf16_latin1_false_3,
    is_utf16_latin1,
    3,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_ascii_3,
    bench_safe_mem_utf16_valid_up_to_ascii_3,
    utf16_valid_up_to,
    3,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_latin1_3,
    bench_safe_mem_utf16_valid_up_to_latin1_3,
    utf16_valid_up_to,
    3,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_bmp_3,
    bench_safe_mem_utf16_valid_up_to_bmp_3,
    utf16_valid_up_to,
    3,
    "wikipedia/ar.txt"
);

mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_ascii_3,
    bench_safe_mem_ensure_utf16_validity_ascii_3,
    ensure_utf16_validity,
    3,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_latin1_3,
    bench_safe_mem_ensure_utf16_validity_latin1_3,
    ensure_utf16_validity,
    3,
    "wikipedia/de-edit.txt"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_bmp_3,
    bench_safe_mem_ensure_utf16_validity_bmp_3,
    ensure_utf16_validity,
    3,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_ascii_3,
    bench_safe_mem_convert_utf8_to_utf16_ascii_3,
    convert_utf8_to_utf16,
    3,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_bmp_3,
    bench_safe_mem_convert_utf8_to_utf16_bmp_3,
    convert_utf8_to_utf16,
    3,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_latin1_to_utf16_3,
    bench_safe_mem_convert_latin1_to_utf16_3,
    convert_latin1_to_utf16,
    3,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_copy_ascii_to_basic_latin_3,
    bench_safe_mem_copy_ascii_to_basic_latin_3,
    copy_ascii_to_basic_latin,
    3,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_ascii_3,
    bench_safe_mem_convert_str_to_utf16_ascii_3,
    convert_str_to_utf16,
    3,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_bmp_3,
    bench_safe_mem_convert_str_to_utf16_bmp_3,
    convert_str_to_utf16,
    3,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_ascii_3,
    bench_safe_mem_convert_utf16_to_utf8_ascii_3,
    convert_utf16_to_utf8,
    3,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_bmp_3,
    bench_safe_mem_convert_utf16_to_utf8_bmp_3,
    convert_utf16_to_utf8,
    3,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_latin1_lossy_3,
    bench_safe_mem_convert_utf16_to_latin1_lossy_3,
    convert_utf16_to_latin1_lossy,
    3,
    "wikipedia/de-edit.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_copy_basic_latin_to_ascii_3,
    bench_safe_mem_copy_basic_latin_to_ascii_3,
    copy_basic_latin_to_ascii,
    3,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_ascii_3,
    bench_safe_mem_convert_utf16_to_str_ascii_3,
    convert_utf16_to_str,
    3,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_bmp_3,
    bench_safe_mem_convert_utf16_to_str_bmp_3,
    convert_utf16_to_str,
    3,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_latin1_to_utf8_3,
    bench_safe_mem_convert_latin1_to_utf8_3,
    convert_latin1_to_utf8,
    3,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_utf8_to_latin1_lossy_3,
    bench_safe_mem_convert_utf8_to_latin1_lossy_3,
    convert_utf8_to_latin1_lossy,
    3,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_copy_ascii_to_ascii_3,
    bench_safe_mem_copy_ascii_to_ascii_3,
    copy_ascii_to_ascii,
    3,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u8_to_str!(
    bench_mem_convert_latin1_to_str_3,
    bench_safe_mem_convert_latin1_to_str_3,
    convert_latin1_to_str,
    3,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_true_15,
    bench_safe_mem_is_ascii_true_15,
    is_ascii,
    15,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_false_15,
    bench_safe_mem_is_ascii_false_15,
    is_ascii,
    15,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_true_15,
    bench_safe_mem_is_utf8_latin1_true_15,
    is_utf8_latin1,
    15,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_false_15,
    bench_safe_mem_is_utf8_latin1_false_15,
    is_utf8_latin1,
    15,
    "wikipedia/ar.txt"
);

mem_bench_is_str!(
    bench_mem_is_str_latin1_true_15,
    bench_safe_mem_is_str_latin1_true_15,
    is_str_latin1,
    15,
    "wikipedia/de-edit.txt"
);
mem_bench_is_str!(
    bench_mem_is_str_latin1_false_15,
    bench_safe_mem_is_str_latin1_false_15,
    is_str_latin1,
    15,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_is_basic_latin_true_15,
    bench_safe_mem_is_basic_latin_true_15,
    is_basic_latin,
    15,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_is_basic_latin_false_15,
    bench_safe_mem_is_basic_latin_false_15,
    is_basic_latin,
    15,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_true_15,
    bench_safe_mem_is_utf16_latin1_true_15,
    is_utf16_latin1,
    15,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_false_15,
    bench_safe_mem_is_utf16_latin1_false_15,
    is_utf16_latin1,
    15,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_ascii_15,
    bench_safe_mem_utf16_valid_up_to_ascii_15,
    utf16_valid_up_to,
    15,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_latin1_15,
    bench_safe_mem_utf16_valid_up_to_latin1_15,
    utf16_valid_up_to,
    15,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_bmp_15,
    bench_safe_mem_utf16_valid_up_to_bmp_15,
    utf16_valid_up_to,
    15,
    "wikipedia/ar.txt"
);

mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_ascii_15,
    bench_safe_mem_ensure_utf16_validity_ascii_15,
    ensure_utf16_validity,
    15,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_latin1_15,
    bench_safe_mem_ensure_utf16_validity_latin1_15,
    ensure_utf16_validity,
    15,
    "wikipedia/de-edit.txt"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_bmp_15,
    bench_safe_mem_ensure_utf16_validity_bmp_15,
    ensure_utf16_validity,
    15,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_ascii_15,
    bench_safe_mem_convert_utf8_to_utf16_ascii_15,
    convert_utf8_to_utf16,
    15,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_bmp_15,
    bench_safe_mem_convert_utf8_to_utf16_bmp_15,
    convert_utf8_to_utf16,
    15,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_latin1_to_utf16_15,
    bench_safe_mem_convert_latin1_to_utf16_15,
    convert_latin1_to_utf16,
    15,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_copy_ascii_to_basic_latin_15,
    bench_safe_mem_copy_ascii_to_basic_latin_15,
    copy_ascii_to_basic_latin,
    15,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_ascii_15,
    bench_safe_mem_convert_str_to_utf16_ascii_15,
    convert_str_to_utf16,
    15,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_bmp_15,
    bench_safe_mem_convert_str_to_utf16_bmp_15,
    convert_str_to_utf16,
    15,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_ascii_15,
    bench_safe_mem_convert_utf16_to_utf8_ascii_15,
    convert_utf16_to_utf8,
    15,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_bmp_15,
    bench_safe_mem_convert_utf16_to_utf8_bmp_15,
    convert_utf16_to_utf8,
    15,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_latin1_lossy_15,
    bench_safe_mem_convert_utf16_to_latin1_lossy_15,
    convert_utf16_to_latin1_lossy,
    15,
    "wikipedia/de-edit.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_copy_basic_latin_to_ascii_15,
    bench_safe_mem_copy_basic_latin_to_ascii_15,
    copy_basic_latin_to_ascii,
    15,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_ascii_15,
    bench_safe_mem_convert_utf16_to_str_ascii_15,
    convert_utf16_to_str,
    15,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_bmp_15,
    bench_safe_mem_convert_utf16_to_str_bmp_15,
    convert_utf16_to_str,
    15,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_latin1_to_utf8_15,
    bench_safe_mem_convert_latin1_to_utf8_15,
    convert_latin1_to_utf8,
    15,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_utf8_to_latin1_lossy_15,
    bench_safe_mem_convert_utf8_to_latin1_lossy_15,
    convert_utf8_to_latin1_lossy,
    15,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_copy_ascii_to_ascii_15,
    bench_safe_mem_copy_ascii_to_ascii_15,
    copy_ascii_to_ascii,
    15,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u8_to_str!(
    bench_mem_convert_latin1_to_str_15,
    bench_safe_mem_convert_latin1_to_str_15,
    convert_latin1_to_str,
    15,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_true_16,
    bench_safe_mem_is_ascii_true_16,
    is_ascii,
    16,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_false_16,
    bench_safe_mem_is_ascii_false_16,
    is_ascii,
    16,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_true_16,
    bench_safe_mem_is_utf8_latin1_true_16,
    is_utf8_latin1,
    16,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_false_16,
    bench_safe_mem_is_utf8_latin1_false_16,
    is_utf8_latin1,
    16,
    "wikipedia/ar.txt"
);

mem_bench_is_str!(
    bench_mem_is_str_latin1_true_16,
    bench_safe_mem_is_str_latin1_true_16,
    is_str_latin1,
    16,
    "wikipedia/de-edit.txt"
);
mem_bench_is_str!(
    bench_mem_is_str_latin1_false_16,
    bench_safe_mem_is_str_latin1_false_16,
    is_str_latin1,
    16,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_is_basic_latin_true_16,
    bench_safe_mem_is_basic_latin_true_16,
    is_basic_latin,
    16,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_is_basic_latin_false_16,
    bench_safe_mem_is_basic_latin_false_16,
    is_basic_latin,
    16,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_true_16,
    bench_safe_mem_is_utf16_latin1_true_16,
    is_utf16_latin1,
    16,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_false_16,
    bench_safe_mem_is_utf16_latin1_false_16,
    is_utf16_latin1,
    16,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_ascii_16,
    bench_safe_mem_utf16_valid_up_to_ascii_16,
    utf16_valid_up_to,
    16,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_latin1_16,
    bench_safe_mem_utf16_valid_up_to_latin1_16,
    utf16_valid_up_to,
    16,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_bmp_16,
    bench_safe_mem_utf16_valid_up_to_bmp_16,
    utf16_valid_up_to,
    16,
    "wikipedia/ar.txt"
);

mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_ascii_16,
    bench_safe_mem_ensure_utf16_validity_ascii_16,
    ensure_utf16_validity,
    16,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_latin1_16,
    bench_safe_mem_ensure_utf16_validity_latin1_16,
    ensure_utf16_validity,
    16,
    "wikipedia/de-edit.txt"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_bmp_16,
    bench_safe_mem_ensure_utf16_validity_bmp_16,
    ensure_utf16_validity,
    16,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_ascii_16,
    bench_safe_mem_convert_utf8_to_utf16_ascii_16,
    convert_utf8_to_utf16,
    16,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_bmp_16,
    bench_safe_mem_convert_utf8_to_utf16_bmp_16,
    convert_utf8_to_utf16,
    16,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_latin1_to_utf16_16,
    bench_safe_mem_convert_latin1_to_utf16_16,
    convert_latin1_to_utf16,
    16,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_copy_ascii_to_basic_latin_16,
    bench_safe_mem_copy_ascii_to_basic_latin_16,
    copy_ascii_to_basic_latin,
    16,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_ascii_16,
    bench_safe_mem_convert_str_to_utf16_ascii_16,
    convert_str_to_utf16,
    16,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_bmp_16,
    bench_safe_mem_convert_str_to_utf16_bmp_16,
    convert_str_to_utf16,
    16,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_ascii_16,
    bench_safe_mem_convert_utf16_to_utf8_ascii_16,
    convert_utf16_to_utf8,
    16,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_bmp_16,
    bench_safe_mem_convert_utf16_to_utf8_bmp_16,
    convert_utf16_to_utf8,
    16,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_latin1_lossy_16,
    bench_safe_mem_convert_utf16_to_latin1_lossy_16,
    convert_utf16_to_latin1_lossy,
    16,
    "wikipedia/de-edit.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_copy_basic_latin_to_ascii_16,
    bench_safe_mem_copy_basic_latin_to_ascii_16,
    copy_basic_latin_to_ascii,
    16,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_ascii_16,
    bench_safe_mem_convert_utf16_to_str_ascii_16,
    convert_utf16_to_str,
    16,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_bmp_16,
    bench_safe_mem_convert_utf16_to_str_bmp_16,
    convert_utf16_to_str,
    16,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_latin1_to_utf8_16,
    bench_safe_mem_convert_latin1_to_utf8_16,
    convert_latin1_to_utf8,
    16,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_utf8_to_latin1_lossy_16,
    bench_safe_mem_convert_utf8_to_latin1_lossy_16,
    convert_utf8_to_latin1_lossy,
    16,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_copy_ascii_to_ascii_16,
    bench_safe_mem_copy_ascii_to_ascii_16,
    copy_ascii_to_ascii,
    16,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u8_to_str!(
    bench_mem_convert_latin1_to_str_16,
    bench_safe_mem_convert_latin1_to_str_16,
    convert_latin1_to_str,
    16,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_true_30,
    bench_safe_mem_is_ascii_true_30,
    is_ascii,
    30,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_false_30,
    bench_safe_mem_is_ascii_false_30,
    is_ascii,
    30,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_true_30,
    bench_safe_mem_is_utf8_latin1_true_30,
    is_utf8_latin1,
    30,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_false_30,
    bench_safe_mem_is_utf8_latin1_false_30,
    is_utf8_latin1,
    30,
    "wikipedia/ar.txt"
);

mem_bench_is_str!(
    bench_mem_is_str_latin1_true_30,
    bench_safe_mem_is_str_latin1_true_30,
    is_str_latin1,
    30,
    "wikipedia/de-edit.txt"
);
mem_bench_is_str!(
    bench_mem_is_str_latin1_false_30,
    bench_safe_mem_is_str_latin1_false_30,
    is_str_latin1,
    30,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_is_basic_latin_true_30,
    bench_safe_mem_is_basic_latin_true_30,
    is_basic_latin,
    30,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_is_basic_latin_false_30,
    bench_safe_mem_is_basic_latin_false_30,
    is_basic_latin,
    30,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_true_30,
    bench_safe_mem_is_utf16_latin1_true_30,
    is_utf16_latin1,
    30,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_false_30,
    bench_safe_mem_is_utf16_latin1_false_30,
    is_utf16_latin1,
    30,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_ascii_30,
    bench_safe_mem_utf16_valid_up_to_ascii_30,
    utf16_valid_up_to,
    30,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_latin1_30,
    bench_safe_mem_utf16_valid_up_to_latin1_30,
    utf16_valid_up_to,
    30,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_bmp_30,
    bench_safe_mem_utf16_valid_up_to_bmp_30,
    utf16_valid_up_to,
    30,
    "wikipedia/ar.txt"
);

mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_ascii_30,
    bench_safe_mem_ensure_utf16_validity_ascii_30,
    ensure_utf16_validity,
    30,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_latin1_30,
    bench_safe_mem_ensure_utf16_validity_latin1_30,
    ensure_utf16_validity,
    30,
    "wikipedia/de-edit.txt"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_bmp_30,
    bench_safe_mem_ensure_utf16_validity_bmp_30,
    ensure_utf16_validity,
    30,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_ascii_30,
    bench_safe_mem_convert_utf8_to_utf16_ascii_30,
    convert_utf8_to_utf16,
    30,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_bmp_30,
    bench_safe_mem_convert_utf8_to_utf16_bmp_30,
    convert_utf8_to_utf16,
    30,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_latin1_to_utf16_30,
    bench_safe_mem_convert_latin1_to_utf16_30,
    convert_latin1_to_utf16,
    30,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_copy_ascii_to_basic_latin_30,
    bench_safe_mem_copy_ascii_to_basic_latin_30,
    copy_ascii_to_basic_latin,
    30,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_ascii_30,
    bench_safe_mem_convert_str_to_utf16_ascii_30,
    convert_str_to_utf16,
    30,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_bmp_30,
    bench_safe_mem_convert_str_to_utf16_bmp_30,
    convert_str_to_utf16,
    30,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_ascii_30,
    bench_safe_mem_convert_utf16_to_utf8_ascii_30,
    convert_utf16_to_utf8,
    30,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_bmp_30,
    bench_safe_mem_convert_utf16_to_utf8_bmp_30,
    convert_utf16_to_utf8,
    30,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_latin1_lossy_30,
    bench_safe_mem_convert_utf16_to_latin1_lossy_30,
    convert_utf16_to_latin1_lossy,
    30,
    "wikipedia/de-edit.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_copy_basic_latin_to_ascii_30,
    bench_safe_mem_copy_basic_latin_to_ascii_30,
    copy_basic_latin_to_ascii,
    30,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_ascii_30,
    bench_safe_mem_convert_utf16_to_str_ascii_30,
    convert_utf16_to_str,
    30,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_bmp_30,
    bench_safe_mem_convert_utf16_to_str_bmp_30,
    convert_utf16_to_str,
    30,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_latin1_to_utf8_30,
    bench_safe_mem_convert_latin1_to_utf8_30,
    convert_latin1_to_utf8,
    30,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_utf8_to_latin1_lossy_30,
    bench_safe_mem_convert_utf8_to_latin1_lossy_30,
    convert_utf8_to_latin1_lossy,
    30,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_copy_ascii_to_ascii_30,
    bench_safe_mem_copy_ascii_to_ascii_30,
    copy_ascii_to_ascii,
    30,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u8_to_str!(
    bench_mem_convert_latin1_to_str_30,
    bench_safe_mem_convert_latin1_to_str_30,
    convert_latin1_to_str,
    30,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_true_1000,
    bench_safe_mem_is_ascii_true_1000,
    is_ascii,
    1000,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u8!(
    bench_mem_is_ascii_false_1000,
    bench_safe_mem_is_ascii_false_1000,
    is_ascii,
    1000,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_true_1000,
    bench_safe_mem_is_utf8_latin1_true_1000,
    is_utf8_latin1,
    1000,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u8!(
    bench_mem_is_utf8_latin1_false_1000,
    bench_safe_mem_is_utf8_latin1_false_1000,
    is_utf8_latin1,
    1000,
    "wikipedia/ar.txt"
);

mem_bench_is_str!(
    bench_mem_is_str_latin1_true_1000,
    bench_safe_mem_is_str_latin1_true_1000,
    is_str_latin1,
    1000,
    "wikipedia/de-edit.txt"
);
mem_bench_is_str!(
    bench_mem_is_str_latin1_false_1000,
    bench_safe_mem_is_str_latin1_false_1000,
    is_str_latin1,
    1000,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_is_basic_latin_true_1000,
    bench_safe_mem_is_basic_latin_true_1000,
    is_basic_latin,
    1000,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_is_basic_latin_false_1000,
    bench_safe_mem_is_basic_latin_false_1000,
    is_basic_latin,
    1000,
    "wikipedia/de-edit.txt"
);

mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_true_1000,
    bench_safe_mem_is_utf16_latin1_true_1000,
    is_utf16_latin1,
    1000,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_is_utf16_latin1_false_1000,
    bench_safe_mem_is_utf16_latin1_false_1000,
    is_utf16_latin1,
    1000,
    "wikipedia/ar.txt"
);

mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_ascii_1000,
    bench_safe_mem_utf16_valid_up_to_ascii_1000,
    utf16_valid_up_to,
    1000,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_latin1_1000,
    bench_safe_mem_utf16_valid_up_to_latin1_1000,
    utf16_valid_up_to,
    1000,
    "wikipedia/de-edit.txt"
);
mem_bench_is_u16!(
    bench_mem_utf16_valid_up_to_bmp_1000,
    bench_safe_mem_utf16_valid_up_to_bmp_1000,
    utf16_valid_up_to,
    1000,
    "wikipedia/ar.txt"
);

mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_ascii_1000,
    bench_safe_mem_ensure_utf16_validity_ascii_1000,
    ensure_utf16_validity,
    1000,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_latin1_1000,
    bench_safe_mem_ensure_utf16_validity_latin1_1000,
    ensure_utf16_validity,
    1000,
    "wikipedia/de-edit.txt"
);
mem_bench_mut_u16!(
    bench_mem_ensure_utf16_validity_bmp_1000,
    bench_safe_mem_ensure_utf16_validity_bmp_1000,
    ensure_utf16_validity,
    1000,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_ascii_1000,
    bench_safe_mem_convert_utf8_to_utf16_ascii_1000,
    convert_utf8_to_utf16,
    1000,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u8_to_u16!(
    bench_mem_convert_utf8_to_utf16_bmp_1000,
    bench_safe_mem_convert_utf8_to_utf16_bmp_1000,
    convert_utf8_to_utf16,
    1000,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_convert_latin1_to_utf16_1000,
    bench_safe_mem_convert_latin1_to_utf16_1000,
    convert_latin1_to_utf16,
    1000,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u16!(
    bench_mem_copy_ascii_to_basic_latin_1000,
    bench_safe_mem_copy_ascii_to_basic_latin_1000,
    copy_ascii_to_basic_latin,
    1000,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_ascii_1000,
    bench_safe_mem_convert_str_to_utf16_ascii_1000,
    convert_str_to_utf16,
    1000,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_str_to_u16!(
    bench_mem_convert_str_to_utf16_bmp_1000,
    bench_safe_mem_convert_str_to_utf16_bmp_1000,
    convert_str_to_utf16,
    1000,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_ascii_1000,
    bench_safe_mem_convert_utf16_to_utf8_ascii_1000,
    convert_utf16_to_utf8,
    1000,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_utf8_bmp_1000,
    bench_safe_mem_convert_utf16_to_utf8_bmp_1000,
    convert_utf16_to_utf8,
    1000,
    "wikipedia/ar.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_convert_utf16_to_latin1_lossy_1000,
    bench_safe_mem_convert_utf16_to_latin1_lossy_1000,
    convert_utf16_to_latin1_lossy,
    1000,
    "wikipedia/de-edit.txt"
);

mem_bench_u16_to_u8!(
    bench_mem_copy_basic_latin_to_ascii_1000,
    bench_safe_mem_copy_basic_latin_to_ascii_1000,
    copy_basic_latin_to_ascii,
    1000,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_ascii_1000,
    bench_safe_mem_convert_utf16_to_str_ascii_1000,
    convert_utf16_to_str,
    1000,
    "jquery/jquery-3.1.1.min.js"
);
mem_bench_u16_to_str!(
    bench_mem_convert_utf16_to_str_bmp_1000,
    bench_safe_mem_convert_utf16_to_str_bmp_1000,
    convert_utf16_to_str,
    1000,
    "wikipedia/ar.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_latin1_to_utf8_1000,
    bench_safe_mem_convert_latin1_to_utf8_1000,
    convert_latin1_to_utf8,
    1000,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_convert_utf8_to_latin1_lossy_1000,
    bench_safe_mem_convert_utf8_to_latin1_lossy_1000,
    convert_utf8_to_latin1_lossy,
    1000,
    "wikipedia/de-edit.txt"
);

mem_bench_u8_to_u8!(
    bench_mem_copy_ascii_to_ascii_1000,
    bench_safe_mem_copy_ascii_to_ascii_1000,
    copy_ascii_to_ascii,
    1000,
    "jquery/jquery-3.1.1.min.js"
);

mem_bench_u8_to_str!(
    bench_mem_convert_latin1_to_str_1000,
    bench_safe_mem_convert_latin1_to_str_1000,
    convert_latin1_to_str,
    1000,
    "wikipedia/de-edit.txt"
);
// END GENERATED CODE
