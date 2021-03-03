use std::fs;
use std::error::Error;
use std::net::IpAddr;
use pf_rs::*;

fn get_addrs(fd: &fs::File, table_name: &str)
    -> Result<Vec<IpAddr>, Box<dyn Error>> 
{
    // Prepare an Ioctl call
    let mut io = PfIocTable::with_table(table_name);

    // Ask the kernel how many entries there are
    io.fire(&fd, PfIocCommand::GetAddrs)?;

    // Allocate room for number of entries based on returned size
    io.buffer = vec![PfrAddr::new(); io.size];
    io.fire(&fd, PfIocCommand::GetAddrs)?;

    // Extract addresses
    let addrs = io.buffer.iter()
        .map(|x| x.addr)
        .collect();
    
    Ok(addrs)
}

fn add_addrs(fd: &fs::File, table_name: &str, addrs: Vec<IpAddr>)
    -> Result<u32, Box<dyn Error>>
{
    let addrs = addrs.into_iter().map(move |x| {
            let mut a = PfrAddr::new();
            a.addr = x;
            a
        }).collect();
    
    let mut io = PfIocTable::with_table(table_name);
    io.buffer = addrs;
    io.fire(&fd, PfIocCommand::AddAddrs)?;

    Ok(io.added)
}

fn main() -> Result<(), Box<dyn Error>> {
    let fd = fs::OpenOptions::new()
        .write(true)
        .open("/dev/pf")?;

    // Add an address
    let added = add_addrs(&fd, "my_table", vec![
        IpAddr::V4("10.0.0.2".parse()?)
    ])?;
    println!("Added: {}", added);

    // Get addresses
    println!("{:?}", get_addrs(&fd, "my_table")?);


    Ok(())
}