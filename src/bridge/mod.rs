#[cfg(test)] mod tests;
pub mod bindings;
use bindings::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::convert::{TryFrom, TryInto};
use crate::PfError;

// Create more Rust-friendly (and safer) versions of the pf structs
#[derive(Debug, PartialEq)]
pub struct PfrAddr {
    pub addr: IpAddr,
    pub ifname: String,
    pub subnet: u8,
    // Other fields are unused right now
}

impl PfrAddr {
    pub fn new() -> PfrAddr {
        PfrAddr {
            addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            ifname: String::new(),
            subnet: 32,
        }
    }
}

impl TryFrom<pfr_addr> for PfrAddr {
    type Error = crate::PfError;
    /// Will fail if pfra_af field is invalid or if pfra_ifname contains invalid utf8
    fn try_from(a: pfr_addr) -> Result<PfrAddr, PfError> {
        let addr = match a.pfra_af {
            AF_INET => {
                let v = unsafe { a.pfra_u._pfra_ip4addr };
                IpAddr::V4(Ipv4Addr::from(u32::from_be(v)))
            },
            AF_INET6 => {
                let v = unsafe { a.pfra_u._pfra_ip6addr };
                IpAddr::V6(Ipv6Addr::from(u128::from_be_bytes(v)))
            },
            _ => {
                return Err(PfError::UnknownAddressFamily)
            },
        };

        let mut ifname = a.pfra_ifname.to_vec();
        while ifname.len() > 0 && ifname[ifname.len() - 1] == 0 {
            ifname.pop();
        }

        let ifname = match String::from_utf8(ifname) {
            Ok(v) => v,
            Err(_) => return Err(PfError::ConversionError),
        };

        Ok(PfrAddr {
            addr,
            ifname,
            subnet: a.pfra_net,
        })
    }
}

impl TryInto<pfr_addr> for PfrAddr {
    type Error = crate::PfError;
    /// Will fail if ifname is IFNAMSIZ characters or longer
    fn try_into(self) -> Result<pfr_addr, PfError> {
        if self.ifname.len() >= IFNAMSIZ {
            return Err(PfError::ConversionError);
        }

        let mut c_addr = pfr_addr::init();
        c_addr.pfra_net = self.subnet;
        
        match self.addr {
            IpAddr::V4(v) => {
                c_addr.pfra_af = AF_INET;
                c_addr.pfra_u._pfra_ip4addr = u32::to_be(v.into());
            },
            IpAddr::V6(v) => {
                c_addr.pfra_af = AF_INET6;
                c_addr.pfra_u._pfra_ip6addr = u128::from(v).to_be_bytes();
            },
        };

        for i in 0..self.ifname.len() {
            c_addr.pfra_ifname[i] = self.ifname.as_bytes()[i] as u8;
        }

        Ok(c_addr)
    }
}

#[derive(Debug, PartialEq)]
pub struct PfrTable {
    pub anchor: String,
    pub name: String,
    // Fields below are unused at the moment
    // pub flags: u32,
    // pub fback: u8,
}

impl PfrTable {
    pub fn new() -> PfrTable {
        PfrTable {
            anchor: String::new(),
            name: String::new(),
        }
    }
}

impl TryFrom<pfr_table> for PfrTable {
    type Error = crate::PfError;
    /// Will fail if pfrt_anchor or pfrt_name contain invalid utf8
    fn try_from(t: pfr_table) -> Result<PfrTable, PfError> {
        let mut anchor = t.pfrt_anchor.to_vec();
        while anchor.len() > 0 && anchor[anchor.len() - 1] == 0 {
            anchor.pop();
        }

        let anchor = match String::from_utf8(anchor) {
            Ok(v) => v,
            Err(_) => return Err(PfError::ConversionError),
        };

        let mut name = t.pfrt_name.to_vec();
        while name.len() > 0 && name[name.len() - 1] == 0 {
            name.pop();
        }

        let name = match String::from_utf8(name) {
            Ok(v) => v,
            Err(_) => return Err(PfError::ConversionError),
        };

        Ok(PfrTable { anchor, name })
    }
}

impl TryInto<pfr_table> for PfrTable {
    type Error = crate::PfError;
    /// Will fail if anchor is PATH_MAX or greater, of if name is PF_TABLE_NAME_SIZE or greater
    fn try_into(self) -> Result<pfr_table, PfError> {
        if self.anchor.len() >= PATH_MAX 
            || self.name.len() >= PF_TABLE_NAME_SIZE 
        {
            return Err(PfError::ConversionError);
        }

        let mut c_table = pfr_table::init();

        for i in 0..self.anchor.len() {
            c_table.pfrt_anchor[i] = self.anchor.as_bytes()[i];
        }

        for i in 0..self.name.len() {
            c_table.pfrt_name[i] = self.name.as_bytes()[i];
        }

        Ok(c_table)
    }
}

#[derive(Debug, PartialEq)]
pub struct PfiocTable {
    pub table: PfrTable,
    pub buffer: Vec<PfrAddr>,
    // pub esize: i32, // len of pfr_addr... maybe impl a get_size() on PfrAddr?
    // pub size: i32, // len of buffer can be infered
    // Below fields are currently unused
    // pub size2: i32,
    // pub added: i32,
    // pub deleted: i32,
    // pub changed: i32,
    // pub flags: i32,
    // pub ticket: u32,
}

impl PfiocTable {
    pub fn new() -> PfiocTable {
        PfiocTable {
            table: PfrTable::new(),
            buffer: Vec::new(),
        }
    }
}

impl TryFrom<PfiocTableInter> for PfiocTable {
    type Error = crate::PfError;

    /// Will fail if PfrTable or PfrAddr conversions fail
    fn try_from(io: PfiocTableInter) -> Result<PfiocTable, PfError> {
        let PfiocTableInter { io, addrs } = io;

        let mut addrs2: Vec<PfrAddr> = vec![];
        for addr in addrs {
            addrs2.push(addr.try_into()?);
        }

        Ok(PfiocTable {
            table: io.pfrio_table.try_into()?,
            buffer: addrs2,
        })
    }
}

impl TryInto<PfiocTableInter> for PfiocTable {
    type Error = crate::PfError;
    fn try_into(self) -> Result<PfiocTableInter, PfError> {
        let mut addrs: Vec<pfr_addr> = vec![];
        for addr in self.buffer {
            addrs.push(addr.try_into()?);
        }

        let mut io = pfioc_table::init();
        io.pfrio_table = self.table.try_into()?;
        io.pfrio_buffer = addrs.as_mut_ptr();
        io.pfrio_esize = PFR_ADDR_SIZE as i32;
        io.pfrio_size = addrs.len() as i32;

        Ok(PfiocTableInter { io, addrs })
    }
}

/// Intermediate type to retain ownership of pfrio_buffer upon conversion to/from pfioc_table
pub struct PfiocTableInter {
    pub io: pfioc_table,
    pub addrs: Vec<pfr_addr>,
}