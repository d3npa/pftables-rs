pub mod bridge;
pub use bridge::*;
use std::{fmt, fs, io};
use std::error::Error;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum PfError {
    ConversionError,
    UnknownAddressFamily,
    Unimplemented,
    IoctlError(io::Error),
    Other(String),
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
    pub fn new(name: &str) -> PfTable {
        PfTable {
            name: String::from(name),
            addrs: Vec::new(),
        }
    }

    /// Queries the kernel to update internal vector of addresses
    pub fn get_addrs(&mut self, fd: &fs::File) -> Result<()> {
        // Prepare an Ioctl call
        let mut io = PfIocTable::with_table(&self.name);

        // Ask the kernel how many entries there are
        io.fire(&fd, PfIocCommand::GetAddrs)?;

        // Allocate room for number of entries based on returned size
        io.buffer = vec![PfrAddr::new(); io.size];
        io.fire(&fd, PfIocCommand::GetAddrs)?;

        self.addrs = io.buffer;
        Ok(())
    }

    pub fn add_addrs(&mut self, fd: &fs::File, addrs: Vec<PfrAddr>) -> Result<()> {
        let mut io = PfIocTable::with_table(&self.name);
        io.buffer = addrs;
        io.fire(&fd, PfIocCommand::AddAddrs)?;
        self.get_addrs(fd)
    }

    pub fn del_addrs(&mut self, fd: &fs::File, addrs: Vec<PfrAddr>) -> Result<()> {
        let mut io = PfIocTable::with_table(&self.name);
        io.buffer = addrs;
        io.fire(&fd, PfIocCommand::DelAddrs)?;
        self.get_addrs(fd)
    }

    pub fn clr_addrs(&mut self, fd: &fs::File) -> Result<()> {
        let mut io = PfIocTable::with_table(&self.name);
        io.fire(&fd, PfIocCommand::ClrAddrs)?;
        self.get_addrs(fd)
    }
}