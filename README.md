# pf-rs
Rust library for interacting with /dev/pf on OpenBSD

I wrote this library because I needed it for a personal project. 

It mainly operates on pf tables, and doesn't have anything to work with rules or anchors. I'm not sure whether or not I'll implement those in the future; all the code is written by hand in a kind of unscalable way. Supporting all of /dev/pf's functionality would take more time than I'm willing to invest right now.

Currently, it supports the following operations:

- [x] Add Addresses in table
- [x] Del Addresses in table
- [x] Get Addresses in table
- [ ] Clear Addresses in table
- [ ] Set Addresses in table
- [ ] Create new table
- [ ] Delete table
