#![allow(unused_imports)]
use pf_rs::*;
use pf_rs::bindings::*;
use std::error::Error;
use std::fs;
use std::net::IpAddr::{V4, V6};
use std::os::unix::io::IntoRawFd;
use std::convert::{TryInto, TryFrom};

extern "C" {
    fn ioctl(d: i32, request: u64, ...) -> i32;
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut io = PfiocTable::new();
    io.table = PfrTable::new();
    io.table.name = String::from("my_table");

    let fd = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/pf")?
        .into_raw_fd();

    let PfiocTableInter { mut io, addrs } = io.try_into()?;

    unsafe {
        ioctl(fd, DIOCRGETADDRS, &mut io as *mut pfioc_table);
    }

    println!("Reported size: {}", io.pfrio_size);

    let mut io = PfiocTable::try_from(PfiocTableInter { io, addrs })?;

    for _ in 0..io.buffer.capacity() {
        io.buffer.push(PfrAddr::new());
    }

    let PfiocTableInter { mut io, addrs } = io.try_into()?;

    unsafe {
        ioctl(fd, DIOCRGETADDRS, &mut io as *mut pfioc_table);
    }

    let io = PfiocTable::try_from(PfiocTableInter { io, addrs })?;
    println!("{:?}", io);

    Ok(())
}