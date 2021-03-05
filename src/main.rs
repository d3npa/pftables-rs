use std::fs;
use std::error::Error;
use std::net::IpAddr;
use std::convert::{From, Into};
use pf_rs::*;

#[derive(Debug, Clone)]
/// Represents an Ip Address with a cidr
struct IpAddress {
    addr: IpAddr,
    cidr: u8,
}

impl IpAddress {
    pub fn ipv4(addr: &str, cidr: u8) -> Result<IpAddress, Box<dyn Error>> {
        let addr = IpAddr::V4(addr.parse()?);
        Ok(IpAddress { addr, cidr })
    }

    pub fn ipv6(addr: &str, cidr: u8) -> Result<IpAddress, Box<dyn Error>> {
        let addr = IpAddr::V6(addr.parse()?);
        Ok(IpAddress { addr, cidr })
    }
}

impl From<PfrAddr> for IpAddress {
    fn from(a: PfrAddr) -> IpAddress {
        IpAddress { 
            addr: a.addr,
            cidr: a.subnet
        }
    }
}

impl Into<PfrAddr> for IpAddress {
    fn into(self) -> PfrAddr {
        PfrAddr { 
            addr: self.addr,
            subnet: self.cidr,
            ifname: String::new(),
        }
    }
}

fn get_addrs(fd: &fs::File, table_name: &str)
    -> Result<Vec<IpAddress>, Box<dyn Error>> 
{
    // Prepare an Ioctl call
    let mut io = PfIocTable::with_table(table_name);

    // Ask the kernel how many entries there are
    io.fire(&fd, PfIocCommand::GetAddrs)?;

    // Allocate room for number of entries based on returned size
    io.buffer = vec![PfrAddr::new(); io.size];
    io.fire(&fd, PfIocCommand::GetAddrs)?;

    // Extract addresses
    let addrs = io.buffer.into_iter()
        .map(move |x| x.into())
        .collect();
    
    Ok(addrs)
}

fn add_addrs(fd: &fs::File, table_name: &str, addrs: Vec<IpAddress>)
    -> Result<u32, Box<dyn Error>>
{
    let addrs: Vec<PfrAddr> = addrs.into_iter()
        .map(move |x| x.into())
        .collect();
    
    let mut io = PfIocTable::with_table(table_name);
    io.buffer = addrs;
    io.fire(&fd, PfIocCommand::AddAddrs)?;

    Ok(io.added)
}

fn del_addrs(fd: &fs::File, table_name: &str, addrs: Vec<IpAddress>)
    -> Result<u32, Box<dyn Error>>
{
    let addrs = addrs.into_iter()
        .map(move |x| x.into())
        .collect();
    
    let mut io = PfIocTable::with_table(table_name);
    io.buffer = addrs;
    io.fire(&fd, PfIocCommand::DelAddrs)?;

    Ok(io.deleted)
}

fn clr_addrs(fd: &fs::File, table_name: &str)
    -> Result<u32, Box<dyn Error>>
{
    let mut io = PfIocTable::with_table(table_name);
    io.fire(&fd, PfIocCommand::ClrAddrs)?;

    Ok(io.deleted)
}

fn main() -> Result<(), Box<dyn Error>> {
    let fd = fs::OpenOptions::new()
        .write(true)
        .open("/dev/pf")?;

    let mut addrs = vec![
        IpAddress::ipv4("127.0.0.1", 32)?,
        IpAddress::ipv4("127.0.0.2", 32)?,
        IpAddress::ipv4("127.0.0.3", 32)?,
        IpAddress::ipv6("::1", 128)?,
    ];
        
    // Add addresses
    let added = add_addrs(&fd, "my_table", addrs.clone())?;
    println!("Added: {}", added);
    
    // Get addresses
    println!("{:?}", get_addrs(&fd, "my_table")?);

    // Del some of the addresses
    addrs.pop();
    let deleted = del_addrs(&fd, "my_table", addrs.clone())?;
    println!("Deleted: {}", deleted);

    // Get addresses
    println!("{:?}", get_addrs(&fd, "my_table")?);

    // Clear addresses
    let cleared = clr_addrs(&fd, "my_table")?;
    println!("Cleared: {}", cleared);

    // Get addresses
    println!("{:?}", get_addrs(&fd, "my_table")?);

    Ok(())
}