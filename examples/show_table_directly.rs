use std::fs;
use std::error::Error;
use std::os::unix::io::IntoRawFd;
use pf_rs::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

extern "C" {
    fn strlcpy(dst: *mut u8, src: *const u8, dstsize: usize) -> usize;
    fn ioctl(d: i32, request: u64, ...) -> i32;
}

fn main() -> Result<(), Box<dyn Error>> {
    let dev = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open("/dev/pf")?
        .into_raw_fd();

    println!("Got fd to /dev/pf: {}", dev);

    // println!("Size of pfr_addr: {}", std::mem::size_of::<pfr_addr>());
    // println!("Size of pfr_table: {}", std::mem::size_of::<pfr_table>());
    // println!("Size of pfioc_table: {}", std::mem::size_of::<pfioc_table>());

    let mut table = pfr_table::init();

    unsafe {
        strlcpy(
            table.pfrt_name.as_mut_ptr(), 
            b"my_table\0".as_ptr(), 
            PF_TABLE_NAME_SIZE
        );
    }

    let mut io = pfioc_table::init();
    io.pfrio_table = table;
    io.pfrio_buffer = 0 as *mut pfr_addr;
    io.pfrio_esize = PFR_ADDR_SIZE as i32;
    io.pfrio_size = 0;

    unsafe {
        ioctl(dev, DIOCRGETADDRS, &mut io as *mut pfioc_table);
    }

    // println!("{}", needed);
    println!("Number of addresses in table: {}", io.pfrio_size);
    let num_addr = io.pfrio_size as usize;

    let mut addr = vec![pfr_addr::init(); num_addr];
    io.pfrio_buffer = addr.as_mut_ptr();

    unsafe {
        ioctl(dev, DIOCRGETADDRS, &mut io as *mut pfioc_table);
    }

    for i in addr {
        let ip: IpAddr = unsafe {
            match i.pfra_af {
                AF_INET => {
                    let pfr_addr_u { _pfra_ip4addr: a } = i.pfra_u;
                    IpAddr::V4(Ipv4Addr::from(u32::from_be(a)))
                }, 
                AF_INET6 => {
                    let pfr_addr_u { _pfra_ip6addr: a } = i.pfra_u;
                    IpAddr::V6(Ipv6Addr::from(u128::from_be_bytes(a)))
                },
                _ => {
                    panic!("Unknown Address Format");
                }
            }
        };

        println!("{}", ip);
    }

    Ok(())
}