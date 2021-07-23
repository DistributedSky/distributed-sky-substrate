#![cfg_attr(not(feature = "std"), no_std)]

use substrate_fixed::{types::{I10F22, I42F22}, traits::FromFixed};
// Set of traits, required for Coord struct, used in maps pallet

pub trait IntDiv<RHS = Self> {
    fn integer_division_u16(self, rhs: RHS) -> u16;
    fn integer_division_u32(self, rhs: RHS) -> u32;
}

pub trait Signed { 
    fn abs(self) -> Self;
    fn signum(self) -> Self;
}

pub trait FromRaw {
    fn from_raw(input: i32) -> Self;
}

pub trait CastToType {
    fn to_u32_with_frac_part(self, cell_size: u32, max_digits_in_frac_part: u8) -> u32;
}

// TODO consider naming of two traits, as they are used in pair
pub trait ToBigCoord {
    type Output;
    fn try_into(self) -> Self::Output;
}

pub trait FromBigCoord {
    type Output;
    fn try_from(self) -> Self::Output;
}

pub trait GetEpsilon {
    fn get_epsilon() -> Self;
}

// Here comes the implementations 
// Want to change Coord type => impl trait for it here
impl IntDiv for I10F22 {
    fn integer_division_u16(self, rhs: I10F22) -> u16 {
        (self / rhs).to_num::<u16>()
    }

    fn integer_division_u32(self, rhs: I10F22) -> u32 {
        (self / rhs).to_num::<u32>()
    }
}

impl FromRaw for I10F22 {
    fn from_raw(input: i32) -> Self {
        I10F22::from_bits(input)
    }
}

impl Signed for I10F22 {
    fn abs(self) -> Self {
        self.abs()
    }
    /// returns sign (-1, 1, 0)
    fn signum(self) -> Self {
        self.signum()
    }
}

impl CastToType for I10F22 {
    /// Converts a number of type I10F22 to u32 with a shift of n digits of the fractional part.
    /// It is required minimum 3 non-zero (simultaneous) digits after the point
    /// due to the type features.
    fn to_u32_with_frac_part(self, coefficient: u32, max_digits_in_frac_part: u8) -> u32 {
        let mut integer_part: u32 = self.to_num::<u32>();
        let mut degree_counter: u8 = 0;

        while max_digits_in_frac_part > degree_counter {
            integer_part /= 10;
            degree_counter += 1;
        }

        let base: u32 = 10;
        integer_part = self.to_num::<u32>() * base.pow(degree_counter as u32) * coefficient;
        let frac_part: u32 = ((self - I10F22::from_num(self.to_num::<u32>()))
                                * base.pow(max_digits_in_frac_part as u32) as i32).to_num::<u32>();

        integer_part + frac_part
    }
}

impl ToBigCoord for I10F22 {
    type Output = I42F22;
    fn try_into(self) -> Self::Output {
        self.into()
    }
}

// TODO handle possible errors through checked_from_fixed()
impl FromBigCoord for I42F22 {
    type Output = I10F22;
    fn try_from(self) -> Self::Output {
        I10F22::from_fixed(self)
    }
}

impl GetEpsilon for I42F22 {
    fn get_epsilon() -> I42F22 {
        I42F22::from_num(0.00001f64)
    }
}
