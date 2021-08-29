use std::num::Wrapping;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

macro_rules! def_hardware_type {
    ($type_name:ident, $base_type:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $type_name(Wrapping<$base_type>);

        #[allow(dead_code)]
        impl $type_name {
            pub const ZERO: $type_name = Self(Wrapping(0));
            pub const MIN: $type_name = Self(Wrapping(<$base_type>::MIN));
            pub const MAX: $type_name = Self(Wrapping(<$base_type>::MAX));

            #[inline]
            pub const fn new(value: $base_type) -> Self {
                Self(Wrapping(value))
            }
        }

        impl Add for $type_name {
            type Output = Self;

            #[inline]
            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }
        impl AddAssign for $type_name {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }
        impl Add<$base_type> for $type_name {
            type Output = Self;

            #[inline]
            fn add(self, rhs: $base_type) -> Self::Output {
                Self(self.0 + Wrapping(rhs))
            }
        }
        impl AddAssign<$base_type> for $type_name {
            #[inline]
            fn add_assign(&mut self, rhs: $base_type) {
                self.0 += Wrapping(rhs);
            }
        }

        impl Sub for $type_name {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }
        impl SubAssign for $type_name {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
        impl Sub<$base_type> for $type_name {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: $base_type) -> Self::Output {
                Self(self.0 + Wrapping(rhs))
            }
        }
        impl SubAssign<$base_type> for $type_name {
            #[inline]
            fn sub_assign(&mut self, rhs: $base_type) {
                self.0 -= Wrapping(rhs);
            }
        }

        impl Mul for $type_name {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }
        impl MulAssign for $type_name {
            #[inline]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }
        impl Mul<$base_type> for $type_name {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: $base_type) -> Self::Output {
                Self(self.0 * Wrapping(rhs))
            }
        }
        impl MulAssign<$base_type> for $type_name {
            #[inline]
            fn mul_assign(&mut self, rhs: $base_type) {
                self.0 *= Wrapping(rhs);
            }
        }

        impl Div for $type_name {
            type Output = Self;

            #[inline]
            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }
        impl DivAssign for $type_name {
            #[inline]
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }
        impl Div<$base_type> for $type_name {
            type Output = Self;

            #[inline]
            fn div(self, rhs: $base_type) -> Self::Output {
                Self(self.0 / Wrapping(rhs))
            }
        }
        impl DivAssign<$base_type> for $type_name {
            #[inline]
            fn div_assign(&mut self, rhs: $base_type) {
                self.0 /= Wrapping(rhs);
            }
        }

        impl Rem for $type_name {
            type Output = Self;

            #[inline]
            fn rem(self, rhs: Self) -> Self::Output {
                Self(self.0 % rhs.0)
            }
        }
        impl RemAssign for $type_name {
            #[inline]
            fn rem_assign(&mut self, rhs: Self) {
                self.0 %= rhs.0;
            }
        }
        impl Rem<$base_type> for $type_name {
            type Output = Self;

            #[inline]
            fn rem(self, rhs: $base_type) -> Self::Output {
                Self(self.0 % Wrapping(rhs))
            }
        }
        impl RemAssign<$base_type> for $type_name {
            #[inline]
            fn rem_assign(&mut self, rhs: $base_type) {
                self.0 %= Wrapping(rhs);
            }
        }

        impl BitAnd for $type_name {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }
        impl BitAndAssign for $type_name {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }
        impl BitAnd<$base_type> for $type_name {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: $base_type) -> Self::Output {
                Self(self.0 & Wrapping(rhs))
            }
        }
        impl BitAndAssign<$base_type> for $type_name {
            #[inline]
            fn bitand_assign(&mut self, rhs: $base_type) {
                self.0 &= Wrapping(rhs);
            }
        }

        impl BitOr for $type_name {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }
        impl BitOrAssign for $type_name {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }
        impl BitOr<$base_type> for $type_name {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: $base_type) -> Self::Output {
                Self(self.0 | Wrapping(rhs))
            }
        }
        impl BitOrAssign<$base_type> for $type_name {
            #[inline]
            fn bitor_assign(&mut self, rhs: $base_type) {
                self.0 |= Wrapping(rhs);
            }
        }

        impl BitXor for $type_name {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.0 ^ rhs.0)
            }
        }
        impl BitXorAssign for $type_name {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 ^= rhs.0;
            }
        }
        impl BitXor<$base_type> for $type_name {
            type Output = Self;

            #[inline]
            fn bitxor(self, rhs: $base_type) -> Self::Output {
                Self(self.0 ^ Wrapping(rhs))
            }
        }
        impl BitXorAssign<$base_type> for $type_name {
            #[inline]
            fn bitxor_assign(&mut self, rhs: $base_type) {
                self.0 ^= Wrapping(rhs);
            }
        }

        impl Not for $type_name {
            type Output = Self;

            #[inline]
            fn not(self) -> Self::Output {
                Self(!self.0)
            }
        }

        impl Shl for $type_name {
            type Output = Self;

            #[inline]
            fn shl(self, rhs: Self) -> Self::Output {
                Self(self.0 << (rhs.0 .0 as usize))
            }
        }
        impl ShlAssign for $type_name {
            #[inline]
            fn shl_assign(&mut self, rhs: Self) {
                self.0 <<= rhs.0 .0 as usize
            }
        }
        impl Shl<usize> for $type_name {
            type Output = Self;

            #[inline]
            fn shl(self, rhs: usize) -> Self::Output {
                Self(self.0 << rhs)
            }
        }
        impl ShlAssign<usize> for $type_name {
            #[inline]
            fn shl_assign(&mut self, rhs: usize) {
                self.0 <<= rhs
            }
        }

        impl Shr for $type_name {
            type Output = Self;

            #[inline]
            fn shr(self, rhs: Self) -> Self::Output {
                Self(self.0 >> (rhs.0 .0 as usize))
            }
        }
        impl ShrAssign for $type_name {
            #[inline]
            fn shr_assign(&mut self, rhs: Self) {
                self.0 >>= rhs.0 .0 as usize
            }
        }
        impl Shr<usize> for $type_name {
            type Output = Self;

            #[inline]
            fn shr(self, rhs: usize) -> Self::Output {
                Self(self.0 >> rhs)
            }
        }
        impl ShrAssign<usize> for $type_name {
            #[inline]
            fn shr_assign(&mut self, rhs: usize) {
                self.0 >>= rhs
            }
        }

        impl From<$base_type> for $type_name {
            #[inline]
            fn from(value: $base_type) -> Self {
                Self(Wrapping(value))
            }
        }
        impl Into<$base_type> for $type_name {
            #[inline]
            fn into(self) -> $base_type {
                self.0 .0
            }
        }
    };
}

def_hardware_type!(Byte, u8);
def_hardware_type!(Word, u16);

impl Word {
    #[inline]
    pub fn low(&self) -> Byte {
        Byte(Wrapping(self.0 .0.to_le_bytes()[0]))
    }

    #[inline]
    pub fn high(&self) -> Byte {
        Byte(Wrapping(self.0 .0.to_le_bytes()[1]))
    }

    #[inline]
    pub fn set_low(&mut self, value: Byte) {
        let high = self.high();
        self.0 = Wrapping(u16::from_le_bytes([value.0 .0, high.0 .0]));
    }

    #[inline]
    pub fn set_high(&mut self, value: Byte) {
        let low = self.low();
        self.0 = Wrapping(u16::from_le_bytes([low.0 .0, value.0 .0]));
    }
}
