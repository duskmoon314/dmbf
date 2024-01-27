use dmbf_impl::bitfield;

#[test]
fn hybrid_fields_u16() {
    #[bitfield(endianness = dmbf::Endianness::Msb0)]
    struct Foo {
        #[bitfield(bits = 1, from_into, from = |v: u16| v as u8)]
        a: u8,
        #[bitfield(bits = 15)]
        b: u16,
    }

    for i in 0_u16..=0xFFFF_u16 {
        let data: [u8; 2] = i.to_be_bytes();
        let foo = Foo::from(&data);

        assert_eq!(foo.a().get(), ((i & 0x8000) >> 15) as u8);
        assert_eq!(foo.b().get(), i & 0x7FFF);
    }

    #[bitfield(endianness = dmbf::Endianness::Lsb0)]
    struct Bar {
        #[bitfield(bits = 1, from_into, from = |v: u16| v as u8)]
        a: u8,
        #[bitfield(bits = 15)]
        b: u16,
    }

    for i in 0_u16..=0xFFFF_u16 {
        let data: [u8; 2] = i.to_be_bytes();
        let bar = Bar::from(&data);

        assert_eq!(bar.a().get(), ((i & 0x0080) >> 7) as u8);
        assert_eq!(bar.b().get(), ((i & 0xFF00) >> 8) | ((i & 0x007F) << 8));
    }
}

#[test]
fn hybrid_fields_u8_3() {
    #[bitfield(endianness = dmbf::Endianness::Msb0)]
    struct Foo {
        #[bitfield(bits = 1, from = |v: [u8; 3]| v[2] as u8, into = |v: u8| [0, 0, v])]
        a: u8,
        #[bitfield(bits = 2, from = |v: [u8; 3]| v[2] as u8, into = |v: u8| [0, 0, v])]
        b: u8,
        #[bitfield(bits = 3, from = |v: [u8; 3]| v[2] as u8, into = |v: u8| [0, 0, v])]
        c: u8,
        #[bitfield(bits = 5, from = |v: [u8; 3]| v[2] as u8, into = |v: u8| [0, 0, v])]
        d: u8,
        #[bitfield(bits = 7, from = |v: [u8; 3]| v[2] as u8, into = |v: u8| [0, 0, v])]
        e: u8,
        #[bitfield(bits = 6, from = |v: [u8; 3]| v[2] as u8, into = |v: u8| [0, 0, v])]
        f: u8,
    }

    for i in 0..=0xFF {
        for j in 0..=0xFF {
            for k in 0..=0xFF {
                let data: [u8; 3] = [i, j, k];
                let foo = Foo::from(&data);

                assert_eq!(foo.a().get(), (i & 0x80) >> 7);
                assert_eq!(foo.b().get(), (i & 0x60) >> 5);
            }
        }
    }
}
