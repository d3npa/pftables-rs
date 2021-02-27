pub mod bridge;

pub use bridge::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::error::Error;

#[derive(Debug)]
pub struct PfTable {
    name: String,
}

impl PfTable {
    pub fn new(name: &str) -> PfTable {
        let name = String::from(name);
        PfTable { name }
    }

    pub fn get_addrs(&self) -> Result<Vec<IpAddr>, Box<dyn Error>> {
        let mut io = bridge::pfioc_table::init();
        io.pfrio_table = bridge::pfr_table::new(&self.name[..]);
        io.pfrio_buffer = 0 as *mut _; // Actually unused for the first call
        io.pfrio_esize = bridge::PFR_ADDR_SIZE as i32;
        io.pfrio_size = 0; // Ask the kernel how many entries there are
        io.fire(bridge::DIOCRGETADDRS)?;

        let len = io.pfrio_size;
        let mut addrs = vec![bridge::pfr_addr::init(); len as usize];
        io.pfrio_buffer = addrs.as_mut_ptr();
        io.fire(bridge::DIOCRGETADDRS)?;

        let mut list: Vec<IpAddr> = vec![];
        for addr in addrs {
            let family = addr.pfra_af;
            if family == bridge::AF_INET {
                let v = unsafe { addr.pfra_u._pfra_ip4addr };
                let ip = IpAddr::V4(Ipv4Addr::from(u32::from_be(v)));
                list.push(ip);
            } else if family == bridge::AF_INET6 {
                let v = unsafe { addr.pfra_u._pfra_ip6addr };
                let ip = IpAddr::V6(Ipv6Addr::from(u128::from_be_bytes(v)));
                list.push(ip);
            } else {
                eprintln!("Unknown family: {}", family);
            }
        }

        Ok(list)
    }
}