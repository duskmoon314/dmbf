/// Raw field type
///
/// The following types are supported:
/// - u8, u16, u32, u64
/// - [u8; 3], [u8; 5], [u8; 6], [u8; 7]
pub trait RawField: Copy + Default + RawFieldOps {}

pub trait RawFieldOps {
    fn not(&self) -> Self;
    fn bitand(&self, rhs: Self) -> Self;
    fn bitor(&self, rhs: Self) -> Self;
    fn shl(&self, rhs: u8) -> Self;
    fn shr(&self, rhs: u8) -> Self;
}

macro_rules! impl_raw_field_ops_ux {
    ($( $Ux : ty ), *) => {
        $(
            impl RawFieldOps for $Ux {
                #[inline]
                fn not(&self) -> Self {
                    !self
                }
                #[inline]
                fn bitand(&self, rhs: Self) -> Self {
                    self & rhs
                }
                #[inline]
                fn bitor(&self, rhs: Self) -> Self {
                    self | rhs
                }
                #[inline]
                fn shl(&self, rhs: u8) -> Self {
                    self << rhs
                }
                #[inline]
                fn shr(&self, rhs: u8) -> Self {
                    self >> rhs
                }
            }
        )*
    };
}
impl_raw_field_ops_ux!(u8, u16, u32, u64);

macro_rules! impl_raw_field_ops_u8s {
    ($( $l : literal ), *) => {
        $(
            impl RawFieldOps for [u8; $l] {
                #[inline]
                fn not(&self) -> Self {
                    let mut ret = [0; $l];
                    for i in 0..$l {
                        ret[i] = !self[i];
                    }
                    ret
                }
                #[inline]
                fn bitand(&self, rhs: Self) -> Self {
                    if rhs == [!0; $l] {
                        return *self;
                    }
                    let mut ret = [0; $l];
                    for i in 0..$l {
                        ret[i] = self[i] & rhs[i];
                    }
                    ret
                }
                #[inline]
                fn bitor(&self, rhs: Self) -> Self {
                    if rhs == [0; $l] {
                        return *self;
                    }
                    let mut ret = [0; $l];
                    for i in 0..$l {
                        ret[i] = self[i] | rhs[i];
                    }
                    ret
                }
                #[inline]
                fn shl(&self, rhs: u8) -> Self {
                    if rhs == 0 {
                        return *self;
                    }
                    let mut ret = [0; $l];
                    let mut tmp: u16 = 0;
                    for i in (0..$l).rev() {
                        tmp = (self[i] as u16) << rhs | (tmp >> 8);
                        ret[i] = tmp as u8;
                    }
                    ret
                }
                #[inline]
                fn shr(&self, rhs: u8) -> Self {
                    if rhs == 0 {
                        return *self;
                    }
                    let mut ret = [0; $l];
                    let mut tmp: u16 = 0;
                    for i in 0..$l {
                        tmp = ((self[i] as u16) << (8 - rhs)) | (tmp << 8);
                        ret[i] = ((tmp & 0xff00) >> 8) as u8;
                        tmp &= 0x00ff;
                    }
                    ret
                }
            }
        )*
    }
}
impl_raw_field_ops_u8s!(3, 5, 6, 7);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_field_ops() {
        // test [u8, 3]
        assert_eq!(
            [0x01, 0x02, 0x04].bitand([0x02, 0x01, 0x03]),
            [0x00, 0x00, 0x00]
        );
        assert_eq!(
            [0x01, 0x02, 0x04].bitand([0x01, 0x02, 0x04]),
            [0x01, 0x02, 0x04]
        );
        assert_eq!(
            [0x01, 0x02, 0x04].bitor([0x02, 0x01, 0x03]),
            [0x03, 0x03, 0x07]
        );
        assert_eq!([0x00, 0x80, 0x00].shl(1), [0x01, 0x00, 0x00]);
        assert_eq!([0x01, 0x00, 0x00].shr(1), [0x00, 0x80, 0x00]);
    }
}
