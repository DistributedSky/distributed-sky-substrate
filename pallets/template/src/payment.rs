use frame_support::sp_runtime::{traits::Convert, Perquintill};
use pallet_transaction_payment::{Multiplier, MultiplierUpdate};

/// Replace default pallet_transaction_payment MultiplierUpdater
/// that make multiplier equal to 1.
/// transaction fee = WeightToFe(ExtrinsicBaseWeight) + (tx_len * TransactionByteFee) + multiplier * WeightToFee(weight)
pub struct IdentityMultiplierUpdater {}

impl Convert<Multiplier, Multiplier> for IdentityMultiplierUpdater {
    fn convert(previous: Multiplier) -> Multiplier {
        previous
    }
}

impl MultiplierUpdate for IdentityMultiplierUpdater {
    fn min() -> Multiplier {
        Default::default()
    }
    fn target() -> Perquintill {
        Default::default()
    }
    fn variability() -> Multiplier {
        Default::default()
    }
}
