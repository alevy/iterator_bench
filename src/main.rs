#![feature(stmt_expr_attributes, test)]

extern crate rand;
extern crate test;

fn main() {
}

macro_rules! conv {
    ( $name:ident, $output:ty, $input:ty, $impl:expr ) => {
        mod $name {
            use test::{Bencher, black_box};
            #[test]
            fn _test() {
                let mut app_buf: [u8; 640000] = [0; 640000];
                let mut other_buf: [u16; 320000] = [0; 320000];
                for c in other_buf.iter_mut() {
                    *c = 0xffaa;
                }

                _impl(&mut app_buf, &other_buf);

                for (i, c) in app_buf.iter().enumerate() {
                    if i % 2 == 0 {
                      assert_eq!(*c, 0xaa);
                    } else {
                      assert_eq!(*c, 0xff);
                    }
                }
            }

            #[bench]
            fn _bench(bencher: &mut Bencher) {
                let mut app_buf: [u8; 640000] = [0; 640000];
                let mut other_buf: [u16; 320000] = [0; 320000];
                for i in 0..320000 {
                     other_buf[i] = rand::random();
                }

                bencher.iter(|| {
                  black_box(_impl(&mut app_buf, &other_buf));
                });
            }

            #[inline(never)]
            fn _impl(output: &mut $output, input: &$input) {
                $impl(output, input)
            }
        }
    }
}


conv!(c_style_fixed_size, [u8; 640000], [u16; 320000], |output: &mut [u8; 640000], input: &[u16; 320000]| {
    for i in 0..input.len() {
        let b = input[i];
        output[2 * i] = (b & 0xff) as u8;
        output[2 * i + 1] = ((b >> 8) & 0xff) as u8;
    }
});

conv!(c_style_input_size_fixed, [u8], [u16; 320000], |output: &mut [u8], input: &[u16; 320000]| {
    for (i, &b) in input.iter().enumerate() {
        output[2 * i] = (b & 0xff) as u8;
        output[2 * i + 1] = ((b >> 8) & 0xff) as u8;
   }
});

conv!(c_style_output_size_fixed, [u8; 640000], [u16], |output: &mut [u8; 640000], input: &[u16]| {
    for i in 0..input.len() {
        let b = input[i];
        output[2 * i] = (b & 0xff) as u8;
        output[2 * i + 1] = ((b >> 8) & 0xff) as u8;
   }
});

conv!(c_style_unknown_size, [u8], [u16], #[inline(never)] |output: &mut [u8], input: &[u16]| {
    for i in 0..input.len() {
        let b = input[i];
        output[2 * i] = (b & 0xff) as u8;
        output[2 * i + 1] = ((b >> 8) & 0xff) as u8;
   }
});

conv!(zip_chunks_fixed_size, [u8; 640000], [u16; 320000], |output: &mut [u8; 640000], input: &[u16; 320000]| {
    for (&b, ac) in input.iter().zip(output.chunks_mut(2)) {
        // the type of `ac` is `[u8]`, but the optimizer should be able to figure out that it's
        // always size 2, and thus should elide the bounds checks in the inner loop.
        ac[0] = (b & 0xff) as u8;
        ac[1] = ((b >> 8) & 0xff) as u8;
    }
});

conv!(zip_chunks_fixed_size_take, [u8; 640000], [u16; 320000], |output: &mut [u8; 640000], input: &[u16; 320000]| {
    for (&b, ac) in input.iter().zip(output.chunks_mut(2)).take(320000) {
        // the type of `ac` is `[u8]`, but the optimizer should be able to figure out that it's
        // always size 2, and thus should elide the bounds checks in the inner loop.
        ac[0] = (b & 0xff) as u8;
        ac[1] = ((b >> 8) & 0xff) as u8;
    }
});

conv!(zip_chunks_fixed_size_take_iter, [u8; 640000], [u16; 320000], |output: &mut [u8; 640000], input: &[u16; 320000]| {
    for (&b, ac) in input.iter().zip(output.chunks_mut(2)).take(320000) {
        let mut val = b;
        for byte in ac.iter_mut() {
            *byte = (val & 0xFF) as u8;
            val = val >> 8;
        }
    }
});

conv!(zip_chunks_output_size_fixed, [u8; 640000], [u16], |output: &mut [u8; 640000], input: &[u16]| {
    for (&b, ac) in input.iter().zip(output.chunks_mut(2)) {
        // the type of `ac` is `[u8]`, but the optimizer should be able to figure out that it's
        // always size 2, and thus should elide the bounds checks in the inner loop.
        ac[0] = (b & 0xff) as u8;
        ac[1] = ((b >> 8) & 0xff) as u8;
    }
});

conv!(zip_chunks_input_size_fixed, [u8], [u16; 320000], |output: &mut [u8], input: &[u16; 320000]| {
    for (&b, ac) in input.iter().zip(output.chunks_mut(2)) {
        // the type of `ac` is `[u8]`, but the optimizer should be able to figure out that it's
        // always size 2, and thus should elide the bounds checks in the inner loop.
        ac[0] = (b & 0xff) as u8;
        ac[1] = ((b >> 8) & 0xff) as u8;
    }
});

conv!(zip_chunks_unknown_size, [u8], [u16], |output: &mut [u8], input: &[u16]| {
   for (b, ac) in input.iter().zip(output.chunks_mut(2)) {
     ac[0] = (*b & 0xff) as u8;
     ac[1] = ((*b >> 8) & 0xff) as u8;
   }
});

conv!(zip_chunks_exact_unknown_size_take, [u8], [u16], |output: &mut [u8], input: &[u16]| {
   for (b, ac) in input.iter().zip(output.chunks_exact_mut(2)) {
     ac[0] = (*b & 0xff) as u8;
     ac[1] = ((*b >> 8) & 0xff) as u8;
   }
});

conv!(zip_chunks_exact_unknown_size_take_iter, [u8], [u16], |output: &mut [u8], input: &[u16]| {
    for (&b, ac) in input.iter().zip(output.chunks_exact_mut(2)).take(320000) {
        let mut val = b;
        for byte in ac.iter_mut() {
            *byte = (val & 0xFF) as u8;
            val = val >> 8;
        }
    }
});
