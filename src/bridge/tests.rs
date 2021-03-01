use crate::bridge::*;
use std::net::{IpAddr, Ipv4Addr};
use std::convert::TryInto;
use std::error::Error;
use std::{mem, slice};

#[test]
fn convert_pfraddr_from_c() {
    let mut c_addr = pfr_addr::init();
    c_addr.pfra_af = AF_INET;
    c_addr.pfra_u._pfra_ip4addr = u32::from_le_bytes([127, 0, 0, 1]);
    c_addr.pfra_net = 32;
    c_addr.pfra_ifname = [
        118, 105, 111, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    ];

    let pf_addr = PfrAddr::try_from(c_addr).unwrap();
    assert_eq!(pf_addr.addr, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    assert_eq!(pf_addr.ifname, String::from("vio0"));
    assert_eq!(pf_addr.subnet, 32);
}

#[test]
fn convert_pfraddr_from_c_fail_af() {
    let c_addr = pfr_addr::init();
    assert!(PfrAddr::try_from(c_addr).is_err());
}

#[test]
fn convert_pfraddr_into_c() {
    let pf_addr = PfrAddr {
        addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        ifname: String::from("vio0"),
        subnet: 32,
    };

    let c_addr: pfr_addr = PfrAddr::try_into(pf_addr).unwrap();
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
fn convert_pfraddr_into_c_fail_len() {
    let pf_addr = PfrAddr {
        addr: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        ifname: String::from_utf8(vec![61u8; 16]).unwrap(),
        subnet: 32,
    };

    assert!(TryInto::<pfr_addr>::try_into(pf_addr).is_err());
}

#[test]
fn convert_pfrtable_from_c() {
    let mut c_table = pfr_table::init();
    c_table.pfrt_anchor = [0; PATH_MAX];
    c_table.pfrt_name = [
        104, 101, 108, 108, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    
    let table = PfrTable::try_from(c_table).unwrap();
    assert_eq!(table, PfrTable {
        anchor: String::from(""),
        name: String::from("hello"),
    });
}

#[test]
fn convert_pfrtable_into_c() {
    let table = PfrTable {
        anchor: String::from(""),
        name: String::from("hello"),
    };

    let pfrt = TryInto::<pfr_table>::try_into(table).unwrap();

    assert_eq!(pfrt.pfrt_name, [
        104, 101, 108, 108, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn convert_pfrtable_from_c_invalid_utf8() {
    let mut c_table = pfr_table::init();
    c_table.pfrt_name = [
        0, 159, 146, 150, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    
    assert!(PfrTable::try_from(c_table).is_err());
}

#[test]
fn convert_pfioctable_into_c() -> Result<(), Box<dyn Error>> {
    let io = PfiocTable {
        table: PfrTable {
            name: String::from("my_table"),
            anchor: String::from(""),
        },
        buffer: vec![
            PfrAddr { 
                ifname: String::from("vio0"), 
                addr: IpAddr::V4("127.0.0.1".parse()?),
                subnet: 32,
            },
        ],
    };

    let PfiocTableInter { io, addrs } = io.try_into()?;

    // addrs and addrs2 will be one and the same. Don't forget to 'forget'!
    let addrs2 = unsafe { slice::from_raw_parts(io.pfrio_buffer, addrs.len()) };

    assert_eq!(addrs[0].pfra_net, addrs2[0].pfra_net);
    mem::forget(addrs2);

    Ok(())
}

#[test]
fn convert_pfioctable_from_c() -> Result<(), Box<dyn Error>> {
    let mut addrs: Vec<pfr_addr> = vec![PfrAddr { 
        ifname: String::from("vio0"), 
        addr: IpAddr::V4("127.0.0.1".parse()?),
        subnet: 32,
    }.try_into()?];
    let mut io = pfioc_table::init();
    io.pfrio_table = pfr_table::init();
    io.pfrio_buffer = addrs.as_mut_ptr();
    io.pfrio_esize = PFR_ADDR_SIZE as i32;
    io.pfrio_size = addrs.len() as i32;

    let inter = PfiocTableInter { io, addrs };
    let _io: PfiocTable = inter.try_into()?;

    Ok(())
}