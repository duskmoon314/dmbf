# dmbf (Duskmoon's bitfield)

Yet another bitfield in Rust inspired by `svd2rust`'s generated code. I made
this because other crates not very suitable for my use case and I like making
(maybe uesless) wheels.

You may find other crates more suitable for your use case:

- [bitfield](https://crates.io/crates/bitfield)
- [modular-bitfield](https://crates.io/crates/modular-bitfield)
- [bitfield-struct](https://crates.io/crates/bitfield-struct)

## Usage

### An example of IPv4 header

Here's an example of representing IPv4 header:

```rust
#[bitfield]
pub struct Ipv4 {
    #[bitfield(bits = 4)]
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
```

Then you can use it like this:

```rust
let data: [u8; 20] = [
    0x45, 0x00, 0x00, 0x28, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11,
    0xb8, 0x0e, 0xc0, 0xa8, 0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7,
];

let mut ipv4 = Ipv4::from(&data);

assert_eq!(ipv4.version().get(), 4);
ipv4.version_mut().set(5);
assert_eq!(ipv4.version().get(), 5);

assert_eq!(ipv4.total_length().get(), 40);
assert_eq!(ipv4.src().get(), Ipv4Addr::new(192, 168, 0, 1));
assert_eq!(ipv4.dst().get(), Ipv4Addr::new(192, 168, 0, 199));
```

### How it works

The `bitfield` attribute macro constructs a `FieldBlock` struct, adhering to
the following rules:

-  Each field is a `Field<T>` where `T` must implement `FieldSpec` trait.
  - `FieldSpec` has two associated types: `Ux` for the underlying integer type
    and `Target` for the type we want to use.
  - `FieldSpec` also has three associated constants:
    - `DEFAULT` for resetting the field to default value.
    - `MASK` for masking the field. This is useful in a hybrid field.
    - `SHIFT` for shifting the field. This is useful in a hybrid field.
-  The type `T` is constructed by:
  - If `bits` is not specified, `T` is almost the same as the provided type.
  - If `bits` is specified and is a multiple of 8, `Ux` is a `u8`, `u16`, `u32`
    or `u64` depending on the number of bits.
  - If `bits` is specified and is not a multiple of 8, a hybrid field will be
    created by composing more than one fields into a `union`.

It also constructs a struct like this:

```rust
struct Ipv4<'a> {
    pub data: &'a [u8],
}

impl<'a> core::ops::Deref for Ipv4<'a> {
    type Target = FieldBlock;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.data.as_ptr() as *const Self::Target) }
    }
}

impl<'a> core::ops::DerefMut for Ipv4<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.data.as_ptr() as *mut Self::Target) }
    }
}
```

Yes, I use `unsafe` to cast a `&[u8]` to a `&FieldBlock` and it allows me to
access the fields without implementing complex parsing logic.

### attribute arguments

- `bits`: Number of bits to use for the field. If not specified, `<T as FieldSpec>::Ux` is used.
- `default`: Default value of the field. If not specified, `0` is used.
- `from_into: bool`: Whether to use `From` and `Into` to convert the field.
- `from` and `into`: Custom `From` and `Into` implementations
