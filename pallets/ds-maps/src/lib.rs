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

use sp_std::{
    str::FromStr,
    marker::PhantomData,
};

use dsky_utils::{CastToType, IntDiv, Signed};
use frame_system::ensure_signed;
use pallet_ds_accounts as accounts;
use accounts::REGISTRAR_ROLE;

mod default_weight;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const GREEN_AREA: u8 = 0b00000001;

/// RootBox parameters
pub const MAX_ROW_INDEX: u16 = 35_999;
pub const MAX_COLUMN_INDEX: u16 = 18_000;

/// Zero-degree indexes for latitude and longitude in bitmap
pub const ZERO_DEGREE_ROW_INDEX: u16 = 18_000;
pub const ZERO_DEGREE_COLUMN_INDEX: u16 = 9_000;

/// Page parameters
pub const MAX_PAGE_INDEX: u32 = 404_999;
pub const MAX_PAGES_AMOUNT_TO_EXTRACT: u8 = 9;
pub const PAGE_LENGTH: usize = 32;
pub const PAGE_WIDTH: usize = 50;

/// Bitmap cell parameters in degree e-2
const BITMAP_CELL_LENGTH: u32 = 1;
const BITMAP_CELL_WIDTH: u32 = 1;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point2D<Coord> {
    lat: Coord,
    lon: Coord,
}

