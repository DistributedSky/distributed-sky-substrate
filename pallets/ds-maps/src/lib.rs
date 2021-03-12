#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    weights::{Weight},
    Parameter,
};
use frame_system::ensure_signed;
use pallet_ds_accounts as accounts;
use accounts::REGISTRAR_ROLE;

mod default_weight;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, Debug, PartialEq, Eq)]
pub struct Point2D<Coord> {
    x: Coord,
    y: Coord,
}

impl<Coord> Point2D<Coord> {
    pub fn new(x: Coord, y: Coord) -> Self {
        Point2D{x, y}
    }
}

/*
//derives and if req by compiler
pub struct Rect2D<Point> {
    point_1: Point,
    point_2: Point,
}

impl<Point> Rect2D<Point> {
    pub fn new(point_1: Point, point_2: Point) -> Self {
        Rect2D{point_1, point_2}
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default)]
pub struct Zone<Point> {
    pub rect: Rect2D<Point>,
    pub height: u16,
}

impl<Point> Zone<Point> {
    pub fn zone_is(&self, zone: ZoneType) -> bool {
        self.zone_type == zone
    }

    pub fn new( zone_id: u32, 
                zone_type: ZoneType, 
                bounding_box: Box3D<Point> ) -> Self {
            Zone {
                bounding_box,
                zone_type,
                zone_id,
            }
    }
} 
*/
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default, Debug, PartialEq, Eq)]
pub struct Point3D<Coord> {
    lat: Coord,
    lon: Coord,
    alt: Coord,
}

impl<Coord> Point3D<Coord> {
    pub fn new(lat: Coord, lon: Coord, alt: Coord) -> Self {
        Point3D{lat, lon, alt}
    }
}

pub struct Box3D<Point3D, Coord> {
    pub south_west: Point3D,
    pub north_east: Point3D,
}

impl <Point3D, Coord> Box3D<Point3D, Coord>{
    pub fn new(south_west: Point3D, north_east: Point3D) -> Self {
        Box3D{south_west, north_east}
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug)]
pub struct RootBox<BoxType, LocalCoord> {
    pub id: u32,
    bounding_box: BoxType,
    pub delta: LocalCoord,
}

impl<BoxType, LocalCoord> RootBox<BoxType, LocalCoord> {
    pub fn new(id: u32, bounding_box: BoxType, delta: LocalCoord) -> Self {
        RootBox{id, bounding_box, delta}
    }

    pub fn detect_touch(&self, touch: Point2D) -> u16 {
        2
    }
}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: accounts::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    // Describe pallet constants.
    // Lean more https://substrate.dev/docs/en/knowledgebase/runtime/metadata
    type WeightInfo: WeightInfo;
    // new types, consider description
    /// representing a box in global space
    type BoxType: Default + Parameter;
    /// representing a rectangle on a plane
    type RectType: Default + Parameter;

    /// use u32 for representing global coords, u16 for local
    type Coord: Default + Parameter;
    type LocalCoord: Default + Parameter;
}    

pub trait WeightInfo {
    fn zone_add() -> Weight;
}

decl_storage!{
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvv
    trait Store for Module<T: Trait> as DSMapsModule {
        // MAX is 4_294_967_295. Change if required more.
        TotalRoots get(fn total_roots): u32;    

        RootBoxes get(fn root_box_data): 
            map hasher(blake2_128_concat) u32 => RootBoxOf<T>;
        // RedZones get(fn zone_data): 
        //     map hasher(blake2_128_concat) u32 => ZoneOf<T>;
    }
}
pub type RootBoxOf<T> = RootBox<<T as Trait>::BoxType, <T as Trait>::LocalCoord>;
//pub type ZoneOf<T> = Zone<<T as Trait>::Point>;

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        // Event documentation should end with an array that provides descriptive names for event parameters.
        /// New root box has been created [box number, who]
        ZoneCreated(u32, AccountId),
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

        #[weight = <T as Trait>::WeightInfo::zone_add()]
        pub fn root_add(origin, 
                        bounding_box: BoxType,
                        delta: u32 ) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            // TODO implement inverted index, so we will not store same zones twice
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            
            let id = <TotalRoots>::get();
            let zone = RootBoxOf::<T>::new(id, bounding_box, delta);
            RootBoxes::<T>::insert(id, zone);
            Self::deposit_event(RawEvent::ZoneCreated(id, who));
            <TotalRoots>::put(id + 1);
            Ok(())
        }
    }
}

// Module allows  use  common functionality by dispatchables
impl<T: Trait> Module<T> {
    // Implement module function.
    // Public functions can be called from other runtime modules.
    // Check if zone have required type
    // pub fn zone_is(zone: u32, zone_type: ZoneType) -> bool {
    //     CityMap::<T>::get(zone).zone_is(zone_type)
    // }
}

