pub mod bridge;

pub use bridge::*;
use std::net::IpAddr;
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
        let mut io = pfioc_table::init();
        io.pfrio_table = pfr_table::new(&self.name[..]);
        io.pfrio_buffer = 0 as *mut _; // Actually unused for the first call
        io.pfrio_esize = PFR_ADDR_SIZE as i32;
        io.pfrio_size = 0; // Ask the kernel how many entries there are
        io.fire(DIOCRGETADDRS)?;

        let len = io.pfrio_size;
        let mut addrs = vec![pfr_addr::init(); len as usize];
        io.pfrio_buffer = addrs.as_mut_ptr();
        io.fire(DIOCRGETADDRS)?;

        let mut list: Vec<IpAddr> = vec![];
        for addr in addrs {
            let ip = addr.into();
            list.push(ip);
        }

        Ok(list)
    }
}