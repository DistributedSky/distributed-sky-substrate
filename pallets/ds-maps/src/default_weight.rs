use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

impl crate::WeightInfo for () {
    fn root_add() -> Weight {
        1_000_000_u64.saturating_add(DbWeight::get().writes(1))
    }
    fn zone_add() -> Weight {
        1_000_000_u64.saturating_add(DbWeight::get().writes(1))
    }
    fn root_remove() -> Weight {
        1_000_000_u64.saturating_add(DbWeight::get().writes(1))
    }   
    fn zone_remove() -> Weight {
        1_000_000_u64.saturating_add(DbWeight::get().writes(1))
    }   
    fn change_area_type() -> Weight {
        100_000_u64.saturating_add(DbWeight::get().writes(1))
    }   

}
