//! A small library for managing pf (packet filter) tables on OpenBSD 
//! via `/dev/pf`
pub mod bridge;
pub use bridge::*;
use std::{fmt, fs, io};
use std::error::Error;

type Result<T> = std::result::Result<T, PfError>;

#[derive(Debug)]
pub enum PfError {
    TranslationError,
    UnknownAddressFamily,
    IoctlError(io::Error),
    Other(String),
    Unimplemented,
}

impl fmt::Display for PfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PfError::*;
        match self {
            Other(message) => {
                write!(f, "{}", message)
            },
            _ => {
                write!(f, "{:?}", self)
            }
        }
    }
}

impl Error for PfError {}

/// A high-level struct representing a pf table containing addresses
#[derive(Debug, Clone)]
pub struct PfTable {
    pub name: String,
}

impl PfTable {
    /// Prepares a new `PfTable` with a provided `name`
    pub fn new(name: &str) -> PfTable {
        PfTable {
            name: String::from(name),
        }
    }

    /// Asks the kernel for a list of addresses in the table
    pub fn get_addrs(&self, fd: &fs::File) -> Result<Vec<PfrAddr>> {
        // Prepare an Ioctl call
        let mut io = PfIocTable::new(&self.name);

        // Ask the kernel how many entries there are
        io.fire(&fd, PfIocCommand::GetAddrs)?;

        // Allocate room for number of entries based on returned size
        io.buffer = vec![PfrAddr::default(); io.size()];
        io.fire(&fd, PfIocCommand::GetAddrs)?;

        Ok(io.buffer)
    }

    /// Asks the kernel to add a list of addresses to the table
    pub fn add_addrs(&self, fd: &fs::File, addrs: Vec<PfrAddr>) 
        -> Result<()> 
    {
        let mut io = PfIocTable::new(&self.name);
        io.buffer = addrs;
        io.fire(&fd, PfIocCommand::AddAddrs)
    }

    /// Asks the kernel to delete a list of addresses from the table
    pub fn del_addrs(&self, fd: &fs::File, addrs: Vec<PfrAddr>) 
        -> Result<()> 
    {
        let mut io = PfIocTable::new(&self.name);
        io.buffer = addrs;
        io.fire(&fd, PfIocCommand::DelAddrs)
    }

    /// Asks the kernel to remove every address from the table
    pub fn clr_addrs(&self, fd: &fs::File) -> Result<()> {
        let mut io = PfIocTable::new(&self.name);
        io.fire(&fd, PfIocCommand::ClrAddrs)
    }
}