#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::{
    codec::{Decode, Encode},
    storage::StorageDoubleMap,
    dispatch::fmt::Debug,
    sp_runtime::sp_std::ops::{Sub, Div},
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,    
    weights::Weight,
    Parameter,
    traits::Get,
};
use sp_std::str::FromStr;
use dsky_utils::MathUtils;

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

impl<Coord: PartialOrd 
          + Sub<Output = Coord> 
          + MathUtils> Point2D<Coord> {
    pub fn new(lon: Coord, lat: Coord) -> Self {
        Point2D{lon, lat}
    }

    /// There is no shared trait implementing method abs(), so it's written like that
    pub fn get_distance_vector(self, second_point: Point2D<Coord>) -> Point2D<Coord> {
        let lat_length = (self.lat - second_point.lat).abs();
        let long_length = (self.lon - second_point.lon).abs();
        Point2D::new(lat_length, long_length)
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug, Clone, PartialEq, Eq)]
pub struct Rect2D<Coord> {
    north_west: Point2D<Coord>,
    south_east: Point2D<Coord>,
}

impl<Coord> Rect2D<Coord> {
    pub fn new(north_west: Point2D<Coord>, south_east: Point2D<Coord>) -> Self {
        Rect2D{north_west, south_east}
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default)]
pub struct Zone<Coord, LightCoord> {
    pub zone_id: ZoneId,
    pub rect: Rect2D<Coord>,
    pub height: LightCoord,
}

impl<Coord, LightCoord> Zone<Coord, LightCoord> {
    pub fn new(zone_id: ZoneId, rect: Rect2D<Coord>, height: LightCoord) -> Self {
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
pub struct Box3D<Coord> {
    pub north_west: Point3D<Coord>,
    pub south_east: Point3D<Coord>,
}

impl <Coord> Box3D<Coord> {
    pub fn new(north_west: Point3D<Coord>, south_east: Point3D<Coord>) -> Self {
        Box3D{north_west, south_east}
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug)]
pub struct RootBox<Coord> {
    pub id: RootId,
    pub bounding_box: Box3D<Coord>,
    pub delta: Coord,
}

impl<Coord: PartialOrd
          + Sub<Output = Coord> 
          + Div<Output = Coord> 
          + MathUtils
          + Copy> RootBox <Coord> {
    pub fn new(id: RootId, bounding_box: Box3D<Coord>, delta: Coord) -> Self {
        RootBox{id, bounding_box, delta}
    }
    /// Returns maximum area index of given root. Guess, mostly - 655356.
    pub fn get_max_area(self) -> AreaId {
        let touch = Point2D::new(self.bounding_box.south_east.lat,
                                 self.bounding_box.south_east.lon); 
        Self::detect_intersected_area(self, touch)
    }

    /// Returns id of an area in root, in which supplied point is located
    fn detect_intersected_area(self, touch: Point2D<Coord>) -> AreaId {
        let root_base_point = 
        Point2D::new(self.bounding_box.north_west.lat,
                     self.bounding_box.north_west.lon); 
        let root_secondary_point = 
        Point2D::new(self.bounding_box.south_east.lat,
                    self.bounding_box.south_east.lon); 
        let root_dimensions = root_base_point.get_distance_vector(root_secondary_point);
        let distance_vector = root_base_point.get_distance_vector(touch);
        
        let delta = self.delta;
        let touch_lon = distance_vector.lon;
        let touch_lat = distance_vector.lat;
        let root_lat_dimension = root_dimensions.lat;

        let row: u16 = touch_lat.integer_divide(delta) + 1;
        let column: u16 = touch_lon.integer_divide(delta) + 1;
        let total_rows: u16 = root_lat_dimension.integer_divide(delta);

        (total_rows * (column - 1)) + row
    }

    // This function used only in tests, consider usability
    pub fn is_active(&self) -> bool {
        self.id != 0
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug, PartialEq)]
pub struct Area {
    pub area_type: u8,
}

impl Area {
    pub fn new(area_type: u8) -> Self {
        Area{area_type}
    } 
}

type AreaId = u16;
type RootId = u32;
type ZoneId = u64;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: accounts::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    // Describe pallet constants.
    // Lean more https://substrate.dev/docs/en/knowledgebase/runtime/metadata
    type WeightInfo: WeightInfo;

    /// Represents GPS coordinate, usually 32 bit variables
    type Coord: Default 
    + Parameter
    + Copy
    + PartialOrd
    + PartialEq
    + FromStr
    + Default
    + MathUtils
    + Sub<Output = Self::Coord>
    + Div<Output = Self::Coord>;

    /// Used in places where u32 is too much (as altitude)
    type LightCoord: Default 
    + Parameter
    + Copy
    + PartialOrd
    + FromStr;

    /// This allows us to have a top border for zones
    type MaxBuildingsInArea: Get<u16>;
    
    /// Max available height of any building
    type MaxHeight: Get<Self::LightCoord>;
}    

pub trait WeightInfo {
    fn root_add() -> Weight;
    fn zone_add() -> Weight;
    fn root_remove() -> Weight;
    fn zone_remove() -> Weight;
    fn change_area_type() -> Weight;
}

decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvv
    trait Store for Module<T: Trait> as DSMapsModule {
        // MAX is 4_294_967_295. Change if required more.
        TotalRoots get(fn total_roots): RootId = 1;    

