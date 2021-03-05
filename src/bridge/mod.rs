/// More Rust-friendly (and safer) versions of the pf structs
/// The goal of these structs are that a library user would not need
/// to use the original repr(C) structs directly unless they really wanted to.
#[cfg(test)] mod tests;
pub mod bindings;
use bindings::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::cell::RefCell;
use crate::PfError;
use std::{fmt, io};
use std::os::unix::io::AsRawFd;
use std::fs::File;

pub trait RusticBinding<T> {
    /// Generates a repr(C) struct equivalent to the implementor
    fn repr_c(self: &Self) -> Result<T, PfError>;
    /// Consumes a repr(C) struct to update the implementor's internal values
    fn sync_c(self: &mut Self, c: T) -> Result<(), PfError>;
}

#[derive(Debug, Clone, PartialEq)]
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

    pub fn from_addr(addr: IpAddr, subnet: u8) -> PfrAddr {
        PfrAddr {
            addr,
            subnet,
            ifname: String::new(),
        }
    }
}

impl RusticBinding<pfr_addr> for PfrAddr {
    /// Updates the values in this struct from a C struct
    /// Will fail if addres family or ifname contain invalid data
    fn sync_c(&mut self, c: pfr_addr) -> Result<(), PfError> {
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
            Err(_) => return Err(PfError::ConversionError),
        };

        // No errors; Ok to update
        self.addr = addr;
        self.ifname = ifname;
        self.subnet = c.pfra_net;

        Ok(())
    }
    
    /// Creates a C struct equivalent to this one
    /// Fails if length of ifname is longer than or equal to IFNAMSIZ
    fn repr_c(&self) -> Result<pfr_addr, PfError> {
        if self.ifname.len() >= bindings::IFNAMSIZ {
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

impl RusticBinding<pfr_table> for PfrTable {
    /// Fails if strings in C structure contain invalid UTF-8
    fn sync_c(&mut self, c: pfr_table) -> Result<(), PfError> {
        let mut anchor = c.pfrt_anchor.to_vec();
        while anchor.len() > 0 && anchor[anchor.len() - 1] == 0 {
            anchor.pop();
        }

        let anchor = match String::from_utf8(anchor) {
            Ok(v) => v,
            Err(_) => return Err(PfError::ConversionError),
        };

        let mut name = c.pfrt_name.to_vec();
        while name.len() > 0 && name[name.len() - 1] == 0 {
            name.pop();
        }

        let name = match String::from_utf8(name) {
            Ok(v) => v,
            Err(_) => return Err(PfError::ConversionError),
        };

        // No errors; Ok to update
        self.anchor = anchor;
        self.name = name;

        Ok(())
    }
    
    /// Fails if the length of anchor is equal to or greater than PATH_MAX,
    /// or if the length of name is equal to or greater than PF_TABLE_NAME_SIZE
    fn repr_c(&self) -> Result<pfr_table, PfError> {
        if self.anchor.len() >= bindings::PATH_MAX 
            || self.name.len() >= bindings::PF_TABLE_NAME_SIZE 
        {
            return Err(PfError::ConversionError);
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

pub struct PfIocTable {
    pub table: PfrTable,
    pub buffer: Vec<PfrAddr>,
    pub size: usize,
    pub added: u32,
    pub deleted: u32,
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

    pub fn with_table(name: &str) -> PfIocTable {
        let mut io = PfIocTable::new();

        io.table = PfrTable {
            anchor: String::new(),
            name: String::from(name),
        };

        io
    }

    pub fn fire(&mut self, fd: &File, cmd: PfIocCommand) -> Result<(), PfError> {
        let fd = fd.as_raw_fd();
        let mut io = self.repr_c()?;

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

        self.sync_c(io)?;
        Ok(())
    }
}

impl RusticBinding<pfioc_table> for PfIocTable {
    /// Will fail if PfrTable or PfrAddr conversions fail
    fn sync_c(&mut self, io: pfioc_table) -> Result<(), PfError> {
        // replace table, replace buffer, replace size
        self.table.sync_c(io.pfrio_table)?;
        self.buffer.clear();

        // Update self.buffer from internal pfrio_buffer
        // We use pop() to take ownership of the value because pfr_addr is 
        // not Copy.
        // BUG: This reverses the order of addresses!!
        let mut internal = self.pfrio_buffer.borrow_mut();
        for _ in 0..internal.len() {
            let addr = internal.pop().unwrap();
            // This could be reduced by implementing TryInto
            let mut addr2 = PfrAddr::new();
            addr2.sync_c(addr)?;
            self.buffer.push(addr2);
        }

        self.size = io.pfrio_size as usize;
        self.added = io.pfrio_nadd as u32;
        self.deleted = io.pfrio_ndel as u32;

        Ok(())
    }

    /// Note: ignores internal self.size value
    fn repr_c(&self) -> Result<pfioc_table, PfError> {
        // Update self.pfrio_buffer to match self.buffer
        let mut internal = self.pfrio_buffer.borrow_mut();
        internal.clear();
        for addr in &self.buffer {
            internal.push(addr.repr_c()?);
        }

        let mut io = pfioc_table::init();
        io.pfrio_table = self.table.repr_c()?;
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
    fn code(&self) -> Result<u64, PfError> {
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