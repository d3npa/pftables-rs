//! This module provides Rust-friendly wrappers for the `repr(C)` structs 
//! used by OpenBSD's pf.
#[cfg(test)] mod tests;
pub mod bindings;
use bindings::*;

use std::{fmt, io};
use std::fs::File;
use std::cell::RefCell;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::os::unix::io::AsRawFd;

use crate::PfError;
type PfResult<T> = Result<T, PfError>;

/// Represents a Rust-friendly struct with an associated `repr(C)` equivalent
pub trait Translate<T> {
    /// Generates a `repr(C)` struct equivalent to the implementor
    fn translate(self: &Self) -> PfResult<T>;
    /// Consumes a `repr(C)` struct to update the implementor's internal values
    fn update(self: &mut Self, c: T) -> PfResult<()>;
}

/// A Rust-friendly wrapper `#[repr(C)] struct pfr_addr`
#[derive(Debug, Clone, PartialEq)]
pub struct PfrAddr {
    pub addr: IpAddr,
    pub ifname: String,
    pub subnet: u8,
    // Other fields are unused right now
}

impl PfrAddr {
    /// Initializes a PfrAddr representing the host `127.0.0.1/32`
    pub fn new() -> PfrAddr {
        PfrAddr {
            addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            ifname: String::new(),
            subnet: 32,
        }
    }

    /// Constructs a `PfrAddr` from an IP and subnet
    pub fn from_addr(addr: IpAddr, subnet: u8) -> PfrAddr {
        PfrAddr {
            addr,
            subnet,
            ifname: String::new(),
        }
    }
}

impl Translate<pfr_addr> for PfrAddr {
    /// Will fail if address family or ifname contain invalid data
    fn update(&mut self, c: pfr_addr) -> PfResult<()> {
        let addr = match c.pfra_af {
            AF_INET => {
                let v = unsafe { c.pfra_u._pfra_ip4addr };
                IpAddr::V4(Ipv4Addr::from(u32::from_be(v)))
            },
            AF_INET6 => {
                let v = unsafe { c.pfra_u._pfra_ip6addr };
                IpAddr::V6(Ipv6Addr::from(u128::from_be_bytes(v)))
            },
            _ => {
                return Err(PfError::UnknownAddressFamily)
            },
        };

        let mut ifname = c.pfra_ifname.to_vec();
        while ifname.len() > 0 && ifname[ifname.len() - 1] == 0 {
            ifname.pop();
        }

        let ifname = match String::from_utf8(ifname) {
            Ok(v) => v,
            Err(_) => return Err(PfError::TranslationError),
        };

        // No errors; Ok to update
        self.addr = addr;
        self.ifname = ifname;
        self.subnet = c.pfra_net;

        Ok(())
    }
    
