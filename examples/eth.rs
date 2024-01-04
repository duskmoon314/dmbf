use dmbf::bitfield;

#[bitfield]
pub struct Eth {
    dst: [u8; 6],
    src: [u8; 6],
    #[bitfield(from = |v: u16| u16::from_be(v), into = |v: u16| u16::to_be(v))]
    ty: u16,
}

fn main() {
    let data: [u8; 14] = [
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, // dst
        0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, // src
        0x08, 0x00, // type
    ];

    let eth = Eth::from(&data);

    assert_eq!(eth.dst().get(), [0x00, 0x01, 0x02, 0x03, 0x04, 0x05]);
    assert_eq!(eth.src().get(), [0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b]);
    assert_eq!(eth.ty().get(), 0x0800);
}
