#![allow(unused_imports)]
use pf_rs::*;
use pf_rs::bindings::*;
use std::error::Error;
use std::fs;
use std::net::IpAddr::{V4, V6};
use std::os::unix::io::IntoRawFd;
use std::convert::{TryInto, TryFrom};

fn main() -> Result<(), Box<dyn Error>> {
    let fd = fs::OpenOptions::new()
        .write(true)
        .open("/dev/pf")?;

    // Prepare an Ioctl call
    let mut io = PfIocTable::with_table("my_table");

    // Ask the kernel how many entries there are
    io.fire(&fd, PfIocCommand::GetAddrs)?;

    // Allocate room for number of entries based on returned size
    io.buffer = vec![PfrAddr::new(); io.size];
    io.fire(&fd, PfIocCommand::GetAddrs)?;

    println!("{:?}", io);
    Ok(())
}