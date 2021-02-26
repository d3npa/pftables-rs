use std::fs;
use std::error::Error;
use std::os::unix::io::IntoRawFd;
use pf_rs::*;

extern "C" {
    fn strlcpy(dst: *mut u8, src: *const u8, dstsize: usize) -> usize;
}

fn main() -> Result<(), Box<dyn Error>> {
    let dev = fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open("/dev/pf")?
        .into_raw_fd();

    println!("Got fd to /dev/pf: {}", dev);

    println!("Size of pfr_addr: {}", std::mem::size_of::<pfr_addr>());
    println!("Size of pfr_table: {}", std::mem::size_of::<pfr_table>());
    println!("Size of pfioc_table: {}", std::mem::size_of::<pfioc_table>());

    let mut table = pfr_table::init();

    unsafe {
        strlcpy(
            table.pfrt_name.as_mut_ptr(), 
            b"my_table\0".as_ptr(), 
            PF_TABLE_NAME_SIZE
        );
    }

    println!("{:?}", table.pfrt_name);

    Ok(())
}