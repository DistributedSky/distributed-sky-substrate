#![cfg_attr(not(feature = "std"), no_std)]

use substrate_fixed::types::I9F23;

pub trait MathUtils<RHS = Self> {
    fn integer_divide(self, rhs: RHS) -> u16;
    fn abs(self) -> Self;
}

// Want to change Coord type => impl trait for it here
impl MathUtils for I9F23 {
    fn integer_divide(self, rhs: I9F23) -> u16 {
        (self / rhs).to_num::<u16>()
    }

    fn abs(self) -> Self {
        self.abs()
    }
}
