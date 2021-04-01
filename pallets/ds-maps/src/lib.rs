#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::{
    codec::{Decode, Encode},
    storage::StorageDoubleMap,
    sp_runtime::sp_std::ops::{Add, Sub, Div, Mul, Shl, BitOr},
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,    
    weights::Weight,
    Parameter,
};
use sp_std::str::FromStr;
use substrate_fixed::types::{I9F23, I64F64};

use frame_system::ensure_signed;
use pallet_ds_accounts as accounts;
use accounts::REGISTRAR_ROLE;

mod default_weight;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const GREEN_AREA: u8 = 0b00000001;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Point2D<Coord> {
    lon: Coord,
    lat: Coord,
}

impl<
        Coord: From<I9F23> + Into<I9F23> + Sub<Output = Coord>
    > Point2D<Coord> 
{
    pub fn new(lon: Coord, lat: Coord) -> Self {
        Point2D{lon, lat}
    }

    pub fn get_distance_vector(self, second_point: Point2D<Coord>) -> Point2D<Coord> {
        let lat_length = (self.lat - second_point.lat).into().abs();
        let long_length = (self.lon - second_point.lon).into().abs(); 
        Point2D::new(lat_length.into(), long_length.into())
    }
}

//derives and if req by compiler
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug, Clone, PartialEq, Eq)]
pub struct Rect2D<Point2D> {
    north_west: Point2D,
    south_east: Point2D,
}

impl<Point2D> Rect2D<Point2D> {
    pub fn new(north_west: Point2D, south_east: Point2D) -> Self {
        Rect2D{north_west, south_east}
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
        Zone {zone_id, rect, height}
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
    pub north_west: Point3D,
    pub south_east: Point3D,
}

impl <Point3D> Box3D<Point3D> {
    pub fn new(north_west: Point3D, south_east: Point3D) -> Self {
        Box3D{north_west, south_east}
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug)]
pub struct RootBox<RootId, Box3D, Coord> {
    pub id: RootId,
    pub bounding_box: Box3D,
    pub delta: Coord,
}

impl<RootId, Box3D, Coord> RootBox <RootId, Box3D, Coord> {
    pub fn new(id: RootId, bounding_box: Box3D, delta: Coord) -> Self {
        RootBox{id, bounding_box, delta}
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug, PartialEq)]
pub struct Area {
    pub area_type: u8,
    pub child_amount: u16,
}

impl Area {
    pub fn new(area_type: u8, child_amount: u16) -> Self {
        Area{area_type, child_amount}
    } 
}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: accounts::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    // Describe pallet constants.
    // Lean more https://substrate.dev/docs/en/knowledgebase/runtime/metadata
    type WeightInfo: WeightInfo;

    // new types, consider descriptions
    type Coord: Default 
    + Parameter
    + Copy
    + PartialOrd
    + From<I9F23>
    + Into<I9F23>
    + Sub<Output = Self::Coord>
    + Div<Output = Self::Coord>
    + Add<Output = Self::Coord>
    + Mul<Output = Self::Coord>;
    
    type LocalCoord: Default 
    + Parameter
    + Copy
    + PartialOrd;

    type AreaId: Default 
    + Parameter
    + Copy
    + PartialOrd
    + From<u16>
    + Into<u16>;

    type RootId: Default 
    + Parameter
    + Copy
    + PartialOrd
    + From<u32>
    + Into<u32>
    + Add<Output = Self::RootId>;
    
    type ZoneId: Default 
    + Parameter
    + Copy
    + PartialOrd
    + Shl
    + From<u64>
    + Into<u64>
    + BitOr<Output = Self::ZoneId>;
}    

pub trait WeightInfo {
    fn root_add() -> Weight;
    fn zone_add() -> Weight;
    fn change_area_type() -> Weight;
}

decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvv
    trait Store for Module<T: Trait> as DSMapsModule {
        // MAX is 4_294_967_295. Change if required more.
        TotalRoots get(fn total_roots): T::RootId = 0.into();    

        RootBoxes get(fn root_box_data): 
            map hasher(blake2_128_concat) T::RootId => RootBoxOf<T>;

        AreaData get(fn area_info): 
            double_map hasher(blake2_128_concat) T::RootId, 
                       hasher(blake2_128_concat) T::AreaId => Area;    

        RedZones get(fn zone_data): 
            map hasher(blake2_128_concat) T::ZoneId => ZoneOf<T>;
    }
}

