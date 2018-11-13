#![feature(stmt_expr_attributes, test)]

extern crate rand;
extern crate test;

fn main() {
    let mut app_buf: [u8; 640000] = [0; 640000];
    let mut other_buf: [u16; 320000] = [0; 320000];
    for c in other_buf.iter_mut() {
        *c = 0xffaa;
    }
    optimal_unsafe::_impl(&mut app_buf, &other_buf);
    zip_chunks_fixed_size_take_iter::_impl(&mut app_buf, &other_buf);

    println!("{}", app_buf[55]);
}

macro_rules! benchmark {
    ( $name:ident, $output:ty, $input:ty, $impl:expr ) => {
        mod $name {
            #[allow(unused_imports)]
            use test::{black_box, Bencher};
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
            pub fn _impl(output: &mut $output, input: &$input) {
                $impl(output, input)
            }
        }
    };
}

/// This is the implementation we would write in C.
///
///   * It iterators over the input array with a C-style for-loop.
///
///   * It uses regular arrays (rather than slices with double-worded pointers)
///
///   * It explicitly avoids bounds-checking by using the unsafe `get_unchecked[_mut]` operation.
///
/// On my (@alevy's) laptop, with an 2.6GHz Intel i5 and 14-stage pipeline, it take just under
/// 40,000ns, which is approximately 104,000 clock cycles to loop and process over 640,000
/// elements.
///
/// This seems _more_ than optimal, but inspecting the assembly, the math checks out.
///
/// The compiler has managed to compile this into packed moves. So there are only 20,000 iterations
/// of the loop with 5 instructions each.
benchmark!(
    optimal_unsafe,
    [u8; 640000],
    [u16; 320000],
    |output: &mut [u8; 640000], input: &[u16; 320000]| for i in 0..input.len() {
        unsafe {
            let b = *input.get_unchecked(i);
            *output.get_unchecked_mut(2 * i) = (b & 0xff) as u8;
            *output.get_unchecked_mut(2 * i + 1) = ((b >> 8) & 0xff) as u8;
        }
    }
);

benchmark!(
    c_style_fixed_size,
    [u8; 640000],
    [u16; 320000],
    |output: &mut [u8; 640000], input: &[u16; 320000]| for i in 0..input.len() {
        let b = input[i];
        output[2 * i] = (b & 0xff) as u8;
        output[2 * i + 1] = ((b >> 8) & 0xff) as u8;
    }
);

benchmark!(
    c_style_input_size_fixed,
    [u8],
    [u16; 320000],
    |output: &mut [u8], input: &[u16; 320000]| for i in 0..input.len() {
        let b = input[i];
        output[2 * i] = (b & 0xff) as u8;
        output[2 * i + 1] = ((b >> 8) & 0xff) as u8;
    }
);

benchmark!(
    c_style_output_size_fixed,
    [u8; 640000],
    [u16],
    |output: &mut [u8; 640000], input: &[u16]| for i in 0..input.len() {
        let b = input[i];
        output[2 * i] = (b & 0xff) as u8;
        output[2 * i + 1] = ((b >> 8) & 0xff) as u8;
    }
);

benchmark!(
    c_style_unknown_size,
    [u8],
    [u16],
    |output: &mut [u8], input: &[u16]| for i in 0..input.len() {
        let b = input[i];
        output[2 * i] = (b & 0xff) as u8;
        output[2 * i + 1] = ((b >> 8) & 0xff) as u8;
    }
);

benchmark!(
    c_style_unknown_size_limit,
    [u8],
    [u16],
    |output: &mut [u8], input: &[u16]| {
        use std::cmp::max;
        for i in 0..max(320000, input.len()) {
            let b = input[i];
            output[2 * i] = (b & 0xff) as u8;
            output[2 * i + 1] = ((b >> 8) & 0xff) as u8;
        }
    }
);

benchmark!(
    zip_chunks_fixed_size,
    [u8; 640000],
    [u16; 320000],
    |output: &mut [u8; 640000], input: &[u16; 320000]| {
        for (&b, ac) in input.iter().zip(output.chunks_mut(2)) {
            // the type of `ac` is `[u8]`, but the optimizer should be able to figure out that it's
            // always size 2, and thus should elide the bounds checks in the inner loop.
            ac[0] = (b & 0xff) as u8;
            ac[1] = ((b >> 8) & 0xff) as u8;
        }
    }
);

