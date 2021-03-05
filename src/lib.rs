//! A small library for managing pf (packet filter) tables on OpenBSD 
//! via `/dev/pf`
pub mod bridge;
pub use bridge::*;
use std::{fmt, fs, io};
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

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
#[derive(Debug)]
pub struct PfTable {
    pub name: String,
    pub addrs: Vec<PfrAddr>,
}

impl fmt::Display for PfTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display: Vec<String> = self.addrs.iter()
            .map(|x| format!("{}/{}", x.addr, x.subnet))
            .collect();

        write!(f, "{}: {:#?}", self.name, display)
    }
}

impl PfTable {
    /// Prepares a new `PfTable` with a provided `name`. To read an existing
    /// table, `get_addrs()` must first be called to populate `addrs`
    pub fn new(name: &str) -> PfTable {
        PfTable {
            name: String::from(name),
            addrs: Vec::new(),
        }
    }

    /// Asks the kernel to populate the internal field `addrs`
    pub fn get_addrs(&mut self, fd: &fs::File) -> Result<()> {
        // Prepare an Ioctl call
        let mut io = PfIocTable::new(&self.name);

        // Ask the kernel how many entries there are
        io.fire(&fd, PfIocCommand::GetAddrs)?;

        // Allocate room for number of entries based on returned size
        io.buffer = vec![PfrAddr::default(); io.size()];
        io.fire(&fd, PfIocCommand::GetAddrs)?;

        self.addrs = io.buffer;
        Ok(())
    }

    /// Asks the kernel to add a list of addresses to the table
    pub fn add_addrs(&mut self, fd: &fs::File, addrs: Vec<PfrAddr>) 
        -> Result<()> 
    {
        let mut io = PfIocTable::new(&self.name);
        io.buffer = addrs;
        io.fire(&fd, PfIocCommand::AddAddrs)?;
        self.get_addrs(fd)
    }

    /// Asks the kernel to delete a list of addresses from the table
    pub fn del_addrs(&mut self, fd: &fs::File, addrs: Vec<PfrAddr>) 
        -> Result<()> 
    {
        let mut io = PfIocTable::new(&self.name);
        io.buffer = addrs;
        io.fire(&fd, PfIocCommand::DelAddrs)?;
        self.get_addrs(fd)
    }

    /// Asks the kernel to remove every address from the table
    pub fn clr_addrs(&mut self, fd: &fs::File) -> Result<()> {
        let mut io = PfIocTable::new(&self.name);
        io.fire(&fd, PfIocCommand::ClrAddrs)?;
        self.get_addrs(fd)
    }
}