impl<
    Coord: PartialOrd + Sub<Output = Coord> + Signed + IntDiv
    > Point2D<Coord> {
    pub fn new(lat: Coord, lon: Coord) -> Self {
        Point2D{lat, lon}
    }

    // Here Point2D actually represent not a point, but distance
    pub fn get_distance_vector(self, second_point: Point2D<Coord>) -> Point2D<Coord> {
        let lat_length = (self.lat - second_point.lat).abs();
        let lon_length = (self.lon - second_point.lon).abs();
        Point2D::new(lat_length, lon_length)
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect2D<Coord> {
    north_east: Point2D<Coord>,
    south_west: Point2D<Coord>,
}

impl<
    Coord: PartialOrd + Sub<Output = Coord> + Signed + IntDiv
    > Rect2D<Coord> {
    pub fn new(north_east: Point2D<Coord>, south_west: Point2D<Coord>) -> Self {
        Rect2D{north_east, south_west}
    }

    // Here Point2D actually represent not a point, but rect's size
    pub fn get_dimensions(self) -> Point2D<Coord> {
        self.north_east.get_distance_vector(self.south_west)
    }
    
    /// True if this rect intersects other, excluding edges.
    pub fn intersects_rect(self, target: Rect2D<Coord>) -> bool {
        !(self.south_west.lon <= target.north_east.lon ||
          self.north_east.lon >= target.south_west.lon ||
          self.south_west.lat <= target.north_east.lat ||
          self.north_east.lat >= target.south_west.lat)
    }

    /// True, if given point lies inside the rect, excluding edges. 
    pub fn is_point_inside(&self, target: Point2D<Coord>) -> bool {
        !(self.south_west.lon <= target.lon ||
          self.north_east.lon >= target.lon ||
          self.south_west.lat <= target.lat ||
          self.north_east.lat >= target.lat)
    }
}

#[cfg(test)]
mod rect_tests {
    use super::*;
    use crate::tests::{construct_custom_rect, coord};
    // construct_custom_rect(a, b, c, d)
    //
    //       (c,d)
    //   +-----o
    //   |     |
    //   |     |
    //   o-----+
    // (a,b)

    #[test]
    fn rect_intersects_itself() {
        let rect = construct_custom_rect("1", "1", "3", "3");
        assert!(rect.intersects_rect(rect));
    }

    #[test]
    fn rect_b_fully_inside_a() {
        let rect_a = construct_custom_rect("1", "1", "4", "6");
        let rect_b = construct_custom_rect("2", "2", "3", "5");
        assert!(rect_a.intersects_rect(rect_b));
    }

    #[test]
    fn rect_b_on_edge_of_a() {
        let rect_a = construct_custom_rect("1", "1", "4", "6");
        let rect_b = construct_custom_rect("0", "0", "1", "5");
        assert!(!rect_a.intersects_rect(rect_b));
    }

    #[test]
    fn rect_b_outside_a() {
        let rect_a = construct_custom_rect("1", "1", "4", "6");
        let rect_b = construct_custom_rect("10", "10", "30", "30");
        assert!(!rect_a.intersects_rect(rect_b));
    }

    #[test]
    fn point_inside_rect() {
        let rect = construct_custom_rect("1", "1", "3", "5");
        let point = Point2D::new(coord("2"), coord("4"));
        assert!(rect.is_point_inside(point));
    }

    #[test]
    fn point_on_edge_rect() {
        let rect = construct_custom_rect("1", "1", "3", "5");
        let point = Point2D::new(coord("1"), coord("2"));
        assert!(!rect.is_point_inside(point));
    }

    #[test]
    fn point_outside_rect() {
        let rect = construct_custom_rect("1", "1", "3", "5");
        let point = Point2D::new(coord("0"), coord("0"));
        assert!(!rect.is_point_inside(point));
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
#[derive(Encode, Decode, Default, Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Encode, Decode, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Box3D<Coord> {
    pub north_east: Point3D<Coord>,
    pub south_west: Point3D<Coord>,
}

impl<
    Coord: PartialOrd + Sub<Output = Coord> + Signed + IntDiv
    > Box3D<Coord> {
    pub fn new(north_east: Point3D<Coord>, south_west: Point3D<Coord>) -> Self {
        Box3D{north_east, south_west}
    }

    /// Gets rect 2D projection from a box
    pub fn projection_on_plane(self) -> Rect2D<Coord> {
        let north_east =
        Point2D::new(self.north_east.lat,
                     self.north_east.lon);
        let south_west =
        Point2D::new(self.south_west.lat,
                    self.south_west.lon);
        Rect2D::new(north_east, south_west)
    }
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Default, Clone, Copy, Debug)]
pub struct RootBox<Coord> {
    pub id: RootId,
    pub bounding_box: Box3D<Coord>,
    pub delta: Coord,
}

impl<
    Coord: PartialOrd + Sub<Output = Coord> + Signed + IntDiv + Div<Output = Coord> + Copy 
    > RootBox<Coord> {
    pub fn new(id: RootId, bounding_box: Box3D<Coord>, delta: Coord) -> Self {
        RootBox{id, bounding_box, delta}
    }

    /// Returns maximum area index of given root. Max is 65536.
    pub fn get_max_area(self) -> AreaId {
        let root_dimensions = self.bounding_box.projection_on_plane().get_dimensions();
        let total_rows = root_dimensions.lat.integer_division_u16(self.delta);
        let total_columns = root_dimensions.lon.integer_division_u16(self.delta);

        total_rows * total_columns
    }

    /// Returns id of an area in root, in which supplied point is located
    fn detect_intersected_area(self, touch: Point2D<Coord>) -> AreaId {
        let root_projection = self.bounding_box.projection_on_plane();
        if !root_projection.is_point_inside(touch) {
            return 0;
        }
        let root_dimensions = root_projection.get_dimensions();
        let touch_vector = root_projection.north_east.get_distance_vector(touch);
        
        let row = touch_vector.lat.integer_division_u16(self.delta) + 1;
        let column = touch_vector.lon.integer_division_u16(self.delta) + 1;
        let total_rows = root_dimensions.lat.integer_division_u16(self.delta);

        (total_rows * (column - 1)) + row
    }

    #[cfg(test)]
    pub fn is_active(&self) -> bool {
        self.id != 0
    }
}

#[cfg(test)]
mod rootbox_tests {
    use super::*;
    use crate::tests::{construct_custom_box, ROOT_ID, coord};

    #[test]
    fn max_area_small_root() {
        let bbox = construct_custom_box("0", "0", "2", "3");
        let root = RootBox::new(ROOT_ID, bbox, coord("1"));
        assert_eq!(root.get_max_area(), 6); 
    }
    
    #[test]
    fn max_area_frac_delta() {
        let bbox = construct_custom_box("-0", "0", "2", "3");
        let root = RootBox::new(ROOT_ID, bbox, coord("0.5"));
        assert_eq!(root.get_max_area(), 24);
    } 

    #[test]
    fn max_area_big_root() {
        let bbox = construct_custom_box("-90", "-180", "0", "0");
        let root = RootBox::new(ROOT_ID, bbox, coord("1"));
        assert_eq!(root.get_max_area(), 16_200);
    } 

    #[test]
    fn area_detects_correct() {
        let bbox = construct_custom_box("0", "0", "2", "3");
        let root = RootBox::new(100, bbox, coord("1"));

        let point = Point2D::new(coord("0.5"),
                                 coord("0.5"));
        assert_eq!(root.detect_intersected_area(point), 1);

        let point = Point2D::new(coord("1.5"),
                                 coord("1.5"));
        assert_eq!(root.detect_intersected_area(point), 4);
        
        let edge_point = Point2D::new(coord("2"),
                                      coord("3"));
        assert_eq!(root.detect_intersected_area(edge_point), 0); 
        
        let out_point = Point2D::new(coord("50"),
                                     coord("50"));
        assert_eq!(root.detect_intersected_area(out_point), 0); 
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

const CELL_SIZE_DEGREE: u8 = 2;

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page<Coord> {
    pub bitmap: [[u32; PAGE_WIDTH]; PAGE_LENGTH],
    _phantom: PhantomData<Coord>,
}

impl<
    Coord: Default
    + FromStr
    + Copy
    + CastToType
    + core::ops::Div<Output = Coord>
> Page<Coord> {
    fn new() -> Self {
        Page{
            bitmap: [[0u32; PAGE_WIDTH]; PAGE_LENGTH],
            _phantom: PhantomData,
        }
    }

    pub fn get_amount_of_pages_to_extract(bounding_box: Box3D<Coord>) -> u32 {
        let mut amount_of_pages: u32 = 0;

        let (sw_row_index, sw_column_index) = Self::get_cell_indexes(bounding_box.south_west);
        let (ne_row_index, ne_column_index) = Self::get_cell_indexes(bounding_box.north_east);

        let sw_cell_page_index = Self::get_page_index(sw_row_index, sw_column_index);
        let ne_cell_page_index = Self::get_page_index(ne_row_index, ne_column_index);

        if sw_cell_page_index == ne_cell_page_index {
            amount_of_pages = 1;
            return amount_of_pages
        }

        let (sw_page_row_index, sw_page_column_index) = Self::extract_values_from_page_index(sw_cell_page_index);
        let (ne_page_row_index, ne_page_column_index) = Self::extract_values_from_page_index(ne_cell_page_index);

        amount_of_pages += (ne_page_row_index - sw_page_row_index) / MAX_ROW_INDEX as u32 +
                           (ne_page_column_index - sw_page_column_index) / MAX_COLUMN_INDEX as u32;

        amount_of_pages
    }

    pub fn get_cell_indexes(point: Point3D<Coord>) -> (u32, u32) {
        let lat: u32 = point.lat.to_u32_with_frac_part(BITMAP_CELL_LENGTH, CELL_SIZE_DEGREE);
        let lon: u32 = point.lon.to_u32_with_frac_part(BITMAP_CELL_WIDTH, CELL_SIZE_DEGREE);

        let row_index: u32 = lat / BITMAP_CELL_LENGTH;
        let column_index: u32 = lon / BITMAP_CELL_WIDTH;

        (row_index, column_index)
    }

    pub fn get_page_index(cell_row_index: u32, cell_column_index: u32) -> u32 {
        let mut row_index: u32 = 0;
        let mut column_index: u32 = 0;

        if (cell_row_index - 1) % PAGE_LENGTH as u32 != 0 {
            row_index = PAGE_LENGTH as u32 *
                (cell_row_index + PAGE_LENGTH as u32 - (cell_row_index) % PAGE_LENGTH as u32);
        } else {
            row_index = PAGE_LENGTH as u32 * cell_row_index;
        }
        if (cell_column_index - 1) % PAGE_WIDTH as u32 != 0 {
            column_index = PAGE_WIDTH as u32 *
                (cell_column_index + PAGE_WIDTH as u32 - (cell_column_index) % PAGE_WIDTH as u32);
        } else {
            column_index = PAGE_WIDTH as u32 * cell_column_index;
        }

        (row_index << 16) | column_index
    }

    fn extract_values_from_page_index(page_index: u32) -> (u32, u32) {
        let mask_u16: u32 = 0b1111_1111_1111_1111;
        let row_index: u32 = page_index >> 16;
        let column_index: u32 = page_index & mask_u16;
        (row_index, column_index)
    }
}

impl<Coord: Default + FromStr> Default for Page<Coord> {
    fn default() -> Self {
        Page{
            bitmap: [[0u32; PAGE_WIDTH]; PAGE_LENGTH],
            _phantom: PhantomData,
        }
    }
}

type AreaId = u16;
type PageId = u32;
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
    + IntDiv
    + Signed
    + CastToType
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

        EarthBitmap get(fn bitmap_cells):
            map hasher(blake2_128_concat) PageId => PageOf<T>;

        AreaData get(fn area_info):
            double_map hasher(blake2_128_concat) RootId, 
                       hasher(blake2_128_concat) AreaId => Area;    

        RedZones get(fn zone_data): 
            map hasher(blake2_128_concat) ZoneId => ZoneOf<T>;
    }
}

pub type PageOf<T> = Page<<T as Trait>::Coord>;
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
            let root_size = bounding_box.projection_on_plane().get_dimensions();
            ensure!(root_size.lat <= Self::coord_from_str("1"), Error::<T>::BadDimesions);
            ensure!(root_size.lon <= Self::coord_from_str("1"), Error::<T>::BadDimesions);

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
            let area_id = RootBoxes::<T>::get(root_id).detect_intersected_area(rect.north_east);
            let se_area_id = RootBoxes::<T>::get(root_id).detect_intersected_area(rect.south_west);
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
                        let rect_to_check = RedZones::<T>::get(current_zone).rect;
                        ensure!(!rect_to_check.intersects_rect(rect), Error::<T>::OverlappingZone);
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

        /// Removes zone by given id
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

// Module allows use common functionality by dispatchables
impl<T: Trait> Module<T> {
    // Implement module function.
    // Public functions can be called from other runtime modules.
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
        (root as ZoneId) << 32 |
        (area as ZoneId) << 16 | 
        children as ZoneId
    }

    /// Reverse function for pack_index()
    #[allow(dead_code)]
    fn unpack_index(index: ZoneId) -> (RootId, AreaId, u16) {
        let mask_u16: u64 = 0x0000_0000_0000_0000_0000_0000_ffff_ffff;
        let root: RootId = (index >> 32) as RootId;
        let area: AreaId = ((index >> 16) & mask_u16) as AreaId;
        let children: u16 = (index & mask_u16) as u16;
        
        (root, area, children)
    }
}

