#![cfg_attr(not(feature = "std"), no_std)]

use substrate_fixed::types::I9F23;

pub trait IntDiv<RHS = Self> {
    fn integer_divide(self, rhs: RHS) -> u16;
}

pub trait Signed { 
    fn abs(self) -> Self;
}

// Want to change Coord type => impl trait for it here
impl IntDiv for I9F23 {
    fn integer_divide(self, rhs: I9F23) -> u16 {
        (self / rhs).to_num::<u16>()
    }
}

impl Signed for I9F23 {
    fn abs(self) -> Self {
        self.abs()
    }
}