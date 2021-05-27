// Rust language amplification library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
// Updated in 2020-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

//! Custom-sized numeric types
//!
//! Implementation of a various integer types with custom bit dimension. These
//! includes:
//! * large signed and unsigned integers (256, 512, 1024-bit)
//! * custom sub-8 bit unsigned ingegers (5-, 6-, 7-bit)
//! * 24-bit usigned integer.
//!
//! The functions here are designed to be fast.

use core::ops::{
    Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Rem, RemAssign, BitAnd,
    BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Shl, ShlAssign, Shr, ShrAssign,
};
use core::ops::Deref;
use core::convert::TryFrom;

/// A trait which allows numbers to act as fixed-size bit arrays
pub trait BitArray {
    /// Is bit set?
    fn bit(&self, idx: usize) -> bool;

    /// Returns an array which is just the bits from start to end
    fn bit_slice(&self, start: usize, end: usize) -> Self;

    /// Bitwise and with `n` ones
    fn mask(&self, n: usize) -> Self;

    /// Trailing zeros
    fn trailing_zeros(&self) -> usize;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(
    feature = "std",
    derive(Display, Error),
    display(
        "Unable to construct bit-sized integer from a value `{value}` overflowing max value `{max}`"
    )
)]
/// Error indicating that a value does not fit integer dimension
pub struct ValueOverflow {
    /// Integer bit size
    pub max: usize,
    /// Value that overflows
    pub value: usize,
}

macro_rules! construct_bitint {
    ($ty:ident, $inner:ident, $bits:literal, $max:expr, $doc:meta) => {
        #[$doc]
        #[derive(PartialEq, Eq, Debug, Copy, Clone, Default, PartialOrd, Ord, Hash)]
        #[cfg_attr(
            feature = "serde",
            derive(Serialize, Deserialize),
            serde(crate = "serde_crate", transparent)
        )]
        #[allow(non_camel_case_types)]
        pub struct $ty($inner);

        impl $ty {
            /// Bit dimension
            pub const BITS: u32 = $bits;

            /// Minimum value
            pub const MIN: Self = Self(0);

            /// Maximal value
            pub const MAX: Self = Self($max - 1);

            /// One value
            pub const ONE: Self = Self(1);

            /// Returns inner representation
            pub fn as_u8(self) -> $inner {
                self.0 as $inner
            }

            /// Creates a new value from a provided `value.
            ///
            /// Panics if the value exceeds `Self::MAX`
            pub fn with(value: $inner) -> Self {
                assert!(value < $max, "provided value exceeds Self::MAX");
                Self(value)
            }
        }

        impl ::core::convert::TryFrom<$inner> for $ty {
            type Error = ValueOverflow;
            #[inline]
            fn try_from(value: $inner) -> Result<Self, Self::Error> {
                if value >= $max {
                    Err(ValueOverflow { max: $max as usize - 1, value: value as usize })
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl From<$ty> for $inner {
            #[inline]
            fn from(val: $ty) -> Self {
                val.0
            }
        }

        impl AsRef<$inner> for $ty {
            #[inline]
            fn as_ref(&self) -> &$inner {
                &self.0
            }
        }

        impl Deref for $ty {
            type Target = $inner;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        #[cfg(feature = "std")]
        impl ::std::str::FromStr for $ty {
            type Err = ::std::num::ParseIntError;
            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::try_from($inner::from_str(s)?).map_err(|_| u8::from_str("257").unwrap_err())
            }
        }

        #[cfg(feature = "std")]
        impl ::std::fmt::Display for $ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl_op!($ty, $inner, Add, add, AddAssign, add_assign, +);
        impl_op!($ty, $inner, Sub, sub, SubAssign, sub_assign, -);
        impl_op!($ty, $inner, Mul, mul, MulAssign, mul_assign, *);
        impl_op!($ty, $inner, Div, div, DivAssign, div_assign, /);
        impl_op!($ty, $inner, Rem, rem, RemAssign, rem_assign, %);
        impl_op!($ty, $inner, BitAnd, bitand, BitAndAssign, bitand_assign, &);
        impl_op!($ty, $inner, BitOr, bitor, BitOrAssign, bitor_assign, |);
        impl_op!($ty, $inner, BitXor, bitxor, BitXorAssign, bitxor_assign, ^);
        impl_op!($ty, $inner, Shl, shl, ShlAssign, shl_assign, <<);
        impl_op!($ty, $inner, Shr, shr, ShrAssign, shr_assign, >>);
    };
}
macro_rules! impl_op {
    ($ty:ty, $inner:ty, $op:ident, $fn:ident, $op_assign:ident, $fn_assign:ident, $sign:tt) => {
        impl $op for $ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self::try_from((self.0).$fn(rhs.0)).expect(stringify!(
                    "integer overflow during ",
                    $fn,
                    " operation"
                ))
            }
        }
        impl $op for &$ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: Self) -> Self::Output {
                *self $sign *rhs
            }
        }
        impl $op<&$ty> for $ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: &$ty) -> Self::Output {
                self $sign *rhs
            }
        }
        impl $op<$ty> for &$ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: $ty) -> Self::Output {
                *self $sign rhs
            }
        }

        impl $op<$inner> for $ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: $inner) -> Self::Output {
                Self::try_from((self.0).$fn(rhs)).expect(stringify!(
                    "integer overflow during ",
                    $fn,
                    " operation"
                ))
            }
        }
        impl $op<&$inner> for &$ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: &$inner) -> Self::Output {
                *self $sign *rhs
            }
        }
        impl $op<&$inner> for $ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: &$inner) -> Self::Output {
                self $sign *rhs
            }
        }
        impl $op<$inner> for &$ty {
            type Output = $ty;
            #[inline]
            fn $fn(self, rhs: $inner) -> Self::Output {
                *self $sign rhs
            }
        }

        impl $op_assign for $ty {
            #[inline]
            fn $fn_assign(&mut self, rhs: Self) {
                self.0 = (*self $sign rhs).0
            }
        }
        impl $op_assign<&$ty> for $ty {
            #[inline]
            fn $fn_assign(&mut self, rhs: &$ty) {
                self.0 = (*self $sign *rhs).0
            }
        }
        impl $op_assign<$inner> for $ty {
            #[inline]
            fn $fn_assign(&mut self, rhs: $inner) {
                self.0 = (*self $sign rhs).0
            }
        }
        impl $op_assign<&$inner> for $ty {
            #[inline]
            fn $fn_assign(&mut self, rhs: &$inner) {
                self.0 = (*self $sign *rhs).0
            }
        }
    };
}

