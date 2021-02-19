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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
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
pub struct Point<CoordinateSize> {
    pub x: CoordinateSize,
    pub y: CoordinateSize,
    pub z: CoordinateSize,
}
impl<CoordinateSize> Point<CoordinateSize>{
    pub fn new_point(x: CoordinateSize, y: CoordinateSize, z: CoordinateSize) -> Self {
        Point{ x, y, z, }
    }
}
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default)]
pub struct Zone<CoordinateSize> {
    pub coordinates: [Point<CoordinateSize>; 2],
    pub obstacle_type: ZoneType,
    pub zone_id: u32,
}

impl<
    CoordinateSize,
    > Zone<CoordinateSize>
{
    pub fn zone_is(&self, zone: ZoneType) -> bool{
        !(self.obstacle_type == zone)
    }
    pub fn new(id: u32, zone_type: ZoneType, 
        points: (Point<CoordinateSize>, Point<CoordinateSize>)) -> Self{
            Zone{
                coordinates: [points.0, points.1],
                obstacle_type: zone_type,
                zone_id: id,
            }
    }
}

pub type ZoneOf<T> = Zone<<T as Trait>::CoordinateSize>;


/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait + pallet_timestamp::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    // Describe pallet constants.
    // Lean more https://substrate.dev/docs/en/knowledgebase/runtime/metadata
    type WeightInfo: WeightInfo;
    // new types, fixes required
    type Coordinate: Default + Parameter;
    ///guess use u32 for representing global coords, u16 for local
    type CoordinateSize: Default + Parameter;
    //type ZoneNumber: Default + Zero + MaybeSerializeDeserialize + Parameter + Member; 
}    
pub trait WeightInfo {
    fn register_zone() -> Weight;
}

decl_storage!{
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as DSMapsModule {
        //MAX is 4_294_967_295. Change if required more.
        TotalBoxes get(fn total_boxes): u32;    

        CityMap get(fn map_data): 
            map hasher(blake2_128_concat) u32 => ZoneOf<T>;
    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        ZoneNumber = u32,
        ZoneType = ZoneType,
    {
        // Event documentation should end with an array that provides descriptive names for event parameters.
        /// New account has been created [zone number, its type]
        ZoneCreated(ZoneNumber, ZoneType),
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
        pub fn zone_add(origin, 
                    zone_type: ZoneType, 
                    points: (Point<T::CoordinateSize>, Point<T::CoordinateSize>))
                     -> dispatch::DispatchResult {
            let who = ensure_signed(origin);
            //TODO implement call to account pallet and all checks.
            let id = <TotalBoxes>::get();
            //<Owner<T>>::insert(id, sender.clone());
            let zone = ZoneOf::<T>::new(id, zone_type, points);
            CityMap::<T>::insert(id, zone);
            Self::deposit_event(RawEvent::ZoneCreated(id, zone_type));
            
            <TotalBoxes>::put(id + 1);
            Ok(())
        }
    }
}
// Module allows  use  common functionality by dispatchables
impl<T: Trait> Module<T> {
    // Implement module function.
    // Public functions can be called from other runtime modules.

    /// Check if an account has some role
    pub fn zone_is(zone: u32, zone_type: ZoneType) -> bool {
        CityMap::<T>::get(zone).zone_is(zone_type)
    }
}

