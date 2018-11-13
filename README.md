# Rust Iterator Benchmark

This repository contains a set of benchmarks that explore the overhead of
iterator-style constructs compared to more explicit C-style loop programs.

The (current) example application that all benchmarks implement is converting a
large list of `u16`s (16-bit unsigned integers) into a list of bytes (e.g. for
serializing to a network buffer).

Benchmarks explore using fixed-sized arrays, slices with various additional
iterators.

## Conclusion

The concolusion so far is that judicially used iterators perform
indistiguishably from hand-rolled C-style code.

Both C-style versions and iteration version can degrade fairly quickly with the
addition of small change that require bounds checks or prevent other
optimizations.

## Compiling & Running Benchmarks

You need a recent Rust nightly for access to the `#[bench]` feature (for
actually running performance benchmarks), as well as `Iterator`'s
`chunks_exact`. If you are using `rustup` for Rust version management (which
you should be!), the `rust-toolchain` file in this repo will take care of
downloading and installing a working version for you.

To install `rustup`, typically, run:

```bash
$ curl [rustup_url] | sh
```

Some distributions (NixOS, Homebrew) might have `rustup` available through
normal package channels.

To compile and run benchmarks:

```bash
$ cargo bench
   Compiling iterator_bench v0.1.0 (/tmp/conv)                                                                                                   
    Finished release [optimized] target(s) in 2.61s                                                                                              
     Running target/release/deps/iterator_bench-abeb45cfeb9fde62                                                                                 

running 34 tests
...
...
test zip_chunks_unknown_size::_test ... ignored
test zip_chunks_unknown_size_take::_test ... ignored
test zip_chunks_unknown_size_take_iter::_test ... ignored
test c_style_fixed_size::_bench                      ... bench:      39,992 ns/iter (+/- 2,834)
test c_style_input_size_fixed::_bench                ... bench:     339,100 ns/iter (+/- 5,938)
test c_style_output_size_fixed::_bench               ... bench:     375,613 ns/iter (+/- 44,777)
test c_style_unknown_size::_bench                    ... bench:     364,895 ns/iter (+/- 17,090)
test c_style_unknown_size_limit::_bench              ... bench:     429,027 ns/iter (+/- 55,902)
test optimal_unsafe::_bench                          ... bench:      39,969 ns/iter (+/- 571)
test zip_chunks_exact_unknown_size::_bench           ... bench:      42,043 ns/iter (+/- 4,445)
test zip_chunks_exact_unknown_size_take::_bench      ... bench:     298,930 ns/iter (+/- 9,793)
test zip_chunks_exact_unknown_size_take_iter::_bench ... bench:     299,087 ns/iter (+/- 23,908)
test zip_chunks_fixed_size::_bench                   ... bench:      41,568 ns/iter (+/- 10,350)
test zip_chunks_fixed_size_take::_bench              ... bench:      40,001 ns/iter (+/- 554)
test zip_chunks_fixed_size_take_iter::_bench         ... bench:     278,280 ns/iter (+/- 18,207)
test zip_chunks_input_size_fixed::_bench             ... bench:     467,253 ns/iter (+/- 19,558)
test zip_chunks_output_size_fixed::_bench            ... bench:      39,998 ns/iter (+/- 1,977)
test zip_chunks_unknown_size::_bench                 ... bench:     463,868 ns/iter (+/- 5,905)
test zip_chunks_unknown_size_take::_bench            ... bench:     463,912 ns/iter (+/- 12,229)
test zip_chunks_unknown_size_take_iter::_bench       ... bench:   1,299,107 ns/iter (+/- 38,696)

test result: ok. 0 passed; 0 failed; 17 ignored; 17 measured; 0 filtered out

cargo bench  19.05s user 0.19s system 109% cpu 17.641 total
```

Similarly, you can run the associated tests for each benchmark to validate they are functioning properly (most useful if you create a new benchmark, or if you find a bug and re-write the tests):

```bash
$ cargo test
```

Finally, there is a `main` function that can be useful for compiling a binary
that allows you to inspect the assembly of indivual functions. To build:

```bash
$ cargo build --release
```

The binary will end up in `target/release/iterator_bench`.

_Note: ---release is important, since cargo, by default, compiles in debug mode, without optimizations_

## Benchmark structure

The benchmarks use a common macro, `benchmark!` to define most of the machinery
around tests and benchmarking. The macro takes four arguments:

  1. An identifier which will be used as the module name for the benchmark. This
    is what will allow you to distiguish individual benchmarks in the output of
   `cargo bench`, so better to use descriptive names.

  2. A type for the "output" buffer. The output buffer will always be
     initialized as a reference to length-640,000 array of `u8`s, but this type can force an upcast
     to a more general type, such as a slice with dynamically checked bounds.

  3. A type for the "input" buffer. The input buffer will always be
     initialized as a reference to length-32,000 array of `u16`s, but this type can force an upcast
     to a more general type, such as a slice with dynamically checked bounds.

  4. An anonymous function that takes in the output and input buffers as
     arguments, and implements a conversion of the input buffer (containing
     320,000 `u16`s) to the output buffer (containing 320,000 `u8`s). The
     closure requires its arguments' types to be specified explicitly (or else
     the compiler will complain, no other reason specific to this benchmark),
     and they should match arguments (2) and (3) to the macro.

You may be thinking: aren't arguments (2) and (3) redundant with the types in
argument (4)? And you'd be right. However, if we need them as a way to
explicitly specify the argument types for the benchmark function, or else the
compiler will _always_ specialize the closure (in argument 4) to fixed-length
arrays, and comparing arrays to slices is part of the point. If some one has a
better way of doing this, please chime in!

