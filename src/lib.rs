pub use dmbf_impl::bitfield;

pub mod underlay;
pub use underlay::RawField;
use underlay::RawFieldOps;

use std::{cell::Cell, ptr};

pub trait FieldSpec: Sized {
    type Ux: RawField;

    const DEFAULT: Self::Ux;
    const MASK: Self::Ux;
    const SHIFT: u8;

    type Target;

    fn from_underlay(v: Self::Ux) -> Self::Target;
    fn into_underlay(v: Self::Target) -> Self::Ux;
}

#[derive(Debug)]
pub struct Field<F: FieldSpec> {
    value: Cell<F::Ux>,
    _marker: core::marker::PhantomData<F>,
}

impl<F: FieldSpec> Field<F> {
    #[inline]
    pub fn as_ptr(&self) -> *mut F::Ux {
        self.value.as_ptr()
    }

    #[inline]
    pub fn raw(&self) -> F::Ux {
        unsafe { (ptr::read_volatile(self.as_ptr()).bitand(F::MASK)).shr(F::SHIFT) }
    }

    #[inline]
    pub fn get(&self) -> F::Target {
        F::from_underlay(self.raw())
    }

    #[inline]
    pub fn set(&mut self, value: impl Into<F::Target>) {
        let value = F::into_underlay(value.into());
        unsafe {
            ptr::write_volatile(
                self.as_ptr(),
                (value.shl(F::SHIFT))
                    .bitand(F::MASK)
                    .bitor(ptr::read_volatile(self.as_ptr()).bitand(F::MASK.not())),
            );
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        unsafe {
            ptr::write_volatile(
                self.as_ptr(),
                F::DEFAULT
                    .shl(F::SHIFT)
                    .bitand(F::MASK)
                    .bitor(ptr::read_volatile(self.as_ptr()).bitand(F::MASK.not())),
            );
        }
    }
}

impl<F> Default for Field<F>
where
    F: FieldSpec,
{
    fn default() -> Self {
        Self {
            value: Cell::new(F::DEFAULT),
            _marker: core::marker::PhantomData,
        }
    }
}

macro_rules! impl_field_spec_for_raw_field {
    ($( $U : ty ), *) => {
        $(
            impl RawField for $U {}
            impl FieldSpec for $U {
                type Ux = $U;

                const DEFAULT: Self::Ux = 0;
                const MASK: Self::Ux = !0;
                const SHIFT: u8 = 0;

                type Target = Self;

                #[inline]
                fn from_underlay(v: Self::Ux) -> Self::Target {
                    v
                }
                #[inline]
                fn into_underlay(v: Self::Target) -> Self::Ux {
                    v
                }
            }
        )*
    };
    ($( $l : literal ), *) => {
        $(
            impl RawField for [u8; $l] {}
            impl FieldSpec for [u8; $l] {
                type Ux = [u8; $l];

                const DEFAULT: Self::Ux = [0; $l];
                const MASK: Self::Ux = [!0; $l];
                const SHIFT: u8 = 0;

                type Target = Self;

                #[inline]
                fn from_underlay(v: Self::Ux) -> Self::Target {
                    v
                }
                #[inline]
                fn into_underlay(v: Self::Target) -> Self::Ux {
                    v
                }
            }
        )*
    }
}
impl_field_spec_for_raw_field!(u8, u16, u32, u64);
impl_field_spec_for_raw_field!(3, 5, 6, 7);