construct_bitint!(
    u2,
    u8,
    2,
    4,
    doc = "5-bit unsigned integer in the range `0..4`"
);
construct_bitint!(
    u3,
    u8,
    3,
    8,
    doc = "5-bit unsigned integer in the range `0..8`"
);
construct_bitint!(
    u4,
    u8,
    4,
    16,
    doc = "5-bit unsigned integer in the range `0..16`"
);
construct_bitint!(
    u5,
    u8,
    5,
    32,
    doc = "5-bit unsigned integer in the range `0..32`"
);
construct_bitint!(
    u6,
    u8,
    6,
    64,
    doc = "6-bit unsigned integer in the range `0..64`"
);
construct_bitint!(
    u7,
    u8,
    7,
    128,
    doc = "7-bit unsigned integer in the range `0..128`"
);
construct_bitint!(
    u24,
    u32,
    24,
    1u32 << 24,
    doc = "24-bit unsigned integer in the range `0..16_777_216`"
);

macro_rules! construct_uint {
    ($name:ident, $n_words:expr) => {
        /// Little-endian large integer type
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
        pub struct $name([u64; $n_words]);

        impl $name {
            #[inline]
            /// Converts the object to a raw pointer
            pub fn as_ptr(&self) -> *const u64 {
                let &$name(ref dat) = self;
                dat.as_ptr()
            }

            #[inline]
            /// Converts the object to a mutable raw pointer
            pub fn as_mut_ptr(&mut self) -> *mut u64 {
                let &mut $name(ref mut dat) = self;
                dat.as_mut_ptr()
            }

            #[inline]
            /// Returns the length of the object as an array
            pub fn array_len(&self) -> usize {
                $n_words
            }

            #[inline]
            /// Returns the length of the object as an array
            #[deprecated(since = "3.5.2", note = "use `array_len` instead")]
            pub fn len(&self) -> usize {
                $n_words
            }

            #[inline]
            /// Returns the length of the object as an array
            pub fn byte_len(&self) -> usize {
                $n_words * 8
            }

            #[inline]
            /// Returns whether the object, as an array, is empty. Always false.
            pub fn is_empty(&self) -> bool {
                false
            }

            #[inline]
            /// Returns the underlying array of words constituting large integer
            pub fn as_inner(&self) -> &[u64; $n_words] {
                &self.0
            }

            #[inline]
            /// Returns the underlying array of words constituting large integer
            pub fn into_inner(self) -> [u64; $n_words] {
                self.0
            }

            #[inline]
            /// Constructs integer type from the underlying array of words.
            pub fn from_inner(array: [u64; $n_words]) -> Self {
                Self(array)
            }
        }

        impl $name {
            /// Zero value
            pub const ZERO: $name = $name([0u64; $n_words]);

            /// Value for `1`
            pub const ONE: $name = $name({
                let mut one = [0u64; $n_words];
                one[0] = 1u64;
                one
            });

            /// Minimum value
            pub const MIN: $name = $name([0u64; $n_words]);

            /// Maximum value
            pub const MAX: $name = $name([::core::u64::MAX; $n_words]);

            /// Bit dimension
            pub const BITS: u32 = $n_words * 64;

            /// Returns lower 32 bits of the number as `u32`
            #[inline]
            pub fn low_u32(&self) -> u32 {
                let &$name(ref arr) = self;
                (arr[0] & ::core::u32::MAX as u64) as u32
            }

            /// Returns lower 64 bits of the number as `u32`
            #[inline]
            pub fn low_u64(&self) -> u64 {
                let &$name(ref arr) = self;
                arr[0] as u64
            }

            /// Return the least number of bits needed to represent the number
            #[inline]
            pub fn bits_required(&self) -> usize {
                let &$name(ref arr) = self;
                for i in 1..$n_words {
                    if arr[$n_words - i] > 0 {
                        return (0x40 * ($n_words - i + 1))
                            - arr[$n_words - i].leading_zeros() as usize;
                    }
                }
                0x40 - arr[0].leading_zeros() as usize
            }

            /// Multiplication by u32
            pub fn mul_u32(self, other: u32) -> $name {
                let $name(ref arr) = self;
                let mut carry = [0u64; $n_words];
                let mut ret = [0u64; $n_words];
                for i in 0..$n_words {
                    let not_last_word = i < $n_words - 1;
                    let upper = other as u64 * (arr[i] >> 32);
                    let lower = other as u64 * (arr[i] & 0xFFFFFFFF);
                    if not_last_word {
                        carry[i + 1] += upper >> 32;
                    }
                    let (sum, overflow) = lower.overflowing_add(upper << 32);
                    ret[i] = sum;
                    if overflow && not_last_word {
                        carry[i + 1] += 1;
                    }
                }
                $name(ret) + $name(carry)
            }

            /// Creates the integer value from a byte array using big-endian
            /// encoding
            pub fn from_be_bytes(bytes: [u8; $n_words * 8]) -> $name {
                Self::_from_be_slice(&bytes)
            }

            /// Creates the integer value from a byte slice using big-endian
            /// encoding
            pub fn from_be_slice(bytes: &[u8]) -> Result<$name, ParseLengthError> {
                if bytes.len() != $n_words * 8 {
                    Err(ParseLengthError {
                        actual: bytes.len(),
                        expected: $n_words * 8,
                    })
                } else {
                    Ok(Self::_from_be_slice(bytes))
                }
            }

            /// Creates the integer value from a byte array using little-endian
            /// encoding
            pub fn from_le_bytes(bytes: [u8; $n_words * 8]) -> $name {
                Self::_from_le_slice(&bytes)
            }

            /// Creates the integer value from a byte slice using little-endian
            /// encoding
            pub fn from_le_slice(bytes: &[u8]) -> Result<$name, ParseLengthError> {
                if bytes.len() != $n_words * 8 {
                    Err(ParseLengthError {
                        actual: bytes.len(),
                        expected: $n_words * 8,
                    })
                } else {
                    Ok(Self::_from_le_slice(bytes))
                }
            }

            fn _from_be_slice(bytes: &[u8]) -> $name {
                let mut slice = [0u64; $n_words];
                slice
                    .iter_mut()
                    .rev()
                    .zip(bytes.chunks(8).into_iter().map(|s| {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(s);
                        b
                    }))
                    .for_each(|(word, bytes)| *word = u64::from_be_bytes(bytes));
                $name(slice)
            }

            fn _from_le_slice(bytes: &[u8]) -> $name {
                let mut slice = [0u64; $n_words];
                slice
                    .iter_mut()
                    .zip(bytes.chunks(8).into_iter().map(|s| {
                        let mut b = [0u8; 8];
                        b.copy_from_slice(s);
                        b
                    }))
                    .for_each(|(word, bytes)| *word = u64::from_le_bytes(bytes));
                $name(slice)
            }

            /// Convert the integer into a byte array using big-endian encoding
            pub fn to_be_bytes(self) -> [u8; $n_words * 8] {
                let mut res = [0; $n_words * 8];
                for i in 0..$n_words {
                    let start = i * 8;
                    res[start..start + 8]
                        .copy_from_slice(&self.0[$n_words - (i + 1)].to_be_bytes());
                }
                res
            }

            /// Convert a integer into a byte array using little-endian encoding
            pub fn to_le_bytes(self) -> [u8; $n_words * 8] {
                let mut res = [0; $n_words * 8];
                for i in 0..$n_words {
                    let start = i * 8;
                    res[start..start + 8].copy_from_slice(&self.0[i].to_le_bytes());
                }
                res
            }

            // divmod like operation, returns (quotient, remainder)
            #[inline]
            fn div_rem(self, other: Self) -> (Self, Self) {
                let mut sub_copy = self;
                let mut shift_copy = other;
                let mut ret = [0u64; $n_words];

                let my_bits = self.bits_required();
                let your_bits = other.bits_required();

                // Check for division by 0
                assert!(your_bits != 0);

                // Early return in case we are dividing by a larger number than us
                if my_bits < your_bits {
                    return ($name(ret), sub_copy);
                }

                // Bitwise long division
                let mut shift = my_bits - your_bits;
                shift_copy = shift_copy << shift;
                loop {
                    if sub_copy >= shift_copy {
                        ret[shift / 64] |= 1 << (shift % 64);
                        sub_copy = sub_copy - shift_copy;
                    }
                    shift_copy = shift_copy >> 1;
                    if shift == 0 {
                        break;
                    }
                    shift -= 1;
                }

                ($name(ret), sub_copy)
            }
        }

        impl From<u8> for $name {
            fn from(init: u8) -> $name {
                let mut ret = [0; $n_words];
                ret[0] = init as u64;
                $name(ret)
            }
        }

        impl From<u16> for $name {
            fn from(init: u16) -> $name {
                let mut ret = [0; $n_words];
                ret[0] = init as u64;
                $name(ret)
            }
        }

        impl From<u32> for $name {
            fn from(init: u32) -> $name {
                let mut ret = [0; $n_words];
                ret[0] = init as u64;
                $name(ret)
            }
        }

        impl From<u64> for $name {
            fn from(init: u64) -> $name {
                let mut ret = [0; $n_words];
                ret[0] = init;
                $name(ret)
            }
        }

        impl From<u128> for $name {
            fn from(init: u128) -> $name {
                let mut ret = [0; $n_words * 8];
                for (pos, byte) in init.to_le_bytes().iter().enumerate() {
                    ret[pos] = *byte;
                }
                $name::from_le_bytes(ret)
            }
        }

        impl<'a> ::core::convert::TryFrom<&'a [u64]> for $name {
            type Error = $crate::num::ParseLengthError;
            fn try_from(data: &'a [u64]) -> Result<$name, Self::Error> {
                if data.len() != $n_words {
                    Err(ParseLengthError {
                        actual: data.len(),
                        expected: $n_words,
                    })
                } else {
                    let mut bytes = [0u64; $n_words];
                    bytes.copy_from_slice(data);
                    Ok(Self::from_inner(bytes))
                }
            }
        }
        impl ::core::ops::Index<usize> for $name {
            type Output = u64;

            #[inline]
            fn index(&self, index: usize) -> &u64 {
                &self.0[index]
            }
        }

        impl ::core::ops::Index<::std::ops::Range<usize>> for $name {
            type Output = [u64];

            #[inline]
            fn index(&self, index: ::core::ops::Range<usize>) -> &[u64] {
                &self.0[index]
            }
        }

        impl ::core::ops::Index<::std::ops::RangeTo<usize>> for $name {
            type Output = [u64];

            #[inline]
            fn index(&self, index: ::core::ops::RangeTo<usize>) -> &[u64] {
                &self.0[index]
            }
        }

        impl ::core::ops::Index<::core::ops::RangeFrom<usize>> for $name {
            type Output = [u64];

            #[inline]
            fn index(&self, index: ::core::ops::RangeFrom<usize>) -> &[u64] {
                &self.0[index]
            }
        }

        impl ::core::ops::Index<::core::ops::RangeFull> for $name {
            type Output = [u64];

            #[inline]
            fn index(&self, _: ::core::ops::RangeFull) -> &[u64] {
                &self.0[..]
            }
        }

        impl PartialOrd for $name {
            #[inline]
            fn partial_cmp(&self, other: &$name) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(&other))
            }
        }

        impl Ord for $name {
            #[inline]
            fn cmp(&self, other: &$name) -> ::std::cmp::Ordering {
                // We need to manually implement ordering because we use little-endian
                // and the auto derive is a lexicographic ordering(i.e. memcmp)
                // which with numbers is equivilant to big-endian
                for i in 0..$n_words {
                    if self[$n_words - 1 - i] < other[$n_words - 1 - i] {
                        return ::std::cmp::Ordering::Less;
                    }
                    if self[$n_words - 1 - i] > other[$n_words - 1 - i] {
                        return ::std::cmp::Ordering::Greater;
                    }
                }
                ::std::cmp::Ordering::Equal
            }
        }

        impl<T> ::core::ops::Add<T> for $name
        where
            T: Into<$name>,
        {
            type Output = $name;

            fn add(self, other: T) -> $name {
                let $name(ref me) = self;
                let $name(ref you) = other.into();
                let mut ret = [0u64; $n_words];
                let mut carry = [0u64; $n_words];
                let mut b_carry = false;
                for i in 0..$n_words {
                    ret[i] = me[i].wrapping_add(you[i]);
                    if i < $n_words - 1 && ret[i] < me[i] {
                        carry[i + 1] = 1;
                        b_carry = true;
                    }
                }
                if b_carry {
                    $name(ret) + $name(carry)
                } else {
                    $name(ret)
                }
            }
        }

        impl<T> ::core::ops::Sub<T> for $name
        where
            T: Into<$name>,
        {
            type Output = $name;

            #[inline]
            fn sub(self, other: T) -> $name {
                self + !other.into() + $name::ONE
            }
        }

        impl<T> ::core::ops::Mul<T> for $name
        where
            T: Into<$name>,
        {
            type Output = $name;

            fn mul(self, other: T) -> $name {
                let other = other.into();
                let mut me = $name::ZERO;
                // TODO: be more efficient about this
                for i in 0..(2 * $n_words) {
                    let to_mul = (other >> (32 * i)).low_u32();
                    me = me + (self.mul_u32(to_mul) << (32 * i));
                }
                me
            }
        }

        impl<T> ::core::ops::Div<T> for $name
        where
            T: Into<$name>,
        {
            type Output = $name;

            fn div(self, other: T) -> $name {
                self.div_rem(other.into()).0
            }
        }

        impl<T> ::core::ops::Rem<T> for $name
        where
            T: Into<$name>,
        {
            type Output = $name;

            fn rem(self, other: T) -> $name {
                self.div_rem(other.into()).1
            }
        }

        impl<T> ::core::ops::BitAnd<T> for $name
        where
            T: Into<$name>,
        {
            type Output = $name;

            #[inline]
            fn bitand(self, other: T) -> $name {
                let $name(ref arr1) = self;
                let $name(ref arr2) = other.into();
                let mut ret = [0u64; $n_words];
                for i in 0..$n_words {
                    ret[i] = arr1[i] & arr2[i];
                }
                $name(ret)
            }
        }

        impl<T> ::core::ops::BitXor<T> for $name
        where
            T: Into<$name>,
        {
            type Output = $name;

            #[inline]
            fn bitxor(self, other: T) -> $name {
                let $name(ref arr1) = self;
                let $name(ref arr2) = other.into();
                let mut ret = [0u64; $n_words];
                for i in 0..$n_words {
                    ret[i] = arr1[i] ^ arr2[i];
                }
                $name(ret)
            }
        }

        impl<T> ::core::ops::BitOr<T> for $name
        where
            T: Into<$name>,
        {
            type Output = $name;

            #[inline]
            fn bitor(self, other: T) -> $name {
                let $name(ref arr1) = self;
                let $name(ref arr2) = other.into();
                let mut ret = [0u64; $n_words];
                for i in 0..$n_words {
                    ret[i] = arr1[i] | arr2[i];
                }
                $name(ret)
            }
        }

        impl ::core::ops::Shl<usize> for $name {
            type Output = $name;

            fn shl(self, shift: usize) -> $name {
                let $name(ref original) = self;
                let mut ret = [0u64; $n_words];
                let word_shift = shift / 64;
                let bit_shift = shift % 64;
                for i in 0..$n_words {
                    // Shift
                    if bit_shift < 64 && i + word_shift < $n_words {
                        ret[i + word_shift] += original[i] << bit_shift;
                    }
                    // Carry
                    if bit_shift > 0 && i + word_shift + 1 < $n_words {
                        ret[i + word_shift + 1] += original[i] >> (64 - bit_shift);
                    }
                }
                $name(ret)
            }
        }

        impl ::core::ops::Shr<usize> for $name {
            type Output = $name;

            fn shr(self, shift: usize) -> $name {
                let $name(ref original) = self;
                let mut ret = [0u64; $n_words];
                let word_shift = shift / 64;
                let bit_shift = shift % 64;
                for i in word_shift..$n_words {
                    // Shift
                    ret[i - word_shift] += original[i] >> bit_shift;
                    // Carry
                    if bit_shift > 0 && i < $n_words - 1 {
                        ret[i - word_shift] += original[i + 1] << (64 - bit_shift);
                    }
                }
                $name(ret)
            }
        }

        impl ::core::ops::Not for $name {
            type Output = $name;

            #[inline]
            fn not(self) -> $name {
                let $name(ref arr) = self;
                let mut ret = [0u64; $n_words];
                for i in 0..$n_words {
                    ret[i] = !arr[i];
                }
                $name(ret)
            }
        }

        impl $crate::num::BitArray for $name {
            #[inline]
            fn bit(&self, index: usize) -> bool {
                let &$name(ref arr) = self;
                arr[index / 64] & (1 << (index % 64)) != 0
            }

            #[inline]
            fn bit_slice(&self, start: usize, end: usize) -> $name {
                (*self >> start).mask(end - start)
            }

            #[inline]
            fn mask(&self, n: usize) -> $name {
                let &$name(ref arr) = self;
                let mut ret = [0; $n_words];
                for i in 0..$n_words {
                    if n >= 0x40 * (i + 1) {
                        ret[i] = arr[i];
                    } else {
                        ret[i] = arr[i] & ((1 << (n - 0x40 * i)) - 1);
                        break;
                    }
                }
                $name(ret)
            }

            #[inline]
            fn trailing_zeros(&self) -> usize {
                let &$name(ref arr) = self;
                for i in 0..($n_words - 1) {
                    if arr[i] > 0 {
                        return (0x40 * i) + arr[i].trailing_zeros() as usize;
                    }
                }
                (0x40 * ($n_words - 1)) + arr[$n_words - 1].trailing_zeros() as usize
            }
        }

        #[cfg(feature = "std")]
        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let &$name(ref data) = self;
                write!(f, "0x")?;
                for ch in data.iter().rev() {
                    write!(f, "{:016x}", ch)?;
                }
                Ok(())
            }
        }

        #[cfg(feature = "std")]
        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::std::fmt::Debug::fmt(self, f)
            }
        }

        #[cfg(feature = "serde")]
        impl $crate::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: $crate::serde::Serializer,
            {
                use $crate::hex::ToHex;
                let bytes = self.to_be_bytes();
                if serializer.is_human_readable() {
                    serializer.serialize_str(&bytes.to_hex())
                } else {
                    serializer.serialize_bytes(&bytes)
                }
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> $crate::serde::Deserialize<'de> for $name {
            fn deserialize<D: $crate::serde::Deserializer<'de>>(
                deserializer: D,
            ) -> Result<Self, D::Error> {
                use ::std::fmt;
                use $crate::hex::FromHex;
                use $crate::serde::de;
                struct Visitor;
                impl<'de> de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(
                            f,
                            "{} bytes or a hex string with {} characters",
                            $n_words * 8,
                            $n_words * 8 * 2
                        )
                    }

                    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        let bytes = Vec::from_hex(s)
                            .map_err(|_| de::Error::invalid_value(de::Unexpected::Str(s), &self))?;
                        $name::from_be_slice(&bytes)
                            .map_err(|_| de::Error::invalid_length(bytes.len() * 2, &self))
                    }

                    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        $name::from_be_slice(bytes)
                            .map_err(|_| de::Error::invalid_length(bytes.len(), &self))
                    }
                }

                if deserializer.is_human_readable() {
                    deserializer.deserialize_str(Visitor)
                } else {
                    deserializer.deserialize_bytes(Visitor)
                }
            }
        }
    };
}

