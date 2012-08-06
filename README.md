FeMPEG
------

FeMPEG is a simple MPEG-1 Audio Layer II (MP2) decoder written in the Rust
programming language. It's designed to be a demo of the soft real-time
capabilities and safety features of Rust.

FeMPEG performs no allocations (except in format strings in case of errors).
All data is stored in constant memory or on the stack. This can be verified by
the lack of ~ and @ sigils (signifying allocation) in the codebase. Because of
this, the decoding will be uninterrupted by `malloc` and GC latency.
Additionally, FeMPEG is safe; it does not use the unsafe sublanguage of Rust
(which can be verified with the lack of the `unsafe` keyword), so the code
should be immune to buffer overruns, out-of-bounds array accesses, and other
such problems. Like all safe Rust code, FeMPEG is thread-safe, in that it
does not use shared-memory data structures.

FeMPEG itself does not use the Rust garbage collector, but if it's embedded
within a program that does and is not spawned into a separate task, then it
could incur GC pauses. However, if FeMPEG is spawned into a separate task and
scheduler, then the GC will not affect its real-time performance. The Rust GC
is per-task and does not "stop the world".

Building
--------

You'll need [rust-ao][1] and a bleeding-edge master version of the Rust
compiler. (At the time of this writing there are also some fixes on the
incoming branch which you will need, but these should be merged to master
soon.) Compile with:

    rustc -O -L ../path/to/rust-ao -o fempeg fempeg.rs

Usage
-----

Pretty simple. Just run:

    ./fempeg /path/to/file.mp2

[1]: https://github.com/pcwalton/rust-ao

