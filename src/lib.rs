#![feature(test)]

extern crate test;
extern crate encoding_rs;

use test::Bencher;

const OUTPUT_BUFFER_LENGTH: usize = 512 * 1024;

#[bench]
fn bench_decode_to_utf8_en(b: &mut Bencher) {
    let input = include_bytes!("wikipedia/en.html");
    let mut output = [0u8; OUTPUT_BUFFER_LENGTH];
    b.bytes = input.len() as u64;
    b.iter(|| {
        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let (result, _, _, _) = decoder.decode_to_utf8(input, &mut output[..], true);
        match result {
            encoding_rs::CoderResult::InputEmpty => {}
            encoding_rs::CoderResult::OutputFull => {
                unreachable!("Output buffer too short.");
            }
        }
        test::black_box(&output);
    });
}
