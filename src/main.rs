use pf_rs::*;
use std::error::Error;
use std::net::IpAddr::{V4, V6};

fn main() -> Result<(), Box<dyn Error>> {
    let table = PfTable::new("my_table");
    let addrs = table.get_addrs()?;

    for addr in addrs {
        match addr {
            V4(a) => { println!("Ipv4: {}", a); },
            V6(a) => { println!("Ipv6: {}", a); },
        }
    }

    Ok(())
}