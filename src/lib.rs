pub use dmbf_impl::bitfield;

use std::{cell::Cell, ptr};

/// Raw field type (`u8`, `u16`, `u32`, `u64`)
pub trait RawField:
    Copy
    + Default
    + core::ops::BitOr<Output = Self>
    + core::ops::BitAnd<Output = Self>
    + core::ops::BitOrAssign
    + core::ops::BitAndAssign
    + core::ops::Not<Output = Self>
    + core::ops::Shl<u8, Output = Self>
    + core::ops::Shr<u8, Output = Self>
{
}

macro_rules! impl_raw_field {
    ($U : ty) => {
        impl RawField for $U {}
        impl FieldSpec for $U {
            type Ux = $U;

            const DEFAULT: Self::Ux = 0;
            const MASK: Self::Ux = !0;
            const SHIFT: u8 = 0;

            type Target = Self;

            fn from_underlay(v: Self::Ux) -> Self::Target {
                v
            }
            fn into_underlay(v: Self::Target) -> Self::Ux {
                v
            }
        }
    };
}
impl_raw_field!(u8);
impl_raw_field!(u16);
impl_raw_field!(u32);
impl_raw_field!(u64);

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
    pub fn as_ptr(&self) -> *mut F::Ux {
        self.value.as_ptr()
    }

    pub fn raw(&self) -> F::Ux {
        unsafe { (ptr::read_volatile(self.as_ptr()) & F::MASK) >> F::SHIFT }
    }

    pub fn get(&self) -> F::Target {
        F::from_underlay(self.raw())
    }

    pub fn set(&mut self, value: impl Into<F::Target>) {
        let value = F::into_underlay(value.into());
        unsafe {
            ptr::write_volatile(
                self.as_ptr(),
                (value << F::SHIFT) & F::MASK | (ptr::read_volatile(self.as_ptr()) & !F::MASK),
            );
        }
    }

    pub fn reset(&mut self) {
        unsafe {
            ptr::write_volatile(
                self.as_ptr(),
                (F::DEFAULT << F::SHIFT) & F::MASK | (ptr::read_volatile(self.as_ptr()) & !F::MASK),
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
