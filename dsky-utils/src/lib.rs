#![cfg_attr(not(feature = "std"), no_std)]

use substrate_fixed::types::I9F23;

pub trait IntDiv<RHS = Self> {
    fn integer_division_u16(self, rhs: RHS) -> u16;
    fn integer_division_u32(self, rhs: RHS) -> u32;
}

pub trait Signed { 
    fn abs(self) -> Self;
}

pub trait CastToType {
    fn to_u32(self) -> u32;
}

// Want to change Coord type => impl trait for it here
impl IntDiv for I9F23 {
    fn integer_division_u16(self, rhs: I9F23) -> u16 {
        (self / rhs).to_num::<u16>()
    }

    fn integer_division_u32(self, rhs: I9F23) -> u32 {
        (self / rhs).to_num::<u32>()
    }
}

impl Signed for I9F23 {
    fn abs(self) -> Self {
        self.abs()
    }
}

impl CastToType for I9F23 {
    // TODO make this function universal
    // TODO fix situation when int part consists of 1 number
    // '100' is the precision for the first two numbers of the fractional part
    fn to_u32(self) -> u32 {
        let result: u32 = self.to_num::<u32>() * 100;
        let first_number_from_frac: u32 = ((self - I9F23::from_num(self.to_num::<u32>())) * 10).to_num::<u32>() * 10;
        let second_number_from_frac: u32 = ((self - I9F23::from_num(self.to_num::<u32>())) * 100).to_num::<u32>() % 10;

        result + first_number_from_frac + second_number_from_frac
    }
}
