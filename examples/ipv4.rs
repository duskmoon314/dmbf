use std::net::Ipv4Addr;

use dmbf::bitfield;

/// IPv4 header
#[bitfield]
pub struct Ipv4 {
    /// Version (4 bits)
    ///
    /// The version of the IP protocol. This field is always set to 4.
    #[bitfield(bits = 4, default = 4)]
    version: u8,
    #[bitfield(bits = 4)]
    ihl: u8,
    #[bitfield(bits = 6)]
    dscp: u8,
    #[bitfield(bits = 2)]
    ecn: u8,
    #[bitfield(from = |v: u16| u16::from_be(v), into = |v: u16| u16::to_be(v))]
    total_length: u16,
    identification: u16,
    #[bitfield(bits = 3, from_into, from = |v: u16| v as u8)]
    flags: u8,
    #[bitfield(bits = 13)]
    fragment_offset: u16,
    ttl: u8,
    protocol: u8,
    checksum: u16,

    #[bitfield(bits = 32, from = |v: u32| Ipv4Addr::from(u32::from_be(v)), into = |v: Ipv4Addr| u32::to_be(v.into()))]
    src: Ipv4Addr,
    #[bitfield(bits = 32, from = |v: u32| Ipv4Addr::from(u32::from_be(v)), into = |v: Ipv4Addr| u32::to_be(v.into()))]
    dst: Ipv4Addr,
}

fn main() {
    let data: [u8; 20] = [
        0x45, 0x00, 0x00, 0x28, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0xb8, 0x0e, 0xc0, 0xa8, 0x00,
        0x01, 0xc0, 0xa8, 0x00, 0xc7,
    ];

    let mut ipv4 = Ipv4::from(&data[..]);

    assert_eq!(ipv4.version().get(), 4);

    ipv4.version_mut().set(5);

    assert_eq!(ipv4.version().get(), 5);

    ipv4.version_mut().reset();

    assert_eq!(ipv4.version().get(), 4);

    assert_eq!(ipv4.ihl().get(), 5);
    assert_eq!(ipv4.dscp().get(), 0);
    assert_eq!(ipv4.ecn().get(), 0);
    assert_eq!(ipv4.total_length().get(), 40);
    assert_eq!(ipv4.identification().get(), 0);
    assert_eq!(ipv4.flags().get(), 0);
    assert_eq!(ipv4.fragment_offset().get(), 64);
    assert_eq!(ipv4.ttl().get(), 64);
    assert_eq!(ipv4.protocol().get(), 17);
    assert_eq!(ipv4.checksum().get(), 0x0eb8);
    assert_eq!(ipv4.src().get(), Ipv4Addr::new(192, 168, 0, 1));
    assert_eq!(ipv4.dst().get(), Ipv4Addr::new(192, 168, 0, 199));
}
