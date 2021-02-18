use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

impl crate::WeightInfo for () {
    fn register_zone() -> Weight {
        (1_000_000 as Weight).saturating_add(DbWeight::get().writes(1))
    }
}
