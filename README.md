# pftable-rs
A small Rust library for managing pf tables on OpenBSD.

Note: I haven't tested it but it may work on other BSDs and macOS as well.

This was primarily a learning exercise for me. I wanted to manage my pf tables from a [rocket.rs](https://rocket.rs) application, and didn't want to just proxy to `pfctl`. Instead this library talks directly to `/dev/pf`, the kernel device that controls pf (see `man 4 pf` for more info).

I'm not currently planning on supporting operations on rules or anchors. I wrote all of the code by hand and it's incredibly time consuming to create working wrappers for all the structures the kernel is expecting. 

Currently, this library supports the following operations:

- [x] Add Addresses in table
- [x] Del Addresses in table
- [x] Get Addresses in table
- [x] Clear Addresses in table
- [ ] Set Addresses in table

See `examples/my_table` for some sample high-level usage. `src/lib.rs` contains a high-level interface called `PfTable` for interacting with tables. The source of `PfTable` may be a useful reference if you want to do more than what is provided. `src/bridge/mod.rs` contains the majority of the ffi code.
