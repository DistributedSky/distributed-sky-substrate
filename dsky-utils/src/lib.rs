#![cfg_attr(not(feature = "std"), no_std)]

use substrate_fixed::types::I10F22;

pub trait IntDiv<RHS = Self> {
    fn integer_division_u16(self, rhs: RHS) -> u16;
    fn integer_division_u32(self, rhs: RHS) -> u32;
}

pub trait Signed { 
    fn abs(self) -> Self;
}

pub trait FromRaw {
    fn from_raw(input: i32) -> Self;
}

pub trait CastToType {
    fn to_u32_with_frac_part(self, cell_size: u32, max_digits_in_frac_part: u8) -> u32;
}

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
