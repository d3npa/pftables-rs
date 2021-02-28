use crate::bridge::*;
use std::net::{IpAddr, Ipv4Addr};
use std::convert::TryInto;

#[test]
fn convert_pfaddr_from_c() {
    let mut c_addr = pfr_addr::init();
    c_addr.pfra_af = AF_INET;
    c_addr.pfra_u._pfra_ip4addr = u32::from_le_bytes([127, 0, 0, 1]);
    c_addr.pfra_net = 32;
    c_addr.pfra_ifname = [
        118, 105, 111, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ];

    let pf_addr = PfAddr::try_from(c_addr).unwrap();
    assert_eq!(pf_addr.addr, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    assert_eq!(pf_addr.ifname, String::from("vio0"));
    assert_eq!(pf_addr.subnet, 32);
}

#[test]
fn convert_pfaddr_from_c_fail_af() {
    let c_addr = pfr_addr::init();
    assert!(PfAddr::try_from(c_addr).is_err());
}

#[test]
fn convert_pfaddr_into_c() {
    let pf_addr = PfAddr {
        addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        ifname: String::from("vio0"),
        subnet: 32,
    };

    let c_addr: pfr_addr = PfAddr::try_into(pf_addr).unwrap();
    assert_eq!(c_addr.pfra_af, AF_INET);
    assert_eq!(
        unsafe { c_addr.pfra_u._pfra_ip4addr }, 
        u32::from_le_bytes([127, 0, 0, 1]),
    );
    assert_eq!(
        c_addr.pfra_ifname, 
        [118, 105, 111, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(c_addr.pfra_net, 32);
}

#[test]
fn convert_pfaddr_into_c_fail_len() {
    let pf_addr = PfAddr {
        addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        ifname: String::from_utf8(vec![61u8; 16]).unwrap(),
        subnet: 32,
    };

    assert!(TryInto::<pfr_addr>::try_into(pf_addr).is_err());
}