pub use dmbf_impl::bitfield;

pub mod underlay;
pub use underlay::RawField;
use underlay::RawFieldOps;

use std::cell::Cell;

pub enum Endianness {
    Lsb0,
    Msb0,
}

pub trait FieldSpec: Sized {
    /// Underlying type
    ///
    /// This type must implement the `RawField` trait.
    type Underlay: RawField;

    /// Target type
    ///
    /// The type that is returned by `get` and accepted by `set`.
    ///
    /// This type will be converted to and from the underlying type.
    type Target;

    /// Default value of the field
    const DEFAULT: Self::Underlay;

    /// Mask of the subfield
    const MASK: Self::Underlay;

    /// Shift of the subfield
    const SHIFT: u8;

    /// Endianness of the hybrid field
    ///
    /// This is used to determine how MASK and SHIFT are applied.
    const ENDIANNESS: Endianness;

    /// Conversion from underlying type to target type
    fn from_underlay(v: Self::Underlay) -> Self::Target;

    /// Conversion from target type to underlying type
    fn into_underlay(v: Self::Target) -> Self::Underlay;
}

#[derive(Debug)]
pub struct Field<F: FieldSpec> {
    value: Cell<F::Underlay>,
    _marker: core::marker::PhantomData<F>,
}

impl<F: FieldSpec> Field<F> {
    #[inline]
    pub fn as_ptr(&self) -> *mut F::Underlay {
        self.value.as_ptr()
    }

    #[inline]
    pub fn raw(&self) -> F::Underlay {
        match F::ENDIANNESS {
            Endianness::Lsb0 => F::Underlay::from_le(self.value.get())
                .bitand(F::MASK)
                .shr(F::SHIFT),
            Endianness::Msb0 => F::Underlay::from_be(self.value.get())
                .bitand(F::MASK)
                .shr(F::SHIFT),
        }
    }

    #[inline]
    pub fn get(&self) -> F::Target {
        F::from_underlay(self.raw())
    }

    #[inline]
    pub fn set(&mut self, v: F::Target) {
        let value = self.value.get_mut();
        match F::ENDIANNESS {
            Endianness::Lsb0 => {
                *value = value
                    .bitand(F::MASK.not())
                    .bitor(F::into_underlay(v).shl(F::SHIFT))
                    .to_le();
            }
            Endianness::Msb0 => {
                *value = value
                    .bitand(F::MASK.not())
                    .bitor(F::into_underlay(v).shl(F::SHIFT))
                    .to_be();
            }
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        let value = self.value.get_mut();
        match F::ENDIANNESS {
            Endianness::Lsb0 => {
                *value = value
                    .bitand(F::MASK.not())
                    .bitor(F::DEFAULT.shl(F::SHIFT))
                    .to_le();
            }
            Endianness::Msb0 => {
                *value = value
                    .bitand(F::MASK.not())
                    .bitor(F::DEFAULT.shl(F::SHIFT))
                    .to_be();
            }
        }
    }
}

impl<F: FieldSpec> Default for Field<F> {
    fn default() -> Self {
        Self {
            value: match F::ENDIANNESS {
                Endianness::Lsb0 => Cell::new(F::DEFAULT.to_le()),
                Endianness::Msb0 => Cell::new(F::DEFAULT.to_be()),
            },
            _marker: core::marker::PhantomData,
        }
    }
}

macro_rules! impl_field_spec_for_raw_field {
    ($( $U : ty ), *) => {
        $(
            impl FieldSpec for $U {
                type Underlay = $U;

                const DEFAULT: Self::Underlay = 0;
                const MASK: Self::Underlay = !0;
                const SHIFT: u8 = 0;
                const ENDIANNESS: Endianness = Endianness::Lsb0;

                type Target = Self;

                #[inline]
                fn from_underlay(v: Self::Underlay) -> Self::Target {
                    v
                }
                #[inline]
                fn into_underlay(v: Self::Target) -> Self::Underlay {
                    v
                }
            }
        )*
    };
    ($( $l : literal ), *) => {
        $(
            impl FieldSpec for [u8; $l] {
                type Underlay = [u8; $l];

                const DEFAULT: Self::Underlay = [0; $l];
                const MASK: Self::Underlay = [!0; $l];
                const SHIFT: u8 = 0;
                const ENDIANNESS: Endianness = Endianness::Lsb0;

                type Target = Self;

                #[inline]
                fn from_underlay(v: Self::Underlay) -> Self::Target {
                    v
                }
                #[inline]
                fn into_underlay(v: Self::Target) -> Self::Underlay {
                    v
                }
            }
        )*
    }
}

impl_field_spec_for_raw_field!(u8, u16, u32, u64);
impl_field_spec_for_raw_field!(3, 5, 6, 7);
