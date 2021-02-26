use std::fs;
use std::error::Error;
use std::os::unix::io::IntoRawFd;
use pf_rs::*;

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
    println!("{}", io.pfrio_size);

    Ok(())
}