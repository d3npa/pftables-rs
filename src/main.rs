use std::fs;
use std::error::Error;
use std::net::IpAddr;
use pf_rs::*;

static GREEN: &str = "\x1b[92m";
static RESET: &str = "\x1b[0m";


fn main() -> Result<(), Box<dyn Error>> {
    let fd = fs::OpenOptions::new()
        .write(true)
        .open("/dev/pf")?;

    let mut addrs = vec![
        PfrAddr::from_addr(IpAddr::V4("127.0.0.1".parse()?), 32),
        PfrAddr::from_addr(IpAddr::V4("127.0.0.2".parse()?), 32),
        PfrAddr::from_addr(IpAddr::V4("127.0.0.3".parse()?), 32),
        PfrAddr::from_addr(IpAddr::V6("::1".parse()?), 128),
    ];

    let mut table = PfTable::new("my_table");

    println!("{}Adding {} address{} to {}{}", 
        GREEN, 
        addrs.len(), 
        if addrs.len() == 1 { "" } else { "es" }, 
        table.name, 
        RESET);
    table.add_addrs(&fd, addrs.clone())?;
    println!("{}", table);

    let delete = vec![addrs.pop().unwrap()];
    println!("{}Deleting {} address{} from {}{}", 
        GREEN, 
        delete.len(), 
        if delete.len() == 1 { "" } else { "es" }, 
        table.name, 
        RESET);
    table.del_addrs(&fd, delete)?;
    println!("{}", table);

    println!("{}Clearing {}{}", GREEN, table.name, RESET);
    table.clr_addrs(&fd)?;

    Ok(())
}