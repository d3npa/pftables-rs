#[cfg(test)] mod tests;
pub mod bindings;
use bindings::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::convert::{TryFrom, Into};
use crate::PfError;

// Create more Rust-friendly (and safer) versions of the pf structs
pub struct PfAddr {
    pub addr: IpAddr,
    pub ifname: String,
    pub subnet: u8,
    // Other fields are unused right now
}

impl TryFrom<pfr_addr> for PfAddr {
    type Error = crate::PfError;
    /// Will fail if pfra_af field is invalid
    fn try_from(a: pfr_addr) -> Result<PfAddr, PfError> {
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
            Err(_) => return Err(PfError::ConversionError)
        };

        Ok(PfAddr {
            addr,
            ifname,
            subnet: a.pfra_net,
        })
    }
}

impl Into<pfr_addr> for PfAddr {
    fn into(self) -> pfr_addr {
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

        c_addr
    }
}