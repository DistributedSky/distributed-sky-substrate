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


//derives and if req by compiler
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug, Clone, PartialEq, Eq)]
pub struct Rect2D<Point2D> {
    point_1: Point2D,
    point_2: Point2D,
}

impl<Point2D> Rect2D<Point2D> {
    pub fn new(point_1: Point2D, point_2: Point2D) -> Self {
        Rect2D{point_1, point_2}
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default)]
pub struct Zone<ZoneId, Rect2D> {
    pub zone_id: ZoneId,
    pub rect: Rect2D,
    pub height: u16,
}

impl<ZoneId, Rect2D> Zone<ZoneId, Rect2D> {
    pub fn new(zone_id: ZoneId, rect: Rect2D, height: u16) -> Self {
        Zone { zone_id, rect, height}
    }
} 

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

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug, Clone, PartialEq, Eq)]
pub struct Box3D<Point3D> {
    pub south_west: Point3D,
    pub north_east: Point3D,
}

impl <Point3D> Box3D<Point3D> {
    pub fn new(south_west: Point3D, north_east: Point3D) -> Self {
        Box3D{south_west, north_east}
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug)]
pub struct RootBox<RootId, Box3D, LocalCoord> {
    pub id: RootId,
    pub bounding_box: Box3D,
    pub delta: LocalCoord,
}

impl<RootId, Box3D, LocalCoord> RootBox <RootId, Box3D, LocalCoord> {
    pub fn new(id: RootId, bounding_box: Box3D, delta: LocalCoord) -> Self {
        RootBox{id, bounding_box, delta}
    }

    // pub fn detect_touch(&self, touch: Point2D<Coord>) -> AreaId {
    //     2
    // }


    // pub fn generate_zone_id(&self,){

    // }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug)]
pub struct Area {
    pub area_type: u8,
    pub child_amount: u16,
}

impl Area {
    pub fn new(area_type: u8, child_amount: u16) -> Self {
        Area{area_type, child_amount}
    } 
}

// types to make pallet more readable
// TODO later move it to trait w all bounds 
type RootId = u32;
type ZoneId = u64;
type AreaId = u16;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: accounts::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    // Describe pallet constants.
    // Lean more https://substrate.dev/docs/en/knowledgebase/runtime/metadata
    type WeightInfo: WeightInfo;
    // new types, consider description

    /// use u32 for representing global coords, u16 for local
    type Coord: Default + Parameter;
    type LocalCoord: Default + Parameter;
}    

pub trait WeightInfo {
    fn root_add() -> Weight;
    fn zone_add() -> Weight;
}

decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvv
    trait Store for Module<T: Trait> as DSMapsModule {
        // MAX is 4_294_967_295. Change if required more.
        TotalRoots get(fn total_roots): RootId = 0;    

        RootBoxes get(fn root_box_data): 
            map hasher(blake2_128_concat) RootId => RootBoxOf<T>;

        AreaData get(fn zone_index): 
            double_map hasher(blake2_128_concat) RootId, 
                       hasher(blake2_128_concat) AreaId => Area;    

        RedZones get(fn zone_data): 
            map hasher(blake2_128_concat) ZoneId => ZoneOf<T>;
    }
}

pub type RootBoxOf<T> = RootBox<RootId, Box3D<Point3D<<T as Trait>::Coord>>, <T as Trait>::LocalCoord>;
pub type ZoneOf<T> = Zone<ZoneId, Rect2D<Point2D<<T as Trait>::Coord>>>;

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        // Event documentation should end with an array that provides descriptive names for event parameters.
        /// New root box has been created [box number, who]
        RootCreated(u32, AccountId),
        // TODO add double mapping and declare here root and area
        ZoneCreated(AccountId, ZoneId),
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
        /// Area is unavailable for operation
        ForbiddenArea,
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

        #[weight = <T as Trait>::WeightInfo::root_add()]
        pub fn root_add(origin, 
                        bounding_box: Box3D<Point3D<T::Coord>>,
                        delta: T::LocalCoord) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            // TODO implement inverted index, so we will not store same roots twice
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            
            let id = TotalRoots::get();
            let root = RootBoxOf::<T>::new(id, bounding_box, delta);
            RootBoxes::<T>::insert(id, root);
            Self::deposit_event(RawEvent::RootCreated(id, who));
            TotalRoots::put(id + 1);
            Ok(())
        }

        // #[weight = <T as Trait>::WeightInfo::zone_add()]
        // pub fn touch(origin, id: u32, Point2D) -> dispatch::DispatchResult {
        //     let who = ensure_signed(origin)?;
        //     let root = <TotalRoots>::get();
            

        // }

        #[weight = <T as Trait>::WeightInfo::zone_add()]
        pub fn zone_add(origin, 
                        rect: Rect2D<Point2D<T::Coord>>,
                        height: u16,
                        // TODO change to calc required area & root from rect(GL Daniil)
                        root_id: RootId,
                        area_id: AreaId) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            
            // form index and store input to redzones
            // increase area counter by 1
            let area = <AreaData>::get(root_id, area_id);
            ensure!(area.area_type == 0b00000001, Error::<T>::ForbiddenArea);
            let id = Self::form_index(root_id, area_id, area.child_amount); 
            let zone = ZoneOf::<T>::new(id, rect, height);
            RedZones::<T>::insert(id, zone);
            AreaData::mutate(root_id, area_id, |ar| {
                ar.child_amount += 1;
            });

            Self::deposit_event(RawEvent::ZoneCreated(who, id));
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

        // v.............root id here............v v.....area id.....v v..child objects..v
        // 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000
        fn form_index(root: u32, area: u16, childs: u16) -> u64 {
            // consider refactoring this function?
            let area_expanded: u64 = (area as u64) << 16;
            let root_expanded: u64 = (root as u64) << 32;
            
            (childs as u64) | area_expanded | root_expanded
        }
}