construct_uint!(u256, 4);
construct_uint!(u512, 8);
construct_uint!(u1024, 16);

/// Invalid slice length
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
/// Invalid slice length
pub struct ParseLengthError {
    /// The length of the slice de-facto
    pub actual: usize,
    /// The required length of the slice
    pub expected: usize,
}

#[cfg(feature = "std")]
impl ::std::fmt::Display for ParseLengthError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(
            f,
            "Invalid length: got {}, expected {}",
            self.actual, self.expected
        )
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for ParseLengthError {}

impl u256 {
    /// Increment by 1
    #[inline]
    pub fn increment(&mut self) {
        let &mut u256(ref mut arr) = self;
        arr[0] += 1;
        if arr[0] == 0 {
            arr[1] += 1;
            if arr[1] == 0 {
                arr[2] += 1;
                if arr[2] == 0 {
                    arr[3] += 1;
                }
            }
        }
    }
}

impl u512 {
    /// Increment by 1
    #[inline]
    pub fn increment(&mut self) {
        let &mut u512(ref mut arr) = self;
        arr[0] += 1;
        if arr[0] == 0 {
            arr[1] += 1;
            if arr[1] == 0 {
                arr[2] += 1;
                if arr[2] == 0 {
                    arr[3] += 1;
                    if arr[3] == 0 {
                        arr[4] += 1;
                        if arr[4] == 0 {
                            arr[5] += 1;
                            if arr[5] == 0 {
                                arr[6] += 1;
                                if arr[6] == 0 {
                                    arr[7] += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl u1024 {
    /// Increment by 1
    #[inline]
    pub fn increment(&mut self) {
        let &mut u1024(ref mut arr) = self;
        arr[0] += 1;
        if arr[0] == 0 {
            arr[1] += 1;
            if arr[1] == 0 {
                arr[2] += 1;
                if arr[2] == 0 {
                    arr[3] += 1;
                    if arr[3] == 0 {
                        arr[4] += 1;
                        if arr[4] == 0 {
                            arr[5] += 1;
                            if arr[5] == 0 {
                                arr[6] += 1;
                                if arr[6] == 0 {
                                    arr[7] += 1;
                                    if arr[7] == 0 {
                                        arr[8] += 1;
                                        if arr[8] == 0 {
                                            arr[9] += 1;
                                            if arr[9] == 0 {
                                                arr[10] += 1;
                                                if arr[10] == 0 {
                                                    arr[11] += 1;
                                                    if arr[11] == 0 {
                                                        arr[12] += 1;
                                                        if arr[12] == 0 {
                                                            arr[13] += 1;
                                                            if arr[13] == 0 {
                                                                arr[14] += 1;
                                                                if arr[14] == 0 {
                                                                    arr[15] += 1;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused)]

    use super::*;

    construct_uint!(Uint128, 2);

    #[test]
    fn ubit_test() {
        let mut u_2 = u2::try_from(*u2::MAX).unwrap();
        let mut u_3 = u3::try_from(*u3::MAX).unwrap();
        let mut u_4 = u4::try_from(*u4::MAX).unwrap();
        let mut u_5 = u5::try_from(*u5::MAX).unwrap();
        let mut u_6 = u6::try_from(*u6::MAX).unwrap();
        let mut u_7 = u7::try_from(*u7::MAX).unwrap();
        let mut u_24 = u24::try_from(*u24::MAX).unwrap();

        assert_eq!(u_2, u2::with(3));
        assert_eq!(u_3, u3::with(7));
        assert_eq!(u_4, u4::with(15));
        assert_eq!(u_5, u5::with(31));
        assert_eq!(u_6, u6::with(63));
        assert_eq!(u_7, u7::with(127));

        assert_eq!(*u_2, 3u8);
        assert_eq!(*u_3, 7u8);
        assert_eq!(*u_4, 15u8);
        assert_eq!(*u_5, 31u8);
        assert_eq!(*u_6, 63u8);
        assert_eq!(*u_7, 127u8);
        assert_eq!(*u_24, (1 << 24) - 1);

        u_2 -= 1;
        u_3 -= 1;
        u_4 -= 1;
        u_5 -= 1;
        u_6 -= 1;
        u_7 -= 1;
        u_24 -= 1;

        assert_eq!(*u_2, 2u8);
        assert_eq!(*u_3, 6u8);
        assert_eq!(*u_4, 14u8);
        assert_eq!(*u_5, 30u8);
        assert_eq!(*u_6, 62u8);
        assert_eq!(*u_7, 126u8);
        assert_eq!(*u_24, (1 << 24) - 2);

        u_2 /= 2;
        u_2 *= 2;
        u_2 += 1;

        u_3 /= 2;
        u_3 *= 2;
        u_3 += 1;

        u_4 /= 2;
        u_4 *= 2;
        u_4 += 1;

        u_5 /= 2;
        u_5 *= 2;
        u_5 += 1;

        u_6 /= 2;
        u_6 *= 2;
        u_6 += 1;

        u_7 /= 2;
        u_7 *= 2;
        u_7 += 1;

        u_24 /= 2;
        u_24 *= 2;
        u_24 += 1;

        assert_eq!(*u_2, 3u8);
        assert_eq!(*u_3, 7u8);
        assert_eq!(*u_4, 15u8);
        assert_eq!(*u_5, 31u8);
        assert_eq!(*u_6, 63u8);
        assert_eq!(*u_7, 127u8);
        assert_eq!(*u_24, (1 << 24) - 1);

        assert_eq!(*u_2 % 2, 1);
        assert_eq!(*u_3 % 2, 1);
        assert_eq!(*u_4 % 2, 1);
        assert_eq!(*u_5 % 2, 1);
        assert_eq!(*u_6 % 2, 1);
        assert_eq!(*u_7 % 2, 1);
        assert_eq!(*u_24 % 2, 1);
    }

    #[test]
    #[should_panic(expected = "ValueOverflow { max: 3, value: 4 }")]
    fn u2_overflow_test() {
        u2::try_from(4).unwrap();
    }

    #[test]
    #[should_panic(expected = "ValueOverflow { max: 7, value: 8 }")]
    fn u3_overflow_test() {
        u3::try_from(8).unwrap();
    }

    #[test]
    #[should_panic(expected = "ValueOverflow { max: 15, value: 16 }")]
    fn u4_overflow_test() {
        u4::try_from(16).unwrap();
    }

    #[test]
    #[should_panic(expected = "ValueOverflow { max: 31, value: 32 }")]
    fn u5_overflow_test() {
        u5::try_from(32).unwrap();
    }

    #[test]
    #[should_panic(expected = "ValueOverflow { max: 63, value: 64 }")]
    fn u6_overflow_test() {
        u6::try_from(64).unwrap();
    }

    #[test]
    #[should_panic(expected = "ValueOverflow { max: 127, value: 128 }")]
    fn u7_overflow_test() {
        u7::try_from(128).unwrap();
    }

    #[test]
    #[should_panic(expected = "ValueOverflow { max: 16777215, value: 16777216 }")]
    fn u24_overflow_test() {
        u24::try_from(1 << 24).unwrap();
    }

    #[test]
    fn u256_bits_test() {
        assert_eq!(u256::from(255u64).bits_required(), 8);
        assert_eq!(u256::from(256u64).bits_required(), 9);
        assert_eq!(u256::from(300u64).bits_required(), 9);
        assert_eq!(u256::from(60000u64).bits_required(), 16);
        assert_eq!(u256::from(70000u64).bits_required(), 17);

        // Try to read the following lines out loud quickly
        let mut shl = u256::from(70000u64);
        shl = shl << 100;
        assert_eq!(shl.bits_required(), 117);
        shl = shl << 100;
        assert_eq!(shl.bits_required(), 217);
        shl = shl << 100;
        assert_eq!(shl.bits_required(), 0);

        // Bit set check
        assert!(!u256::from(10u64).bit(0));
        assert!(u256::from(10u64).bit(1));
        assert!(!u256::from(10u64).bit(2));
        assert!(u256::from(10u64).bit(3));
        assert!(!u256::from(10u64).bit(4));
    }

    #[test]
    fn u256_display_test() {
        assert_eq!(
            format!("{}", u256::from(0xDEADBEEFu64)),
            "0x00000000000000000000000000000000000000000000000000000000deadbeef"
        );
        assert_eq!(
            format!("{}", u256::from(::core::u64::MAX)),
            "0x000000000000000000000000000000000000000000000000ffffffffffffffff"
        );

        let max_val = u256([
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
        ]);
        assert_eq!(
            format!("{}", max_val),
            "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        );
    }

    #[test]
    fn u256_comp_test() {
        let small = u256([10u64, 0, 0, 0]);
        let big = u256([0x8C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]);
        let bigger = u256([0x9C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]);
        let biggest = u256([0x5C8C3EE70C644118u64, 0x0209E7378231E632, 0, 1]);

        assert!(small < big);
        assert!(big < bigger);
        assert!(bigger < biggest);
        assert!(bigger <= biggest);
        assert!(biggest <= biggest);
        assert!(bigger >= big);
        assert!(bigger >= small);
        assert!(small <= small);
    }

    #[test]
    fn uint_from_be_bytes() {
        assert_eq!(
            Uint128::from_be_bytes([
                0x1b, 0xad, 0xca, 0xfe, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xaf, 0xba, 0xbe, 0x2b, 0xed,
                0xfe, 0xed
            ]),
            Uint128([0xdeafbabe2bedfeed, 0x1badcafedeadbeef])
        );

        assert_eq!(
            u256::from_be_bytes([
                0x1b, 0xad, 0xca, 0xfe, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xaf, 0xba, 0xbe, 0x2b, 0xed,
                0xfe, 0xed, 0xba, 0xad, 0xf0, 0x0d, 0xde, 0xfa, 0xce, 0xda, 0x11, 0xfe, 0xd2, 0xba,
                0xd1, 0xc0, 0xff, 0xe0
            ]),
            u256([
                0x11fed2bad1c0ffe0,
                0xbaadf00ddefaceda,
                0xdeafbabe2bedfeed,
                0x1badcafedeadbeef
            ])
        );
    }

    #[test]
    fn uint_from_le_bytes() {
        let mut be = [
            0x1b, 0xad, 0xca, 0xfe, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xaf, 0xba, 0xbe, 0x2b, 0xed,
            0xfe, 0xed,
        ];
        be.reverse();
        assert_eq!(
            Uint128::from_le_bytes(be),
            Uint128([0xdeafbabe2bedfeed, 0x1badcafedeadbeef])
        );

        let mut be = [
            0x1b, 0xad, 0xca, 0xfe, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xaf, 0xba, 0xbe, 0x2b, 0xed,
            0xfe, 0xed, 0xba, 0xad, 0xf0, 0x0d, 0xde, 0xfa, 0xce, 0xda, 0x11, 0xfe, 0xd2, 0xba,
            0xd1, 0xc0, 0xff, 0xe0,
        ];
        be.reverse();
        assert_eq!(
            u256::from_le_bytes(be),
            u256([
                0x11fed2bad1c0ffe0,
                0xbaadf00ddefaceda,
                0xdeafbabe2bedfeed,
                0x1badcafedeadbeef
            ])
        );
    }

    #[test]
    fn uint_to_be_bytes() {
        assert_eq!(
            Uint128([0xdeafbabe2bedfeed, 0x1badcafedeadbeef]).to_be_bytes(),
            [
                0x1b, 0xad, 0xca, 0xfe, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xaf, 0xba, 0xbe, 0x2b, 0xed,
                0xfe, 0xed
            ]
        );

        assert_eq!(
            u256([
                0x11fed2bad1c0ffe0,
                0xbaadf00ddefaceda,
                0xdeafbabe2bedfeed,
                0x1badcafedeadbeef
            ])
            .to_be_bytes(),
            [
                0x1b, 0xad, 0xca, 0xfe, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xaf, 0xba, 0xbe, 0x2b, 0xed,
                0xfe, 0xed, 0xba, 0xad, 0xf0, 0x0d, 0xde, 0xfa, 0xce, 0xda, 0x11, 0xfe, 0xd2, 0xba,
                0xd1, 0xc0, 0xff, 0xe0
            ]
        );
    }

    #[test]
    fn uint_to_le_bytes() {
        assert_eq!(
            Uint128([0xdeafbabe2bedfeed, 0x1badcafedeadbeef]).to_le_bytes(),
            [
                0xed, 0xfe, 0xed, 0x2b, 0xbe, 0xba, 0xaf, 0xde, 0xef, 0xbe, 0xad, 0xde, 0xfe, 0xca,
                0xad, 0x1b
            ]
        );

        assert_eq!(
            u256([
                0x11fed2bad1c0ffe0,
                0xbaadf00ddefaceda,
                0xdeafbabe2bedfeed,
                0x1badcafedeadbeef
            ])
            .to_le_bytes(),
            [
                0xe0, 0xff, 0xc0, 0xd1, 0xba, 0xd2, 0xfe, 0x11, 0xda, 0xce, 0xfa, 0xde, 0x0d, 0xf0,
                0xad, 0xba, 0xed, 0xfe, 0xed, 0x2b, 0xbe, 0xba, 0xaf, 0xde, 0xef, 0xbe, 0xad, 0xde,
                0xfe, 0xca, 0xad, 0x1b,
            ]
        );
    }

    #[test]
    fn bigint_min_max() {
        assert_eq!(u256::MIN.as_inner(), &[0u64; 4]);
        assert_eq!(u512::MIN.as_inner(), &[0u64; 8]);
        assert_eq!(u1024::MIN.as_inner(), &[0u64; 16]);
        assert_eq!(u256::MAX.as_inner(), &[::core::u64::MAX; 4]);
        assert_eq!(u512::MAX.as_inner(), &[::core::u64::MAX; 8]);
        assert_eq!(u1024::MAX.as_inner(), &[::core::u64::MAX; 16]);
        assert_eq!(u256::BITS, 4 * 64);
        assert_eq!(u512::BITS, 8 * 64);
        assert_eq!(u1024::BITS, 16 * 64);
    }

    #[test]
    fn u256_arithmetic_test() {
        let init = u256::from(0xDEADBEEFDEADBEEFu64);
        let copy = init;

        let add = init + copy;
        assert_eq!(add, u256([0xBD5B7DDFBD5B7DDEu64, 1, 0, 0]));
        // Bitshifts
        let shl = add << 88;
        assert_eq!(shl, u256([0u64, 0xDFBD5B7DDE000000, 0x1BD5B7D, 0]));
        let shr = shl >> 40;
        assert_eq!(shr, u256([0x7DDE000000000000u64, 0x0001BD5B7DDFBD5B, 0, 0]));
        // Increment
        let mut incr = shr;
        incr.increment();
        assert_eq!(
            incr,
            u256([0x7DDE000000000001u64, 0x0001BD5B7DDFBD5B, 0, 0])
        );
        // Subtraction
        let sub = incr - init;
        assert_eq!(sub, u256([0x9F30411021524112u64, 0x0001BD5B7DDFBD5A, 0, 0]));
        // Multiplication
        let mult = sub.mul_u32(300);
        assert_eq!(
            mult,
            u256([0x8C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0])
        );
        // Division
        assert_eq!(u256::from(105u64) / u256::from(5u64), u256::from(21u64));
        let div = mult / u256::from(300u64);
        assert_eq!(div, u256([0x9F30411021524112u64, 0x0001BD5B7DDFBD5A, 0, 0]));

        assert_eq!(u256::from(105u64) % u256::from(5u64), u256::from(0u64));
        assert_eq!(
            u256::from(35498456u64) % u256::from(3435u64),
            u256::from(1166u64)
        );
        let rem_src = mult * u256::from(39842u64) + u256::from(9054u64);
        assert_eq!(rem_src % u256::from(39842u64), u256::from(9054u64));
        // TODO: bit inversion
    }

    #[test]
    fn mul_u32_test() {
        let u64_val = u256::from(0xDEADBEEFDEADBEEFu64);

        let u96_res = u64_val.mul_u32(0xFFFFFFFF);
        let u128_res = u96_res.mul_u32(0xFFFFFFFF);
        let u160_res = u128_res.mul_u32(0xFFFFFFFF);
        let u192_res = u160_res.mul_u32(0xFFFFFFFF);
        let u224_res = u192_res.mul_u32(0xFFFFFFFF);
        let u256_res = u224_res.mul_u32(0xFFFFFFFF);

        assert_eq!(u96_res, u256([0xffffffff21524111u64, 0xDEADBEEE, 0, 0]));
        assert_eq!(
            u128_res,
            u256([0x21524111DEADBEEFu64, 0xDEADBEEE21524110, 0, 0])
        );
        assert_eq!(
            u160_res,
            u256([0xBD5B7DDD21524111u64, 0x42A4822200000001, 0xDEADBEED, 0])
        );
        assert_eq!(
            u192_res,
            u256([
                0x63F6C333DEADBEEFu64,
                0xBD5B7DDFBD5B7DDB,
                0xDEADBEEC63F6C334,
                0
            ])
        );
        assert_eq!(
            u224_res,
            u256([
                0x7AB6FBBB21524111u64,
                0xFFFFFFFBA69B4558,
                0x854904485964BAAA,
                0xDEADBEEB
            ])
        );
        assert_eq!(
            u256_res,
            u256([
                0xA69B4555DEADBEEFu64,
                0xA69B455CD41BB662,
                0xD41BB662A69B4550,
                0xDEADBEEAA69B455C
            ])
        );
    }

    #[test]
    fn multiplication_test() {
        let u64_val = u256::from(0xDEADBEEFDEADBEEFu64);

        let u128_res = u64_val * u64_val;

        assert_eq!(
            u128_res,
            u256([0x048D1354216DA321u64, 0xC1B1CD13A4D13D46, 0, 0])
        );

        let u256_res = u128_res * u128_res;

        assert_eq!(
            u256_res,
            u256([
                0xF4E166AAD40D0A41u64,
                0xF5CF7F3618C2C886u64,
                0x4AFCFF6F0375C608u64,
                0x928D92B4D7F5DF33u64
            ])
        );
    }

    #[test]
    fn u256_bitslice_test() {
        let init = u256::from(0xDEADBEEFDEADBEEFu64);
        let add = init + (init << 64);
        assert_eq!(add.bit_slice(64, 128), init);
        assert_eq!(add.mask(64), init);
    }

    #[test]
    fn u256_extreme_bitshift_test() {
        // Shifting a u64 by 64 bits gives an undefined value, so make sure that
        // we're doing the Right Thing here
        let init = u256::from(0xDEADBEEFDEADBEEFu64);

        assert_eq!(init << 64, u256([0, 0xDEADBEEFDEADBEEF, 0, 0]));
        let add = (init << 64) + init;
        assert_eq!(add, u256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
        assert_eq!(
            add >> 0,
            u256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0])
        );
        assert_eq!(
            add << 0,
            u256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0])
        );
        assert_eq!(add >> 64, u256([0xDEADBEEFDEADBEEF, 0, 0, 0]));
        assert_eq!(
            add << 64,
            u256([0, 0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0])
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn u256_serde_test() {
        let check = |uint, hex| {
            let json = format!("\"{}\"", hex);
            assert_eq!(::serde_json::to_string(&uint).unwrap(), json);
            assert_eq!(::serde_json::from_str::<u256>(&json).unwrap(), uint);

            let bin_encoded = ::bincode::serialize(&uint).unwrap();
            let bin_decoded: u256 = ::bincode::deserialize(&bin_encoded).unwrap();
            assert_eq!(bin_decoded, uint);
        };

        check(
            u256::from(0u64),
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        check(
            u256::from(0xDEADBEEFu64),
            "00000000000000000000000000000000000000000000000000000000deadbeef",
        );
        check(
            u256([0xaa11, 0xbb22, 0xcc33, 0xdd44]),
            "000000000000dd44000000000000cc33000000000000bb22000000000000aa11",
        );
        check(
            u256([
                u64::max_value(),
                u64::max_value(),
                u64::max_value(),
                u64::max_value(),
            ]),
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        );
        check(
            u256([
                0xA69B4555DEADBEEF,
                0xA69B455CD41BB662,
                0xD41BB662A69B4550,
                0xDEADBEEAA69B455C,
            ]),
            "deadbeeaa69b455cd41bb662a69b4550a69b455cd41bb662a69b4555deadbeef",
        );

        assert!(::serde_json::from_str::<u256>(
            "\"fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffg\""
        )
        .is_err()); // invalid char
        assert!(::serde_json::from_str::<u256>(
            "\"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\""
        )
        .is_err()); // invalid length
        assert!(::serde_json::from_str::<u256>(
            "\"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\""
        )
        .is_err()); // invalid length
    }
}