pub type RootBoxOf<T> = RootBox<<T as Trait>::RootId, Box3D<Point3D<<T as Trait>::Coord>>, <T as Trait>::Coord>;
pub type ZoneOf<T> = Zone<<T as Trait>::ZoneId, Rect2D<Point2D<<T as Trait>::Coord>>>;

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        RootId = <T as Trait>::RootId,
        AreaId = <T as Trait>::AreaId,
        ZoneId = <T as Trait>::ZoneId,
        {
        // Event documentation should end with an array that provides descriptive names for event parameters.
        /// New root box has been created [box number, who]
        RootCreated(RootId, AccountId),
        /// New zone added [root, area, zone number, who]
        ZoneCreated(RootId, AreaId, ZoneId, AccountId),
        /// area type changed [role, area, root, who]
        AreaTypeChanged(u8, AreaId, RootId, AccountId),
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
        /// Trying to add zone in non-existing root
        RootDoesNotExist,
        /// Sizes are off bounds
        BadDimesions
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
        /// Adds new root to storage
        #[weight = <T as Trait>::WeightInfo::root_add()]
        pub fn root_add(origin, 
                        bounding_box: Box3D<Point3D<T::Coord>>,
                        delta: T::Coord) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            // Here more complex calculation for root dimensions and delta needed
            let lat_dim = bounding_box.south_east.lat - bounding_box.north_west.lat;
            ensure!(lat_dim.into() <= I9F23::from_str("1").unwrap(), Error::<T>::BadDimesions);
            let lon_dim = bounding_box.south_east.lon - bounding_box.north_west.lon;
            ensure!(lon_dim.into() <= I9F23::from_str("1").unwrap(), Error::<T>::BadDimesions);
            ensure!(delta.into() <= I9F23::from_str("0.1").unwrap() && 
                    delta.into() >= I9F23::from_str("0.002").unwrap(), Error::<T>::InvalidData);

            let id = TotalRoots::<T>::get();
            let root = RootBoxOf::<T>::new(id, bounding_box, delta);
            RootBoxes::<T>::insert(id, root);
            TotalRoots::<T>::put(id + 1.into());
            Self::deposit_event(RawEvent::RootCreated(id, who));
            Ok(())
        }
        
        /// Form index and store input to redzones, creates area struct if it doesnt exist
        #[weight = <T as Trait>::WeightInfo::zone_add()]
        pub fn zone_add(origin, 
                        rect: Rect2D<Point2D<T::Coord>>,
                        height: u16,
                        root_id: T::RootId) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            ensure!(RootBoxes::<T>::contains_key(root_id), Error::<T>::RootDoesNotExist);
            ensure!(height > 1, Error::<T>::InvalidData);
            // TODO calc required area in root from rect, rn no overlap checks
            let area_id = Self::detect_touch(RootBoxes::<T>::get(root_id), rect.north_west);
                        
            let area = if AreaData::<T>::contains_key(root_id, area_id) {
                AreaData::<T>::get(root_id, area_id)
            } else {
                AreaData::<T>::insert(root_id, area_id, Area::new(GREEN_AREA, 0));
                Area::new(GREEN_AREA, 0)
            };

            ensure!(area.area_type == GREEN_AREA, Error::<T>::ForbiddenArea); 
            let id = Self::form_index(root_id, area_id, area.child_amount); 
            let zone = ZoneOf::<T>::new(id, rect, height);
            RedZones::<T>::insert(id, zone);
            AreaData::<T>::mutate(root_id, area_id, |ar| {
                ar.child_amount += 1;
            });
            Self::deposit_event(RawEvent::ZoneCreated(root_id, area_id, id, who));
            Ok(())
        }

        /// Changes area type with u8 bit flag
        #[weight = <T as Trait>::WeightInfo::change_area_type()]
        pub fn change_area_type(origin, 
                                root_id: T::RootId, 
                                area_id: T::AreaId, 
                                area_type: u8) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            ensure!(AreaData::<T>::contains_key(root_id, area_id), Error::<T>::NotExists);
            AreaData::<T>::mutate(root_id, area_id, |ar| {
                ar.area_type = area_type;
            });
            Self::deposit_event(RawEvent::AreaTypeChanged(area_type, area_id, root_id, who));
            Ok(())
        }
    }
}

// Module allows  use  common functionality by dispatchables
impl<T: Trait> Module<T> {
    // Implement module function.
    // Public functions can be called from other runtime modules.
    fn detect_touch(root_box: RootBoxOf<T>, touch: Point2D<T::Coord>) -> T::AreaId {
        let root_base_point: Point2D<T::Coord> = 
        Point2D::new(root_box.bounding_box.north_west.lat,
                     root_box.bounding_box.north_west.lon); 
        let root_secondary_point: Point2D<T::Coord> = 
        Point2D::new(root_box.bounding_box.south_east.lat,
                    root_box.bounding_box.south_east.lon); 
        let root_dimensions = root_base_point.get_distance_vector(root_secondary_point);
        let distance_vector = root_base_point.get_distance_vector(touch);
        // casts required to evade possible overflows in division
        let delta: I64F64 = I64F64::from(root_box.delta.into());
        let touch_lon: I64F64 = I64F64::from(distance_vector.lon.into());
        let touch_lat: I64F64 = I64F64::from(distance_vector.lat.into());
        let root_lat_dimension: I64F64 = I64F64::from(root_dimensions.lat.into());

        let row: u16 = (touch_lat / delta).to_num::<u16>() + 1;
        let column: u16 = (touch_lon / delta).to_num::<u16>() + 1;
        let total_rows: u16 = (root_lat_dimension / delta).to_num::<u16>();

        ((total_rows * (column - 1)) + row).into()
    }

    /// form index for storing zones, wrapped in u64
    // v.............root id here............v v.....area id.....v v..child objects..v
    // 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000
    fn form_index(root: T::RootId, area: T::AreaId, childs: u16) -> T::ZoneId {
        let root: u32 = root.into();
        let area: u16 = area.into();

        ((root as u64) << 32 |
         (area as u64) << 16 | 
         childs as u64).into()      
    }

    #[allow(dead_code)]
    fn unwrap_index(index: T::ZoneId) -> (T::RootId, T::AreaId, u16) {
        let mask_u16: u64 = 0x0000_0000_0000_0000_0000_0000_ffff_ffff;
        let index: u64 = index.into();
        // is refactoring possible?
        let root: T::RootId = ((index >> 32) as u32).into();
        let area: T::AreaId = (((index >> 16) & mask_u16) as u16).into();
        let childs: u16 = (index & mask_u16) as u16;
        
        (root, area, childs)
    }
}