benchmark!(
    zip_chunks_fixed_size_take,
    [u8; 640000],
    [u16; 320000],
    |output: &mut [u8; 640000], input: &[u16; 320000]| {
        for (&b, ac) in input.iter().zip(output.chunks_mut(2)).take(320000) {
            // the type of `ac` is `[u8]`, but the optimizer should be able to figure out that it's
            // always size 2, and thus should elide the bounds checks in the inner loop.
            ac[0] = (b & 0xff) as u8;
            ac[1] = ((b >> 8) & 0xff) as u8;
        }
    }
);

benchmark!(
    zip_chunks_fixed_size_take_iter,
    [u8; 640000],
    [u16; 320000],
    |output: &mut [u8; 640000], input: &[u16; 320000]| for (&b, ac) in
        input.iter().zip(output.chunks_mut(2)).take(320000)
    {
        let mut val = b;
        for byte in ac.iter_mut() {
            *byte = (val & 0xFF) as u8;
            val = val >> 8;
        }
    }
);

benchmark!(
    zip_chunks_output_size_fixed,
    [u8; 640000],
    [u16],
    |output: &mut [u8; 640000], input: &[u16]| {
        for (&b, ac) in input.iter().zip(output.chunks_mut(2)) {
            // the type of `ac` is `[u8]`, but the optimizer should be able to figure out that it's
            // always size 2, and thus should elide the bounds checks in the inner loop.
            ac[0] = (b & 0xff) as u8;
            ac[1] = ((b >> 8) & 0xff) as u8;
        }
    }
);

benchmark!(
    zip_chunks_input_size_fixed,
    [u8],
    [u16; 320000],
    |output: &mut [u8], input: &[u16; 320000]| {
        for (&b, ac) in input.iter().zip(output.chunks_mut(2)) {
            // the type of `ac` is `[u8]`, but the optimizer should be able to figure out that it's
            // always size 2, and thus should elide the bounds checks in the inner loop.
            ac[0] = (b & 0xff) as u8;
            ac[1] = ((b >> 8) & 0xff) as u8;
        }
    }
);

benchmark!(
    zip_chunks_unknown_size,
    [u8],
    [u16],
    |output: &mut [u8], input: &[u16]| for (b, ac) in input.iter().zip(output.chunks_mut(2)) {
        ac[0] = (*b & 0xff) as u8;
        ac[1] = ((*b >> 8) & 0xff) as u8;
    }
);

benchmark!(
    zip_chunks_unknown_size_take,
    [u8],
    [u16],
    |output: &mut [u8], input: &[u16]| for (b, ac) in
        input.iter().zip(output.chunks_mut(2)).take(320000)
    {
        ac[0] = (*b & 0xff) as u8;
        ac[1] = ((*b >> 8) & 0xff) as u8;
    }
);

benchmark!(
    zip_chunks_unknown_size_take_iter,
    [u8],
    [u16],
    |output: &mut [u8], input: &[u16]| for (&b, ac) in
        input.iter().zip(output.chunks_mut(2)).take(320000)
    {
        let mut val = b;
        for byte in ac.iter_mut() {
            *byte = (val & 0xFF) as u8;
            val = val >> 8;
        }
    }
);

benchmark!(
    zip_chunks_exact_unknown_size,
    [u8],
    [u16],
    |output: &mut [u8], input: &[u16]| for (b, ac) in input.iter().zip(output.chunks_exact_mut(2)) {
        ac[0] = (*b & 0xff) as u8;
        ac[1] = ((*b >> 8) & 0xff) as u8;
    }
);

benchmark!(
    zip_chunks_exact_unknown_size_slice,
    [u8],
    [u16],
    |output: &mut [u8], input: &[u16]| {
        use std::cmp::min;
        for (b, ac) in input[..(min(320000, input.len()))]
            .iter()
            .zip(output.chunks_exact_mut(2))
        {
            ac[0] = (*b & 0xff) as u8;
            ac[1] = ((*b >> 8) & 0xff) as u8;
        }
    }
);

benchmark!(
    zip_chunks_exact_unknown_size_take,
    [u8],
    [u16],
    |output: &mut [u8], input: &[u16]| for (b, ac) in
        input.iter().zip(output.chunks_exact_mut(2)).take(320000)
    {
        ac[0] = (*b & 0xff) as u8;
        ac[1] = ((*b >> 8) & 0xff) as u8;
    }
);

benchmark!(
    zip_chunks_exact_unknown_size_take_iter,
    [u8],
    [u16],
    |output: &mut [u8], input: &[u16]| for (&b, ac) in
        input.iter().zip(output.chunks_exact_mut(2)).take(320000)
    {
        let mut val = b;
        for byte in ac.iter_mut() {
            *byte = (val & 0xFF) as u8;
            val = val >> 8;
        }
    }
);