        RootBoxes get(fn root_box_data): 
            map hasher(blake2_128_concat) RootId => RootBoxOf<T>;

        AreaData get(fn area_info): 
            double_map hasher(blake2_128_concat) RootId, 
                       hasher(blake2_128_concat) AreaId => Area;    

        RedZones get(fn zone_data): 
            map hasher(blake2_128_concat) ZoneId => ZoneOf<T>;
    }
}

pub type RootBoxOf<T> = RootBox<<T as Trait>::Coord>;
pub type ZoneOf<T> = Zone<<T as Trait>::Coord, <T as Trait>::LightCoord>;

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        // Event documentation should end with an array that provides descriptive names for event parameters.
        /// New root box has been created [box number, who]
        RootCreated(RootId, AccountId),
        /// New zone added [root, area, zone number, who]
        ZoneCreated(RootId, AreaId, ZoneId, AccountId),
        /// Area type changed [role, area, root, who]
        AreaTypeChanged(u8, AreaId, RootId, AccountId),
        /// Root was removed from storage
        RootRemoved(RootId, AccountId),
        /// Zone was removed from storage
        ZoneRemoved(ZoneId, AccountId),
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
        /// Root you are trying to access is not in storage
        RootDoesNotExist,
        /// Sizes are off bounds
        BadDimesions,
        /// Area can't contain no more buildings
        AreaFull,
        /// Zone points lies in different areas
        ZoneDoesntFit,
        /// Zone you are trying to access is not in storage
        ZoneDoesntExist,
        /// Added zone overlaps with another in current area
        OverlappingZone,
        // Add additional errors below
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
                        bounding_box: Box3D<T::Coord>,
                        delta: T::Coord) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            // TODO replace these ensures w inverted index (using global grid)
            let lat_dim = bounding_box.south_east.lat - bounding_box.north_west.lat;
            ensure!(lat_dim <= Self::coord_from_str("1"), Error::<T>::BadDimesions);
            let lon_dim = bounding_box.south_east.lon - bounding_box.north_west.lon;
            ensure!(lon_dim <= Self::coord_from_str("1"), Error::<T>::BadDimesions);
            ensure!(delta <= Self::coord_from_str("0.1") && 
                    delta >= Self::coord_from_str("0.002"), Error::<T>::InvalidData);

            let id = TotalRoots::get();
            let root = RootBoxOf::<T>::new(id, bounding_box, delta);
            RootBoxes::<T>::insert(id, root);
            TotalRoots::put(id + 1);
            Self::deposit_event(RawEvent::RootCreated(id, who));
            Ok(())
        }
        
        /// Form index and store input to redzones, creates area struct if it doesnt exist
        #[weight = <T as Trait>::WeightInfo::zone_add()]
        pub fn zone_add(origin, 
                        rect: Rect2D<T::Coord>,
                        height: T::LightCoord,
                        root_id: RootId) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            ensure!(RootBoxes::<T>::contains_key(root_id), Error::<T>::RootDoesNotExist);
            ensure!(height < T::MaxHeight::get(), Error::<T>::InvalidData);
            // Check if zone lies in one single area 
            let area_id = RootBoxes::<T>::get(root_id).detect_intersected_area(rect.north_west);
            let se_area_id = RootBoxes::<T>::get(root_id).detect_intersected_area(rect.south_east);
            ensure!(area_id == se_area_id, Error::<T>::ZoneDoesntFit);
            // Getting area from storage, or creating it
            let (area, area_existed) = if AreaData::contains_key(root_id, area_id) {
                (AreaData::get(root_id, area_id), true)
            } else {
                AreaData::insert(root_id, area_id, Area::new(GREEN_AREA));
                (Area::new(GREEN_AREA), false)
            };

            let max_zones = T::MaxBuildingsInArea::get();
            let first_empty_id = Self::pack_index(root_id, area_id, 0);
            let mut zone_id = first_empty_id;
            
            // If area already exists, we check if it's full, and check all zones inside for intersection
            if area_existed {
                ensure!(area.area_type == GREEN_AREA, Error::<T>::ForbiddenArea); 
                let mut current_zone = first_empty_id;
                let mut empty_id_found = false;
                // Maybe, this cycle should be splitted in two. One finds first unused Id,
                // and only if it was found, we should look for intersections. Not sure.
                while current_zone < first_empty_id + max_zones as ZoneId {
                    if RedZones::<T>::contains_key(current_zone) || empty_id_found {
                        // Check if our zone overlaps with another zone in current area
                        ensure!(Self::zone_intersects(&RedZones::<T>::get(current_zone).rect, &rect), Error::<T>::OverlappingZone);
                        current_zone += 1;
                    } else { 
                        zone_id = current_zone;
                        empty_id_found = true;
                    }
                } 
                ensure!(empty_id_found, Error::<T>::AreaFull);
            } else {
                // This is first zone in area, we don't need to check any intersections and stuff.
                zone_id = first_empty_id; 
            }
            
            let zone = ZoneOf::<T>::new(zone_id, rect, height);
            RedZones::<T>::insert(zone_id, zone);
            Self::deposit_event(RawEvent::ZoneCreated(root_id, area_id, zone_id, who));
            Ok(())
        }

        /// Removes root by given id, and zones inside. This means, function might be heavy. 
        #[weight = <T as Trait>::WeightInfo::root_remove()]
        pub fn root_remove(origin, root_id: RootId) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            ensure!(RootBoxes::<T>::contains_key(root_id), Error::<T>::RootDoesNotExist);

            let max_zones = T::MaxBuildingsInArea::get();
            let max_areas = RootBoxes::<T>::get(root_id).get_max_area();
            // Recursively remove all zones inside selected root
            let mut zone_id = Self::pack_index(root_id, 0, 0);
            let mut area_id = 0;
            while area_id <= max_areas {
                let max_zones_in_area = zone_id + max_zones as ZoneId;
                while zone_id < max_zones_in_area {
                    if RedZones::<T>::contains_key(zone_id) {
                        RedZones::<T>::remove(zone_id); 
                    }
                    zone_id += 1;
                }
                area_id += 1;
                zone_id = Self::pack_index(root_id, area_id, 0);
            }

            RootBoxes::<T>::remove(root_id);
            Self::deposit_event(RawEvent::RootRemoved(root_id, who));
            Ok(())
        }
        /// Removes area by given id
        #[weight = <T as Trait>::WeightInfo::zone_remove()]
        pub fn zone_remove(origin, zone_id: ZoneId) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            ensure!(RedZones::<T>::contains_key(zone_id), Error::<T>::ZoneDoesntExist);
            
            RedZones::<T>::remove(zone_id);
            Self::deposit_event(RawEvent::ZoneRemoved(zone_id, who));
            Ok(())
        }
        
        /// Changes area type with u8 bit flag
        #[weight = <T as Trait>::WeightInfo::change_area_type()]
        pub fn change_area_type(origin, 
                                root_id: RootId, 
                                area_id: AreaId, 
                                area_type: u8) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            ensure!(AreaData::contains_key(root_id, area_id), Error::<T>::NotExists);
            
            AreaData::mutate(root_id, area_id, |ar| {
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
    // Just trust me on this one, it works
    fn zone_intersects(a: &Rect2D<T::Coord>, b: &Rect2D<T::Coord>) -> bool {
        a.south_east.lon < b.north_west.lon || a.north_west.lon > b.south_east.lon ||
        a.south_east.lat < b.north_west.lat || a.north_west.lat > b.south_east.lat
    }
    
    /// Creates type from str, no error handling
    fn coord_from_str<Coord> (s: &str) -> Coord
            where Coord: FromStr + Default {
        match Coord::from_str(s) {
            Ok(v) => v,
            Err(_) => Default::default(),
        }
    } 

    /// Form index for storing zones, wrapped in u64............limited by const in runtime
    /// v.............root id here............v v.....area id.....v v..child objects..v
    /// 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000
    fn pack_index(root: RootId, area: AreaId, children: u16) -> ZoneId {
        (root as u64) << 32 |
        (area as u64) << 16 | 
        children as u64
    }

    /// Reverse function for pack_index()
    #[allow(dead_code)]
    fn unpack_index(index: ZoneId) -> (RootId, AreaId, u16) {
        let mask_u16: u64 = 0x0000_0000_0000_0000_0000_0000_ffff_ffff;
        let root: RootId = (index >> 32) as u32;
        let area: AreaId = ((index >> 16) & mask_u16) as u16;
        let children: u16 = (index & mask_u16) as u16;
        
        (root, area, children)
    }
}