    /// Fails if length of ifname is longer than or equal to IFNAMSIZ
    fn translate(&self) -> PfResult<pfr_addr> {
        if self.ifname.len() >= bindings::IFNAMSIZ {
            return Err(PfError::TranslationError);
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

/// A Rust-friendly wrapper to `#[repr(C)] pfr_table`
#[derive(Debug, PartialEq)]
pub struct PfrTable {
    pub anchor: String,
    pub name: String,
    // Fields below are unused at the moment
    // pub flags: u32,
    // pub fback: u8,
}

impl PfrTable {
    /// Constructs a `PfrTable` with empty values
    pub fn new() -> PfrTable {
        PfrTable {
            anchor: String::new(),
            name: String::new(),
        }
    }
}

impl Translate<pfr_table> for PfrTable {
    /// Fails if strings in C structure contain invalid UTF-8
    fn update(&mut self, c: pfr_table) -> PfResult<()> {
        let mut anchor = c.pfrt_anchor.to_vec();
        while anchor.len() > 0 && anchor[anchor.len() - 1] == 0 {
            anchor.pop();
        }

        let anchor = match String::from_utf8(anchor) {
            Ok(v) => v,
            Err(_) => return Err(PfError::TranslationError),
        };

        let mut name = c.pfrt_name.to_vec();
        while name.len() > 0 && name[name.len() - 1] == 0 {
            name.pop();
        }

        let name = match String::from_utf8(name) {
            Ok(v) => v,
            Err(_) => return Err(PfError::TranslationError),
        };

        // No errors; Ok to update
        self.anchor = anchor;
        self.name = name;

        Ok(())
    }
    
    /// Fails if the length of anchor is equal to or greater than PATH_MAX,
    /// or if the length of name is equal to or greater than PF_TABLE_NAME_SIZE
    fn translate(&self) -> PfResult<pfr_table> {
        if self.anchor.len() >= bindings::PATH_MAX 
            || self.name.len() >= bindings::PF_TABLE_NAME_SIZE 
        {
            return Err(PfError::TranslationError);
        }

        let mut c_table = pfr_table::init();

        // Copy bytes for the anchor path
        for i in 0..self.anchor.len() {
            c_table.pfrt_anchor[i] = self.anchor.as_bytes()[i];
        }

        // Copy bytes for the table name
        for i in 0..self.name.len() {
            c_table.pfrt_name[i] = self.name.as_bytes()[i];
        }

        Ok(c_table)
    }
}

/// A Rust-friendly wrapper to `#[repr(C)] struct pfioc_table`
pub struct PfIocTable {
    pub table: PfrTable,
    pub buffer: Vec<PfrAddr>,
    pub size: usize,
    pub added: u32,
    pub deleted: u32,
    /// Internal buffer to preserve ownership while the kernel fills in values
    pfrio_buffer: RefCell<Vec<bindings::pfr_addr>>,
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

impl PfIocTable {
    /// Constructs a `PfIocTable` with empty values
    pub fn new() -> PfIocTable {
        PfIocTable {
            table: PfrTable::new(),
            buffer: Vec::new(),
            size: 0,
            added: 0,
            deleted: 0,
            pfrio_buffer: RefCell::new(vec![]),
        }
    }

    /// Convenience function to prepare a `PfIocTable` with a `name`
    pub fn with_table(name: &str) -> PfIocTable {
        let mut io = PfIocTable::new();

        io.table = PfrTable {
            anchor: String::new(),
            name: String::from(name),
        };

        io
    }

    /// Makes an `ioctl` system call on `fd` operating on this struct
    pub fn fire(&mut self, fd: &File, cmd: PfIocCommand) -> PfResult<()> {
        let fd = fd.as_raw_fd();
        let mut io = self.translate()?;

        use PfIocCommand::*;
        // It seems only commands that use the buffer are allowed to set esize
        if let AddAddrs | DelAddrs | GetAddrs = cmd {
            io.pfrio_esize = std::mem::size_of::<pfr_addr>() as i32;
        }

        let status = unsafe {
            ioctl(fd, cmd.code()?, &mut io as *mut pfioc_table)
        };

        if status == -1 {
            return Err(PfError::IoctlError(io::Error::last_os_error()));
        }

        self.update(io)?;
        Ok(())
    }
}

impl Translate<pfioc_table> for PfIocTable {
    /// Will fail if PfrTable or PfrAddr conversions fail
    fn update(&mut self, io: pfioc_table) -> PfResult<()> {
        // replace table, replace buffer, replace size
        self.table.update(io.pfrio_table)?;
        self.buffer.clear();

        // Update self.buffer from internal pfrio_buffer
        // We use remove(0) to take ownership of the value because pfr_addr is 
        // not Copy.
        let mut internal = self.pfrio_buffer.borrow_mut();
        for _ in 0..internal.len() {
            let addr = internal.remove(0);
            // This could be reduced by implementing TryInto
            let mut addr2 = PfrAddr::new();
            addr2.update(addr)?;
            self.buffer.push(addr2);
        }

        self.size = io.pfrio_size as usize;
        self.added = io.pfrio_nadd as u32;
        self.deleted = io.pfrio_ndel as u32;

        Ok(())
    }

    /// Note: ignores internal self.size value
    fn translate(&self) -> PfResult<pfioc_table> {
        // Update self.pfrio_buffer to match self.buffer
        let mut internal = self.pfrio_buffer.borrow_mut();
        internal.clear();
        for addr in &self.buffer {
            internal.push(addr.translate()?);
        }

        let mut io = pfioc_table::init();
        io.pfrio_table = self.table.translate()?;
        io.pfrio_buffer = internal.as_mut_ptr();
        io.pfrio_size = internal.len() as i32;

        Ok(io)
    }
}

impl fmt::Debug for PfIocTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Skip non-Debug self.pfrio_buffer
        f.debug_struct("PfIocTable")
            .field("table", &self.table)
            .field("buffer", &self.buffer)
            .field("size", &self.size)
            .field("added", &self.added)
            .field("deleted", &self.deleted)
            .finish()
    }
}

impl PartialEq for PfIocTable {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
        && self.buffer == other.buffer
    }
}

pub enum PfIocCommand {
    ClrAddrs,
    AddAddrs,
    DelAddrs,
    GetAddrs,
}

impl PfIocCommand {
    /// Converts this enum into a numeric value the kernel recognizes
    fn code(&self) -> PfResult<u64> {
        #[allow(unreachable_patterns)]
        match &self {
            PfIocCommand::ClrAddrs => Ok(bindings::DIOCRCLRADDRS),
            PfIocCommand::AddAddrs => Ok(bindings::DIOCRADDADDRS),
            PfIocCommand::DelAddrs => Ok(bindings::DIOCRDELADDRS),
            PfIocCommand::GetAddrs => Ok(bindings::DIOCRGETADDRS),
            _ => { return Err(PfError::Unimplemented) },
        }
    }
}