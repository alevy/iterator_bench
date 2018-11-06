#![feature(test)]

extern crate rand;
extern crate test;

use std::time::SystemTime;
use test::{Bencher, black_box};

fn main() {
    let mut app_buf: [u8; 640000] = [0; 640000];
    let mut other_buf: [u16; 320000] = [0; 320000];
    for i in 0..320000 {
         other_buf[i] = rand::random();
    }
    let mut start = SystemTime::now();
    raw_impl_slices(&mut app_buf, &other_buf);
    let raw = start.elapsed();

    start = SystemTime::now();
    zip_impl(&mut app_buf, &other_buf);
    let zip = start.elapsed();

    println!("Raw: {:?}", raw);
    println!("Zip: {:?}", zip);
}

macro_rules! conv {
    ( $name:ident, $impl:expr ) => {
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

            fn _impl(output: &mut [u8; 640000], input: &[u16; 320000]) {
                $impl(output, input)
            }
        }
    }
}

conv!(c_style_fixed_size, |output: &mut [u8; 640000], input: &[u16; 320000]| {
    for i in 0..input.len() {
        let b = input[i];
        output[2 * i] = (b & 0xff) as u8;
        output[2 * i + 1] = ((b >> 8) & 0xff) as u8;
   }
});

#[test]
fn test_raw() {
    let mut app_buf: [u8; 640000] = [0; 640000];
    let mut other_buf: [u16; 320000] = [0; 320000];
    for c in other_buf.iter_mut() {
        *c = 0xffaa;
    }

    raw_impl(&mut app_buf, &other_buf);

    for (i, c) in app_buf.iter().enumerate() {
        if i % 2 == 0 {
          assert_eq!(*c, 0xaa);
        } else {
          assert_eq!(*c, 0xff);
        }
    }
}

#[test]
fn test_zip() {
    let mut app_buf: [u8; 640000] = [0; 640000];
    let mut other_buf: [u16; 320000] = [0; 320000];
    for c in other_buf.iter_mut() {
        *c = 0xffaa;
    }

    zip_impl(&mut app_buf, &other_buf);

    for (i, c) in app_buf.iter().enumerate() {
        if i % 2 == 0 {
          assert_eq!(*c, 0xaa);
        } else {
          assert_eq!(*c, 0xff);
        }
    }
}

#[bench]
fn bench_raw(bencher: &mut Bencher) {
    let mut app_buf: [u8; 640000] = [0; 640000];
    let mut other_buf: [u16; 320000] = [0; 320000];
    for i in 0..320000 {
         other_buf[i] = rand::random();
    }

    bencher.iter(|| {
      black_box(raw_impl(&mut app_buf, &other_buf));
    });
}

#[bench]
fn bench_raw_slice(bencher: &mut Bencher) {
    let mut app_buf: [u8; 640000] = [0; 640000];
    let mut other_buf: [u16; 320000] = [0; 320000];
    for i in 0..320000 {
         other_buf[i] = rand::random();
    }

    bencher.iter(|| {
      black_box(raw_impl_slices(&mut app_buf, &other_buf));
    });
}

#[bench]
fn bench_zip(bencher: &mut Bencher) {
    let mut app_buf: [u8; 640000] = [0; 640000];
    let mut other_buf: [u16; 320000] = [0; 320000];
    for i in 0..320000 {
         other_buf[i] = rand::random();
    }

    bencher.iter(|| {
      black_box(zip_impl(&mut app_buf, &other_buf));
    });
}

#[bench]
fn bench_zip_slice(bencher: &mut Bencher) {
    let mut app_buf: [u8; 640000] = [0; 640000];
    let mut other_buf: [u16; 320000] = [0; 320000];
    for i in 0..320000 {
         other_buf[i] = rand::random();
    }

    bencher.iter(|| {
      black_box(zip_impl_slices(&mut app_buf, &other_buf));
    });
}

#[inline(never)]
fn raw_impl(app_buf: &mut [u8; 640000], other_buf: &[u16; 320000]) {
   for (i, b) in other_buf.iter().enumerate() {
     app_buf[2 * i] = (*b & 0xff) as u8;
     app_buf[2 * i + 1] = ((*b >> 8) & 0xff) as u8;
   }
}

#[inline(never)]
fn zip_impl(app_buf: &mut [u8; 640000], other_buf: &[u16; 320000]) {
    for (&b, ac) in other_buf.iter().zip(app_buf.chunks_mut(2)) {
        // the type of `ac` is `[u8]`, but the optimizer should be able to figure out that it's
        // always size 2, and thus should elide the bounds checks in the inner loop.
        ac[0] = (b & 0xff) as u8;
        ac[1] = ((b >> 8) & 0xff) as u8;
    }
}

#[inline(never)]
fn raw_impl_1slice(app_buf: &mut [u8], other_buf: &[u16; 320000]) {
    if app_buf.len() != other_buf.len() * 2 {
        panic!("Incorrect sizes");
    }
   for (i, b) in other_buf.iter().enumerate() {
       // the optimizer can elide the bounds checks since the pre-loop if-statement actually tells
       // it the exact size of `app_buf`.
     app_buf[2 * i] = (*b & 0xff) as u8;
     app_buf[2 * i + 1] = ((*b >> 8) & 0xff) as u8;
   }
}

#[inline(never)]
fn raw_impl_slices(app_buf: &mut [u8], other_buf: &[u16]) {
    if app_buf.len() != other_buf.len() * 2 {
        panic!("Incorrect sizes");
    }
   for (i, b) in other_buf.iter().enumerate() {
       // since the optimizer has no idea what the size of `other_buf` is, it has no way of
       // reasoning about whether these slice indexes are safe, so it cannot elide the
       // bounds-checks in the inner loop.
     app_buf[2 * i] = (*b & 0xff) as u8;
     app_buf[2 * i + 1] = ((*b >> 8) & 0xff) as u8;
   }
}

#[inline(never)]
fn zip_impl_slices(app_buf: &mut [u8], other_buf: &[u16]) {
   for (b, ac) in other_buf.iter().zip(app_buf.chunks_exact_mut(2)) {
     ac[0] = (*b & 0xff) as u8;
     ac[1] = ((*b >> 8) & 0xff) as u8;
   }
}
