use std::{fs, fmt, io, mem};
use std::error::Error;
use std::os::unix::io::IntoRawFd;
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

#[derive(Debug)]
pub enum PfError {
    TableNameTooLong,
    Other(String),
}

impl fmt::Display for PfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TableNameTooLong => {
                write!(
                    f,
                    "Table names must be under {} characters",
                    PF_TABLE_NAME_SIZE,
                )
            },
            PfError::Other(s) => {
                write!(f, "{}", s)
            },
        }
    }
}

impl Error for PfError {}

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

impl fmt::Debug for pfr_addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let addr: IpAddr = self.clone().into();
        let ifname = String::from_utf8(self.pfra_ifname.to_vec()).unwrap();
        f.debug_struct("pfr_addr")
            .field("pfra_u", &addr)
            .field("pfra_ifname", &ifname.trim_end_matches('\0'))
            .field("pfra_weight", &self.pfra_weight)
            .field("pfra_net", &self.pfra_net)
            .field("pfra_not", &self.pfra_not)
            .field("pfra_fback", &self.pfra_fback)
            .field("pfra_type", &self.pfra_type)
            .field("pad", &self.pad)
            .finish()
    }
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
            panic!("Convert pfr_addr -> IpAddr: Invalid Address Family")
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

impl fmt::Debug for pfr_table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        assert!(self.pfrt_anchor[PATH_MAX - 1] == 0);
        assert!(self.pfrt_name[PF_TABLE_NAME_SIZE - 1] == 0);
        let name = String::from_utf8(self.pfrt_name.to_vec()).unwrap();
        let anchor = String::from_utf8(self.pfrt_anchor.to_vec()).unwrap();

        f.debug_struct("pfr_table")
            .field("pfrt_anchor", &anchor.trim_end_matches('\0'))
            .field("pfrt_name", &name.trim_end_matches('\0'))
            .field("pfrt_flags", &self.pfrt_flags)
            .field("pfrt_fback", &self.pfrt_fback)
            .finish()
    }
}

impl pfr_table {
    /// Creates an empty pfr_table where every field is null
    pub fn init() -> pfr_table {
        let buffer = [0u8; PFR_TABLE_SIZE];
        unsafe {
            mem::transmute::<[u8; PFR_TABLE_SIZE], pfr_table>(buffer)
        }
    }

    /// Creates a new pfr_table with a name
    pub fn new(name: &str) -> Result<pfr_table, PfError> {
        let mut table = Self::init();

        if name.len() >= PF_TABLE_NAME_SIZE {
            return Err(PfError::TableNameTooLong);
        }

        for i in 0..name.len() {
            table.pfrt_name[i] = name.as_bytes()[i];
        }

        Ok(table)
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

impl fmt::Debug for pfioc_table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let addrs = unsafe {
            Vec::from_raw_parts(
                self.pfrio_buffer,
                self.pfrio_size as usize,
                self.pfrio_size as usize,
            )
        };
                
        f.debug_struct("pfioc_table")
            .field("pfrio_table", &self.pfrio_table)
            .field("pfrio_buffer", &addrs)
            .field("pfrio_esize", &self.pfrio_esize)
            .field("pfrio_size", &self.pfrio_size)
            .field("pfrio_size2", &self.pfrio_size2)
            .field("pfrio_nadd", &self.pfrio_nadd)
            .field("pfrio_ndel", &self.pfrio_ndel)
            .field("pfrio_nchange", &self.pfrio_nchange)
            .field("pfrio_flags", &self.pfrio_flags)
            .field("pfrio_ticket", &self.pfrio_ticket)
            .finish()
    }
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