/// These structs are unsafe. 
/// Particularly, PfIocTable contains a raw mutable pointer to an array or 
/// vector which needs to be updated if said vector changes address (such 
/// as when it grows)
use std::mem;

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
    pub fn ioctl(d: i32, request: u64, ...) -> i32;
}

#[repr(C)]
pub union pfr_addr_u {
    pub _pfra_ip4addr: u32,
    pub _pfra_ip6addr: [u8; 16],
}

#[repr(C)]
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

#[repr(C)]
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
}

#[repr(C)]
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
}