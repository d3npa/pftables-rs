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

fn del_addrs(fd: &fs::File, table_name: &str, addrs: Vec<IpAddr>)
    -> Result<u32, Box<dyn Error>>
{
    let addrs = addrs.into_iter().map(move |x| {
            let mut a = PfrAddr::new();
            a.addr = x;
            a
        }).collect();
    
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

    let addrs = vec![
        IpAddr::V4("10.0.0.2".parse()?)
        ];
        
    // Add addresses
    let added = add_addrs(&fd, "my_table", addrs.clone())?;
    println!("Added: {}", added);
    
    // Get addresses
    println!("{:?}", get_addrs(&fd, "my_table")?);

    // Del addresses
    let deleted = del_addrs(&fd, "my_table", addrs.clone())?;
    println!("Deleted: {}", deleted);

    // Get addresses
    println!("{:?}", get_addrs(&fd, "my_table")?);

    // Clear addresses
    let cleared = clr_addrs(&fd, "my_table")?;
    println!("Cleared: {}", cleared);
    /*  
        check /dev/pf kernel src to try and see why ioctl is failing
        could it be another field in pfioc_table?
            97310 pf-rs    CALL  ioctl(3,3293594690,140187732460992)
            97310 pf-rs    RET   ioctl -1 errno 19
        for comparison, pfctl does:
            61064 pfctl    CALL  ioctl(3,3293594690,140187732341936)
            61064 pfctl    RET   ioctl 0
        which is the same. o-o
    */

    Ok(())
}