#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    sp_runtime::{traits::{
            Member, MaybeSerializeDeserialize, Zero
        },
    },
    traits::{
        Currency, Get, LockableCurrency,
        OnKilledAccount,
    },
    weights::{Weight},
    Parameter,
};
use frame_system::ensure_signed;

mod default_weight;
#[cfg(test)]
mod mock;
#[cfg(feature = "payment")]
mod payment;
#[cfg(test)]
mod tests;

//Not sure, are those derives required?
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub enum ZoneType {
    /// Forbidden type zone
    Red,
    /// Available for safe flights
    Green,
    /// Owns zones
    Parent,
    // 
}

impl Default for ZoneType {
    fn default() -> Self {
        ZoneType::Green
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default)]
pub struct Zone<Coordinate, ZoneNumber> {
    pub coordinates: Coordinate,
    pub obstacle_type: ZoneType,
    pub zone_id: ZoneNumber,
}

impl<
    Coordinate: Default,
    ObstacleType: Default,
    > Zone<Coordinate, ObstacleType>
{
    pub fn zone_is(&self, zone: ZoneType) -> bool{
        !(self.obstacle_type == zone)
    }
}

pub type ZoneOf<T> = Zone<<T as Trait>::Coordinate, <T as Trait>::ZoneNumber>;


/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait + pallet_timestamp::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    // Describe pallet constants.
    // Lean more https://substrate.dev/docs/en/knowledgebase/runtime/metadata
    type WeightInfo: WeightInfo;
    // new types,fixes required
    type Coordinate: Default + Parameter;
    type ZoneNumber: Default + Zero + MaybeSerializeDeserialize + Parameter + Member; 
}    
pub trait WeightInfo {
    fn register_zone() -> Weight;
}

decl_storage!{
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as DSMapsModule {
        CityMap get(fn map_data): 
            map hasher(blake2_128_concat) T::ZoneNumber => ZoneOf<T>;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        ZoneNumber = <T as Trait>::ZoneNumber,
        ObstacleType = ZoneType,
    {
        // Event documentation should end with an array that provides descriptive names for event parameters.
        /// New account has been created [zone number, its type]
        ZoneCreated(ZoneNumber, ObstacleType),
    }
);

// Errors inform users that something went wrong.
// learn more https://substrate.dev/docs/en/knowledgebase/runtime/errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        NoneValue,
        /// Operation is not valid
        InvalidAction,
        /// Incorrect data provided
        InvalidData,
        /// Origin do not have sufficient privileges to perform the operation
        NotAuthorized,
        /// Account doesn't exist
        NotExists,
        // add additional errors below
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        #[weight = <T as Trait>::WeightInfo::register_zone()]
        pub fn zone_add(origin) -> dispatch::DispatchResult{
            Ok(())
        }
    }
}
// Module allows  use  common functionality by dispatchables
impl<T: Trait> Module<T> {
    // Implement module function.
    // Public functions can be called from other runtime modules.

    /// Check if an account has some role
    pub fn zone_is(zone: &T::ZoneNumber, zone_type: ZoneType) -> bool {
        CityMap::<T>::get(zone).zone_is(ZoneType::Green)
    }
}

