use std::{fs, io, mem};
use std::error::Error;
use std::os::unix::io::IntoRawFd;
use std::ffi::CString;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::convert::{From, Into};

pub const PATH_MAX: usize = 1024;
pub const IFNAMSIZ: usize = 16;
pub const INET_ADDRSTRLEN: usize = 16;
pub const PF_TABLE_NAME_SIZE: usize = 32;
pub const DIOCRSETADDRS: u64 = 3293594693;
pub const DIOCRGETADDRS: u64 = 3293594694;

pub const AF_INET: u8 = 2;
pub const AF_INET6: u8 = 24;

pub const PFR_ADDR_SIZE: usize = 52;
pub const PFR_TABLE_SIZE: usize = 1064;
pub const PFIOC_TABLE_SIZE: usize = 1104;

extern "C" {
    fn strlcpy(dst: *mut u8, src: *const u8, dstsize: usize) -> usize;
    fn ioctl(d: i32, request: u64, ...) -> i32;
    fn close(d: i32);
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union pfr_addr_u {
    pub _pfra_ip4addr: u32,
    pub _pfra_ip6addr: [u8; 16],
}

#[repr(C)]
#[derive(Clone)]
pub struct pfr_addr {
	pub pfra_u: pfr_addr_u,
    pub pfra_ifname: [u8; IFNAMSIZ],
    pub pfra_states: u32,
    pub pfra_weight: u16,
    pub pfra_af: u8,
    pub pfra_net: u8,
    pub pfra_not: u8,
    pub pfra_fback: u8,
    pub pfra_type: u8,
    pub pad: [u8; 7],
}

impl pfr_addr {
    /// Creates an empty pfr_addr where every field is null
    pub fn init() -> pfr_addr {
        let buffer = [0u8; PFR_ADDR_SIZE];
        unsafe {
            mem::transmute::<[u8; PFR_ADDR_SIZE], pfr_addr>(buffer)
        }
    }
}

impl Into<IpAddr> for pfr_addr {
    fn into(self) -> IpAddr {
        if self.pfra_af == AF_INET {
            let v = unsafe { self.pfra_u._pfra_ip4addr };
            return IpAddr::V4(Ipv4Addr::from(u32::from_be(v)));
        } else if self.pfra_af == AF_INET6 {
            let v = unsafe { self.pfra_u._pfra_ip6addr };
            return IpAddr::V6(Ipv6Addr::from(u128::from_be_bytes(v)));
        } else {
            panic!("Invalid Address Family")
        }
    }
}

impl From<IpAddr> for pfr_addr {
    fn from(ip: IpAddr) -> pfr_addr {
        let mut addr = Self::init();
        match ip {
            IpAddr::V4(v) => {
                addr.pfra_af = AF_INET;
                addr.pfra_u._pfra_ip4addr = u32::to_le(v.into());
            },
            IpAddr::V6(v) => {
                addr.pfra_af = AF_INET6;
                addr.pfra_u._pfra_ip6addr = u128::from(v).to_be_bytes();
            },
        }
        addr
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct pfr_table {
    pub pfrt_anchor: [u8; PATH_MAX],
    pub pfrt_name: [u8; PF_TABLE_NAME_SIZE],
    pub pfrt_flags: u32,
    pub pfrt_fback: u8,
}

impl pfr_table {
    /// Creates an empty pfr_table where every field is null
    pub fn init() -> pfr_table {
        let buffer = [0u8; PFR_TABLE_SIZE];
        unsafe {
            mem::transmute::<[u8; PFR_TABLE_SIZE], pfr_table>(buffer)
        }
    }

    pub fn new(name: &str) -> pfr_table {
        let mut table = Self::init();
        let name = CString::new(name).unwrap();
        unsafe {
            strlcpy(
                table.pfrt_name.as_mut_ptr(), 
                name.as_bytes().as_ptr(), 
                PF_TABLE_NAME_SIZE
            );
        }
        table
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct pfioc_table {
    pub pfrio_table: pfr_table,
    pub pfrio_buffer: *mut pfr_addr,
    pub pfrio_esize: i32,
    pub pfrio_size: i32,
    pub pfrio_size2: i32,
    pub pfrio_nadd: i32,
    pub pfrio_ndel: i32,
    pub pfrio_nchange: i32,
    pub pfrio_flags: i32,
    pub pfrio_ticket: u32,
}

impl pfioc_table {
    /// Creates an empty pfioc_table where every field is null
    pub fn init() -> pfioc_table {
        let buffer = [0u8; PFIOC_TABLE_SIZE];
        unsafe {
            mem::transmute::<[u8; PFIOC_TABLE_SIZE], pfioc_table>(buffer)
        }
    }

    /// Opens `/dev/pf` and calls ioctl on it.
    /// Fails if the ioctl syscall status is -1. 
    pub fn fire(&mut self, op: u64) -> Result<(), Box<dyn Error>> {
        let fd = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/pf")?
            .into_raw_fd();

        let status = unsafe {
            let status = ioctl(fd, op, self as *mut _);
            close(fd);
            status
        };

        if status == -1 {
            return Err(Box::new(io::Error::last_os_error()));
        }

        Ok(())
    }
}