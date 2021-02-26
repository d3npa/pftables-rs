pub const PATH_MAX: usize = 1024;
pub const IFNAMSIZ: usize = 16;
pub const INET_ADDRSTRLEN: usize = 16;
pub const PF_TABLE_NAME_SIZE: usize = 32;
pub const DIOCRSETADDRS: usize = 3293594693;
pub const DIOCRGETADDRS: usize = 3293594694;
use libc::{in_addr, in6_addr};
use std::mem;

#[repr(C)]
pub union pfr_addr_u {
    pub _pfra_ip4addr: in_addr,
    pub _pfra_ip6addr: in6_addr,
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
    pub fn init() -> pfr_addr {
        let buffer = [0u8; 52];
        unsafe {
            mem::transmute::<[u8; 52], pfr_addr>(buffer)
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
    pub fn init() -> pfr_table {
        let buffer = [0u8; 1064];
        unsafe {
            mem::transmute::<[u8; 1064], pfr_table>(buffer)
        }
    }
}

#[repr(C)]
pub struct pfioc_table {
    pub pfrio_table: pfr_table,
    pub pfrio_buffer: *mut [pfr_addr],
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
    pub fn init() -> pfioc_table {
        let buffer = [0u8; 1112];
        unsafe {
            mem::transmute::<[u8; 1112], pfioc_table>(buffer)
        }
    }
}