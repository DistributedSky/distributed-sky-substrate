use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

impl crate::WeightInfo for () {
    fn account_add() -> Weight {
        (1_000_000 as Weight).saturating_add(DbWeight::get().writes(1))
    }

    fn account_disable() -> Weight {
        (1_000_000 as Weight).saturating_add(DbWeight::get().reads_writes(1, 1))
    }

    fn register_pilot() -> Weight {
        (1_000_000 as Weight)
            .saturating_add(DbWeight::get().writes(1 as Weight))
    }
}
