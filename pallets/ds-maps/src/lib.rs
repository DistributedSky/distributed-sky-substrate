#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::{
    codec::{Decode, Encode},
    storage::StorageDoubleMap,
    dispatch::fmt::Debug,
    sp_runtime::sp_std::{ops::{Sub, Div}, vec::Vec},
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,    
    weights::Weight,
    Parameter,
    traits::Get,
};

use sp_std::{
    str::FromStr,
    marker::PhantomData,
    vec,
};

use dsky_utils::{CastToType, FromRaw, IntDiv, Signed};
use frame_system::ensure_signed;
use pallet_ds_accounts as accounts;
use accounts::REGISTRAR_ROLE;

mod default_weight;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const GREEN_AREA: u8 = 0b00000001;

/// Page parameters
pub const MAX_PAGES_AMOUNT_TO_EXTRACT: u32 = 4;
pub const PAGE_LENGTH: u32 = 32;
pub const PAGE_WIDTH: u32 = 50;

/// Bitmap cell parameters in degree e-2
const BITMAP_CELL_LENGTH: u32 = 1;
const BITMAP_CELL_WIDTH: u32 = 1;
const CELL_SIZE_DEGREE: u8 = 2;

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
    south_west: Point2D<Coord>,
    north_east: Point2D<Coord>,
}

impl<
    Coord: PartialOrd + Sub<Output = Coord> + Signed + IntDiv
    > Rect2D<Coord> {
    pub fn new(south_west: Point2D<Coord>, north_east: Point2D<Coord>) -> Self {
        Rect2D{south_west, north_east}
    }

    // Here Point2D actually represent not a point, but rect's size
    pub fn get_dimensions(self) -> Point2D<Coord> {
        self.south_west.get_distance_vector(self.north_east)
    }
    
    /// True if this rect intersects other, excluding edges.
    pub fn intersects_rect(self, target: Rect2D<Coord>) -> bool {
        !(self.north_east.lon <= target.south_west.lon || 
          self.south_west.lon >= target.north_east.lon ||
          self.north_east.lat <= target.south_west.lat || 
          self.south_west.lat >= target.north_east.lat)
    }

    /// True, if given point lies inside the rect, excluding edges. 
    pub fn is_point_inside(&self, target: Point2D<Coord>) -> bool {
        !(self.north_east.lon <= target.lon || 
          self.south_west.lon >= target.lon ||
          self.north_east.lat <= target.lat || 
          self.south_west.lat >= target.lat)
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
pub struct Zone<Coord> {
    pub zone_id: ZoneId,
    pub rect: Rect2D<Coord>,
    pub height: LightCoord,
}

impl<Coord> Zone<Coord> {
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
    pub south_west: Point3D<Coord>,
    pub north_east: Point3D<Coord>,
}

impl<
    Coord: PartialOrd + Sub<Output = Coord> + Signed + IntDiv
    > Box3D<Coord> {
    pub fn new(south_west: Point3D<Coord>, north_east: Point3D<Coord>) -> Self {
        Box3D{south_west, north_east}
    }

    /// Gets rect 2D projection from a box
    pub fn projection_on_plane(self) -> Rect2D<Coord> {
        let south_west = 
        Point2D::new(self.south_west.lat,
                     self.south_west.lon); 
        let north_east = 
        Point2D::new(self.north_east.lat,
                    self.north_east.lon); 
        Rect2D::new(south_west, north_east)
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

    /// Gets page index from boundary cells indexes (southwest and northeast)
    pub fn get_index(sw_cell_row_index: u32, sw_cell_column_index: u32,
                     ne_cell_row_index: u32, ne_cell_column_index: u32) -> RootId {
        (sw_cell_row_index as RootId) << 48 | (sw_cell_column_index as RootId) << 32 |
        (ne_cell_row_index as RootId) << 16 | (ne_cell_column_index as RootId)
    }

    /// Gets boundary cells (southwest and northeast) indexes from RootBox index.
    /// Returns the row and column of the southwest cell and the row and column of the northeast
    /// cell, respectively.
    pub fn get_boundary_cell_indexes(index: RootId) -> [u32; 4] {
        let mask = 0b1111_1111_1111_1111;

        let sw_cell_row_index = index >> 48;
        let sw_cell_column_index = (index >> 32) & mask;
        let ne_cell_row_index = (index >> 16) & mask;
        let ne_cell_column_index = index & mask;

        let indexes: [u32; 4] = [sw_cell_row_index as u32, sw_cell_column_index as u32,
                                 ne_cell_row_index as u32, ne_cell_column_index as u32];

        indexes
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
        let touch_vector = root_projection.south_west.get_distance_vector(touch);
        
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
    use crate::tests::{construct_custom_box, ROOT_ID, coord, Coord};

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

        let inner_mid_point = Point2D::new(coord("1"),
                                            coord("1"));
        assert_eq!(root.detect_intersected_area(inner_mid_point), 4); 

        let inner_edge_point = Point2D::new(coord("1"),
                                            coord("0.5"));
        assert_eq!(root.detect_intersected_area(inner_edge_point), 2); 

        let out_point = Point2D::new(coord("50"),
                                     coord("50"));
        assert_eq!(root.detect_intersected_area(out_point), 0);
    }

    #[test]
    fn extract_values_from_rootbox_index() {
        let rootbox_sw_cell_row: u64 = 0b0000_0000_0000_0101;
        let rootbox_sw_cell_column: u64 = 0b0000_0000_0000_1010;
        let rootbox_ne_cell_row: u64 = 0b0000_0000_0000_1111;
        let rootbox_ne_cell_column: u64 = 0b0000_0000_0000_0010;

        let rootbox_index = rootbox_sw_cell_row << 48 | rootbox_sw_cell_column << 32 |
            rootbox_ne_cell_row << 16 | rootbox_ne_cell_column;

        let indexes: [u32; 4] = RootBox::<Coord>::get_boundary_cell_indexes(rootbox_index);
        assert_eq!(indexes[0], 5);
        assert_eq!(indexes[1], 10);
        assert_eq!(indexes[2], 15);
        assert_eq!(indexes[3], 2);

        let rootbox_sw_cell_row: u64 = 0b0000_0000_0000_0101;
        let rootbox_sw_cell_column: u64 = 0b0000_0101_0011_1001;
        let rootbox_ne_cell_row: u64 = 0b0000_0000_0000_1111;
        let rootbox_ne_cell_column: u64 = 0b0111_1001_0001_1000;

        let rootbox_index = rootbox_sw_cell_row << 48 | rootbox_sw_cell_column << 32 |
            rootbox_ne_cell_row << 16 | rootbox_ne_cell_column;

        let indexes: [u32; 4] = RootBox::<Coord>::get_boundary_cell_indexes(rootbox_index);
        assert_eq!(indexes[0], 5);
        assert_eq!(indexes[1], 1337);
        assert_eq!(indexes[2], 15);
        assert_eq!(indexes[3], 31000);
    }

    #[test]
    fn get_rootbox_index() {
        let rootbox_sw_cell_row: u64 = 0b0000_0000_0000_0101;
        let rootbox_sw_cell_column: u64 = 0b0000_0000_0000_1010;
        let rootbox_ne_cell_row: u64 = 0b0000_0000_0000_1111;
        let rootbox_ne_cell_column: u64 = 0b0000_0000_0000_0010;

        let rootbox_index_expected = rootbox_sw_cell_row << 48 | rootbox_sw_cell_column << 32 |
            rootbox_ne_cell_row << 16 | rootbox_ne_cell_column;

        let cell_indexes: [u32; 4] = [5, 10, 15, 2];
        let rootbox_index = RootBox::<Coord>::get_index(cell_indexes[0], cell_indexes[1],
                                                        cell_indexes[2], cell_indexes[3]
        );
        assert_eq!(rootbox_index, rootbox_index_expected);

        let rootbox_sw_cell_row: u64 = 0b0000_0000_0000_0101;
        let rootbox_sw_cell_column: u64 = 0b0000_0101_0011_1001;
        let rootbox_ne_cell_row: u64 = 0b0000_0000_0000_1111;
        let rootbox_ne_cell_column: u64 = 0b0111_1001_0001_1000;

        let rootbox_index_expected = rootbox_sw_cell_row << 48 | rootbox_sw_cell_column << 32 |
            rootbox_ne_cell_row << 16 | rootbox_ne_cell_column;

        let cell_indexes: [u32; 4] = [5, 1337, 15, 31000];
        let rootbox_index = RootBox::<Coord>::get_index(cell_indexes[0], cell_indexes[1],
                                                        cell_indexes[2], cell_indexes[3]
        );
        assert_eq!(rootbox_index, rootbox_index_expected);

        let rootbox_sw_cell_row: u64 = 0b0001_0101_1010_0001;
        let rootbox_sw_cell_column: u64 = 0b0000_1110_1001_1001;
        let rootbox_ne_cell_row: u64 = 0b0001_0101_1101_1000;
        let rootbox_ne_cell_column: u64 = 0b0000_1110_1100_1110;

        let rootbox_index_expected = rootbox_sw_cell_row << 48 | rootbox_sw_cell_column << 32 |
            rootbox_ne_cell_row << 16 | rootbox_ne_cell_column;


        let cell_indexes: [u32; 4] = [5537, 3737, 5592, 3790];
        let rootbox_index = RootBox::<Coord>::get_index(cell_indexes[0], cell_indexes[1],
                                                        cell_indexes[2], cell_indexes[3]
        );
        assert_eq!(rootbox_index, rootbox_index_expected);
    }

    #[test]
    fn it_extracts_values_from_rootbox_index() {
        let rootbox_index = 0b0101_0000_0000_0000_1010_0000_0000_0000_1111_0000_0000_0000_0010;
        let indexes: [u32; 4] = RootBox::<Coord>::get_boundary_cell_indexes(rootbox_index);
        assert_eq!(indexes[0], 5);
        assert_eq!(indexes[1], 10);
        assert_eq!(indexes[2], 15);
        assert_eq!(indexes[3], 2);

        let rootbox_index = 0b0101_0000_0101_0011_1001_0000_0000_0000_1111_0111_1001_0001_1000;
        let indexes: [u32; 4] = RootBox::<Coord>::get_boundary_cell_indexes(rootbox_index);
        assert_eq!(indexes[0], 5);
        assert_eq!(indexes[1], 1337);
        assert_eq!(indexes[2], 15);
        assert_eq!(indexes[3], 31000);
    }

    #[test]
    fn it_gets_rootbox_index() {
        let cell_indexes: [u32; 4] = [5, 10, 15, 2];
        let rootbox_index = RootBox::<Coord>::get_index(cell_indexes[0], cell_indexes[1],
                                                        cell_indexes[2], cell_indexes[3]
        );
        assert_eq!(rootbox_index, 0b0101_0000_0000_0000_1010_0000_0000_0000_1111_0000_0000_0000_0010);

        let cell_indexes: [u32; 4] = [5, 1337, 15, 31000];
        let rootbox_index = RootBox::<Coord>::get_index(cell_indexes[0], cell_indexes[1],
                                                        cell_indexes[2], cell_indexes[3]
        );
        assert_eq!(rootbox_index, 0b0101_0000_0101_0011_1001_0000_0000_0000_1111_0111_1001_0001_1000);

        let cell_indexes: [u32; 4] = [5537, 3737, 5592, 3790];
        let rootbox_index = RootBox::<Coord>::get_index(cell_indexes[0], cell_indexes[1],
                                                        cell_indexes[2], cell_indexes[3]
        );
        assert_eq!(rootbox_index, 0b0001_0101_1010_0001_0000_1110_1001_1001_0001_0101_1101_1000_0000_1110_1100_1110);
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

#[derive(Encode, Decode, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page<Coord> {
    pub bitmap: [[RootId; PAGE_WIDTH as usize]; PAGE_LENGTH as usize],
    _phantom: PhantomData<Coord>,
}

impl<
    Coord: Default
    + FromStr
    + Copy
    + CastToType
> Page<Coord> {
    fn new() -> Self {
        Page {
            bitmap: [[0u64; PAGE_WIDTH as usize]; PAGE_LENGTH as usize],
            _phantom: PhantomData,
        }
    }

    /// Calculates the number of pages to extract from the storage using the coordinates
    pub fn get_amount_of_pages_to_extract_using_box(bounding_box: Box3D<Coord>) -> u32 {
        let (sw_row_index, sw_column_index) = Self::get_cell_indexes(bounding_box.south_west);
        let (ne_row_index, ne_column_index) = Self::get_cell_indexes(bounding_box.north_east);

        Self::get_amount_of_pages_to_extract(sw_row_index, sw_column_index, ne_row_index, ne_column_index)
    }

    /// Calculates the number of pages to extract from the storage using the indexes
    pub fn get_amount_of_pages_to_extract(
        sw_row_index: u32, sw_column_index: u32,
        ne_row_index: u32, ne_column_index: u32,
    ) -> u32 {
        let sw_cell_page_index = Self::get_index(sw_row_index, sw_column_index);
        let ne_cell_page_index = Self::get_index(ne_row_index, ne_column_index);

        if sw_cell_page_index == ne_cell_page_index {
            let amount_of_pages: u32 = 1;
            return amount_of_pages;
        }

        let (sw_page_row_index, sw_page_column_index) = Self::extract_values_from_page_index(sw_cell_page_index);
        let (ne_page_row_index, ne_page_column_index) = Self::extract_values_from_page_index(ne_cell_page_index);

        if ne_page_row_index < sw_page_row_index || ne_page_column_index < sw_page_column_index {
            return 0;
        }

        let mut offset: u32 = 1;

        if ne_page_row_index == sw_page_row_index {
            if sw_column_index / PAGE_WIDTH == 0 {
                return ne_page_column_index / PAGE_WIDTH;
            }
            return ne_page_column_index / PAGE_WIDTH - sw_page_column_index / PAGE_WIDTH + offset;
        }

        if sw_page_column_index == ne_page_column_index {
            if sw_row_index / PAGE_LENGTH == 0 {
                return ne_page_row_index / PAGE_LENGTH;
            }
            return ne_page_row_index / PAGE_LENGTH - sw_page_row_index / PAGE_LENGTH + offset;
        }

        if sw_row_index / PAGE_LENGTH == 0 && sw_column_index / PAGE_WIDTH == 0 {
            offset = 2;
        }

        (ne_page_row_index / PAGE_LENGTH - sw_page_row_index / PAGE_LENGTH) +
            (ne_page_column_index / PAGE_WIDTH - sw_page_column_index / PAGE_WIDTH)
            + offset
    }

    /// Gets the indexes of the pages to be extracted
    pub fn get_pages_indexes_to_be_extracted(
        amount_of_pages_to_extract: u32,
        sw_cell_row_index: u32, sw_cell_column_index: u32,
        sw_page_index: u32, ne_page_index: u32,
    ) -> Vec<PageId> {
        let mut page_indexes: Vec<PageId> = vec![sw_page_index];

        // Pages's bypass direction
        let right = 1;
        let up = 2;
        let left = 3;
        let down = 4;
        let mut direction = right;

        let mut current_cell_row_index: u32 = sw_cell_row_index;
        let mut current_cell_column_index: u32 = sw_cell_column_index;

        let first_page_index: PageId = sw_page_index;
        let (first_cell_row_index, first_cell_column_index) = Self::extract_values_from_page_index(first_page_index);
        let last_page_index: PageId = ne_page_index;
        let (last_cell_row_index, last_cell_column_index) = Self::extract_values_from_page_index(last_page_index);

        let mut next_page_index: PageId;

        // Bypass algorithm - counterclockwise, starting from the southwest
        for _ in 1..amount_of_pages_to_extract {
            next_page_index = Self::get_index(current_cell_row_index + PAGE_LENGTH, current_cell_column_index);
            let (next_cell_row_index, _) = Self::extract_values_from_page_index(next_page_index);

            if direction == right {
                if next_cell_row_index <= last_cell_row_index {
                    current_cell_row_index += PAGE_LENGTH;
                    page_indexes.push(next_page_index);
                    continue;
                } else {
                    direction = up;
                }
            }

            next_page_index = Self::get_index(current_cell_row_index, current_cell_column_index + PAGE_WIDTH);
            let (_, next_cell_column_index) = Self::extract_values_from_page_index(next_page_index);

            if direction == up {
                if next_cell_column_index <= last_cell_column_index {
                    current_cell_column_index += PAGE_WIDTH;
                    page_indexes.push(next_page_index);
                    continue;
                } else {
                    direction = left;
                }
            }

            next_page_index = Self::get_index(current_cell_row_index - PAGE_LENGTH, current_cell_column_index);
            let (next_cell_row_index, _) = Self::extract_values_from_page_index(next_page_index);

            if direction == left {
                if next_cell_row_index >= first_cell_row_index {
                    current_cell_row_index -= PAGE_LENGTH;
                    page_indexes.push(next_page_index);
                    continue;
                } else {
                    direction = down;
                }
            }

            next_page_index = Self::get_index(current_cell_row_index, current_cell_column_index - PAGE_WIDTH);
            let (_, next_cell_column_index) = Self::extract_values_from_page_index(next_page_index);
            if direction == down {
                if next_cell_column_index <= first_cell_column_index {
                    current_cell_column_index -= PAGE_WIDTH;
                    page_indexes.push(next_page_index);
                    continue;
                } else {
                    direction = right;
                }
            }
        }

        page_indexes
    }

    /// Gets the indexes of the cells where the point is located
    pub fn get_cell_indexes(point: Point3D<Coord>) -> (u32, u32) {
        let lat: u32 = point.lat.to_u32_with_frac_part(BITMAP_CELL_LENGTH, CELL_SIZE_DEGREE);
        let lon: u32 = point.lon.to_u32_with_frac_part(BITMAP_CELL_WIDTH, CELL_SIZE_DEGREE);

        let row_index: u32 = lat / BITMAP_CELL_LENGTH;
        let column_index: u32 = lon / BITMAP_CELL_WIDTH;

        (row_index, column_index)
    }

    /// Gets Page's index
    pub fn get_index(cell_row_index: u32, cell_column_index: u32) -> PageId {
        let row_index: u32;
        let column_index: u32;

        if cell_row_index > 0 && cell_row_index % PAGE_LENGTH != 0 {
            row_index = PAGE_LENGTH + cell_row_index - cell_row_index % PAGE_LENGTH;
        } else if cell_row_index == 0 {
            row_index = PAGE_LENGTH;
        } else {
            row_index = cell_row_index;
        }

        if cell_column_index > 0 && cell_column_index % PAGE_WIDTH != 0 {
            column_index = PAGE_WIDTH + cell_column_index - cell_column_index % PAGE_WIDTH;
        } else if cell_column_index == 0 {
            column_index = PAGE_WIDTH;
        } else {
            column_index = cell_column_index;
        }

        (row_index << 16) | column_index
    }

    /// Gets boundary cells (southwest and northeast) indexes from Page index.
    /// Returns the row and column of the southwest cell and the row and column of the northeast
    /// cell, respectively.
    pub fn get_boundary_cell_indexes(
        index: PageId, sw_cell_row_index: u32, ne_cell_column_index: u32,
    ) -> [u32; 4] {
        let mask = 0b1111_1111_1111_1111;

        let sw_cell_row_index = sw_cell_row_index - sw_cell_row_index % PAGE_LENGTH;
        let sw_cell_column_index = index & mask;
        let ne_cell_column_index = ne_cell_column_index - ne_cell_column_index % PAGE_WIDTH;
        let ne_cell_row_index = index >> 16;

        let indexes: [u32; 4] = [sw_cell_row_index as u32, sw_cell_column_index as u32,
            ne_cell_row_index as u32, ne_cell_column_index as u32];

        indexes
    }

    /// Gets cell indexes from Page index.
    /// Returns the row and column of the southwest cell and the row and column of the northeast
    /// cell, respectively.
    fn extract_values_from_page_index(page_index: PageId) -> (u32, u32) {
        let mask_u16: PageId = 0b1111_1111_1111_1111;
        let row_index: u32 = page_index >> 16;
        let column_index: u32 = page_index & mask_u16;
        (row_index, column_index)
    }

    #[cfg(test)]
    pub fn is_active(&self) -> bool {
        self.bitmap.is_empty()
    }
}

impl<Coord> Default for Page<Coord> {
    fn default() -> Self {
        Page{
            bitmap: [[0u64; PAGE_WIDTH as usize]; PAGE_LENGTH as usize],
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod page_tests {
    use super::*;
    use crate::tests::{construct_custom_box, coord, Coord};

    // These tests are built taking into account all possible rectangles from 4 Pages
    #[test]

    fn get_amount_of_pages_to_extract() {
        // 1 x 1
        let bounding_box = construct_custom_box( "0.0", "0.491", "0.301", "0.0");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 1);

        // 1 x 2
        let bounding_box = construct_custom_box("0.0", "0.0", "0.301", "1.0");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 2);

        // 2 x 1
        let bounding_box = construct_custom_box( "0.0", "0.01", "0.331", "0.0");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 2);

        // 1 x 3
        let bounding_box = construct_custom_box("0.0", "0.0", "0.301", "1.011");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 3);

        // 3 x 1
        let bounding_box = construct_custom_box("0.011", "0.0", "0.651", "0.011");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 3);

        // 1 x 4
        let bounding_box = construct_custom_box("0.0", "0.0", "0.301", "1.511");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 4);

        // 4 x 1
        let bounding_box = construct_custom_box( "0.011", "0.0", "0.981", "0.011");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 4);

        // 4 x 1
        let bounding_box = construct_custom_box( "0.051", "0.0", "1.271", "0.011");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 4);

        // 2 x 2
        let bounding_box = construct_custom_box("0.211", "0.011", "0.631", "0.991");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 4);

        // 2 x 2
        let bounding_box = construct_custom_box("0.05", "0.0", "0.5", "0.75");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 4);

        // 2 x 2
        let bounding_box = construct_custom_box( "55.37", "37.37", "55.92", "37.90");
        let pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(pages_to_extract, 4);
    }

    #[test]

    fn extract_values_from_page_index() {
        let page_sw_column_index: u32 = 0b0000_0000_0010_0000;
        let page_ne_row_index: u32 = 0b0000_0000_0011_0010;
        let page_index_expected: u32 = page_sw_column_index << 16 | page_ne_row_index;

        let point: Point3D<Coord> = Point3D::new(coord("0.011"), coord("0.011"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 1);
        assert_eq!(cell_column_index, 1);
        let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
        assert_eq!(page_index, page_index_expected);
        let (row_index, column_index) = Page::<Coord>::extract_values_from_page_index(page_index);
        assert_eq!(row_index, PAGE_LENGTH);
        assert_eq!(column_index, PAGE_WIDTH);

        let page_sw_column_index: u32 = 0b0000_0100_1110_0000;
        let page_ne_row_index: u32 = 0b0000_0000_0011_0010;
        let page_index_expected: u32 = page_sw_column_index << 16 | page_ne_row_index;

        let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("0.011"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 1225);
        assert_eq!(cell_column_index, 1);
        let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);

        assert_eq!(page_index, page_index_expected);
        let (row_index, column_index) = Page::<Coord>::extract_values_from_page_index(page_index);
        assert_eq!(row_index, 1248);
        assert_eq!(column_index, PAGE_WIDTH);

        let page_sw_column_index: u32 = 0b0000_0100_1110_0000;
        let page_ne_row_index: u32 = 0b0101_1011_1111_1110;
        let page_index_expected: u32 = page_sw_column_index << 16 | page_ne_row_index;

        let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("235.211"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 1225);
        assert_eq!(cell_column_index, 23521);
        let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
        assert_eq!(page_index, page_index_expected);
        let (row_index, column_index) = Page::<Coord>::extract_values_from_page_index(page_index);
        assert_eq!(row_index, 1248);
        assert_eq!(column_index, 23550);
    }

    #[test]
    fn get_page_index() {
        // The formula for getting page index from rows and columns is the same,
        // except for the shift, so only several cases are considered.
        // The entry 1-1 means 1 digit in the row index and 1 digit in the column index, and so on.

        // case 1-1
        let page_sw_column_index: u32 = 0b0000_0000_0010_0000;
        let page_ne_row_index: u32 = 0b0000_0000_0011_0010;
        let page_index_expected: u32 = page_sw_column_index << 16 | page_ne_row_index;

        let point: Point3D<Coord> = Point3D::new(coord("0.011"), coord("0.011"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 1);
        assert_eq!(cell_column_index, 1);
        let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);

        assert_eq!(page_index, page_index_expected);

        // case 2-1
        let page_sw_column_index: u32 = 0b0000_0000_0010_0000;
        let page_ne_row_index: u32 = 0b0000_0000_0011_0010;
        let page_index_expected: u32 = page_sw_column_index << 16 | page_ne_row_index;

        let point: Point3D<Coord> = Point3D::new(coord("0.251"), coord("0.011"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 25);
        assert_eq!(cell_column_index, 1);
        let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
        assert_eq!(page_index, page_index_expected);

        // case 3-1
        let page_sw_column_index: u32 = 0b0000_0001_0000_0000;
        let page_ne_row_index: u32 = 0b0000_0000_0011_0010;
        let page_index_expected: u32 = page_sw_column_index << 16 | page_ne_row_index;

        let point: Point3D<Coord> = Point3D::new(coord("2.251"), coord("0.011"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 225);
        assert_eq!(cell_column_index, 1);
        let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);

        assert_eq!(page_index, page_index_expected);

        // case 4-1
        let page_sw_column_index: u32 = 0b0000_0100_1110_0000;
        let page_ne_row_index: u32 = 0b0000_0000_0011_0010;
        let page_index_expected: u32 = page_sw_column_index << 16 | page_ne_row_index;

        let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("0.011"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 1225);
        assert_eq!(cell_column_index, 1);
        let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
        assert_eq!(page_index, page_index_expected);

        // case 5-1
        let page_sw_column_index: u32 = 0b0011_0100_0010_0000;
        let page_ne_row_index: u32 = 0b0000_0000_0011_0010;
        let page_index_expected: u32 = page_sw_column_index << 16 | page_ne_row_index;

        let point: Point3D<Coord> = Point3D::new(coord("133.251"), coord("0.011"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 13325);
        assert_eq!(cell_column_index, 1);
        let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);

        assert_eq!(page_index, page_index_expected);

        // case 4-5
        let page_sw_column_index: u32 = 0b0000_0100_1110_0000;
        let page_ne_row_index: u32 = 0b0101_1011_1111_1110;
        let page_index_expected: u32 = page_sw_column_index << 16 | page_ne_row_index;

        let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("235.211"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 1225);
        assert_eq!(cell_column_index, 23521);
        let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
        assert_eq!(page_index, page_index_expected);
    }

    #[test]
    fn calculate_cell_indexes() {
        let point: Point3D<Coord> = Point3D::new(coord("1.0"), coord("2.0"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::<Coord>::get_cell_indexes(point);
        assert_eq!(cell_row_index, 100);
        assert_eq!(cell_column_index, 200);

        let point: Point3D<Coord> = Point3D::new(coord("55.371"), coord("33.371"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 5537);
        assert_eq!(cell_column_index, 3337);

        let point: Point3D<Coord> = Point3D::new(coord("133.371"), coord("255.373"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 13337);
        assert_eq!(cell_column_index, 25537);

        let point: Point3D<Coord> = Point3D::new(coord("360.0"), coord("180.0"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 36000);
        assert_eq!(cell_column_index, 18000);

        let point: Point3D<Coord> = Point3D::new(coord("13.3778"), coord("255.3734"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 1337);
        assert_eq!(cell_column_index, 25537);

        let point: Point3D<Coord> = Point3D::new(coord("0.452"), coord("0.3003"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 45);
        assert_eq!(cell_column_index, 30);

        let point: Point3D<Coord> = Point3D::new(coord("55.37"), coord("33.37"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        // Because it is required minimum 3 non-zero (simultaneous) digits after the point
        assert_eq!(cell_row_index, 5536);
        assert_eq!(cell_column_index, 3336);

        let point: Point3D<Coord> = Point3D::new(coord("1.3778321"), coord("25.3222734"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 137);
        assert_eq!(cell_column_index, 2532);

        let point: Point3D<Coord> = Point3D::new(coord("1.301"), coord("25.301"), coord("1"));
        let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
        assert_eq!(cell_row_index, 130);
        assert_eq!(cell_column_index, 2530);
    }
}

type AreaId = u16;
type PageId = u32;
type LightCoord = u32;
type RootId = u64;
type ZoneId = u128;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: accounts::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
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
    + FromRaw
    + CastToType
    + Sub<Output = Self::Coord>
    + Div<Output = Self::Coord>;

    type RawCoord: Default 
    + Parameter 
    + Into<i32>
    + Copy;

    /// This allows us to have a top border for zones
    type MaxBuildingsInArea: Get<u16>;
    
    /// Max available height of any building
    type MaxHeight: Get<LightCoord>;
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
pub type ZoneOf<T> = Zone<<T as Trait>::Coord>;

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
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
        /// Area can't contain no more buildings
        AreaFull,
        /// Sizes are off bounds
        BadDimesions,
        /// Area is unavailable for operation
        ForbiddenArea,
        /// Operation is not valid
        InvalidAction,
        /// Incorrect coordinates were provided
        InvalidCoords,
        /// Incorrect data provided
        InvalidData,
        /// Error names should be descriptive
        NoneValue,
        /// Origin do not have sufficient privileges to perform the operation
        NotAuthorized,
        /// Account doesn't exist
        NotExists,
        /// Added root overlaps with another in current area
        OverlappingRoot,
        /// Added zone overlaps with another in current area
        OverlappingZone,
        /// The number of pages to be extracted exceeds the maximum
        PageLimitExceeded,
        /// Root you are trying to access is not in storage
        RootDoesNotExist,
        /// Zone points lies in different areas
        ZoneDoesntFit,
        /// Zone you are trying to access is not in storage
        ZoneDoesntExist,
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

        /// Adds new RootBox to storage
        #[weight = <T as Trait>::WeightInfo::root_add()]
        pub fn root_add(origin, bounding_box: Box3D<T::Coord>, delta: T::Coord) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);

            // Check amount of pages to be extracted
            let amount_of_pages_to_extract = Page::<T::Coord>::get_amount_of_pages_to_extract_using_box(bounding_box);
            ensure!(amount_of_pages_to_extract <= MAX_PAGES_AMOUNT_TO_EXTRACT, Error::<T>::PageLimitExceeded);

            // Check given coordinates
            let (sw_cell_row_index, sw_cell_column_index) = Page::<T::Coord>::get_cell_indexes(bounding_box.south_west);
            let sw_page_index = Page::<T::Coord>::get_index(sw_cell_row_index, sw_cell_column_index);
            let (ne_cell_row_index, ne_cell_column_index) = Page::<T::Coord>::get_cell_indexes(bounding_box.north_east);
            let ne_page_index = Page::<T::Coord>::get_index(ne_cell_row_index, ne_cell_column_index);
            ensure!(ne_page_index != 0 && sw_page_index != 0, Error::<T>::InvalidCoords);
            ensure!(sw_cell_column_index <= ne_cell_column_index, Error::<T>::InvalidCoords);
            ensure!(sw_cell_row_index <= ne_cell_row_index, Error::<T>::InvalidCoords);
            if ne_page_index == sw_page_index {
                ensure!(sw_cell_row_index <= ne_cell_row_index, Error::<T>::InvalidCoords);
            }

            let page_indexes: Vec<u32> = Page::<T::Coord>::get_pages_indexes_to_be_extracted(
                amount_of_pages_to_extract,
                sw_cell_row_index, sw_cell_column_index,
                sw_page_index, ne_page_index,
            );

            let id = RootBox::<T::Coord>::get_index(sw_cell_row_index, sw_cell_column_index,
                                                    ne_cell_row_index, ne_cell_column_index);
            let rootbox_boundary_cell_indexes = RootBox::<T::Coord>::get_boundary_cell_indexes(id);

            let mut updated_pages: Vec<Page<<T as Trait>::Coord>> = Vec::new();
            for page_index in page_indexes.clone() {
                let mut current_bitmap = EarthBitmap::<T>::get(page_index).bitmap;
                let page_boundary_cell_indexes = Page::<T::Coord>::get_boundary_cell_indexes(
                    page_index, sw_cell_row_index, ne_cell_column_index
                );

                let mut row_start = 0;
                if rootbox_boundary_cell_indexes[0] == page_boundary_cell_indexes[0] {
                    row_start = sw_cell_row_index % PAGE_LENGTH;
                }

                let mut row_end = PAGE_LENGTH;
                if rootbox_boundary_cell_indexes[2] == page_boundary_cell_indexes[2] {
                    row_end = ne_cell_row_index % PAGE_LENGTH;
                }

                for page_row in current_bitmap.iter_mut().take(row_end as usize).skip(row_start as usize) {
                    let mut column_start = 0;
                    if rootbox_boundary_cell_indexes[1] == page_boundary_cell_indexes[1] {
                        column_start = sw_cell_column_index % PAGE_WIDTH;
                    }

                    let mut column_end = PAGE_WIDTH;
                    if rootbox_boundary_cell_indexes[3] == page_boundary_cell_indexes[3] {
                        column_end = ne_cell_column_index % PAGE_WIDTH;
                    }

                    for cell in page_row.iter_mut().take(column_end as usize).skip(column_start as usize) {
                        ensure!(*cell == 0_u64, Error::<T>::OverlappingRoot);
                        *cell = id;
                    }
                }
                let mut page = Page::new();
                page.bitmap = current_bitmap;
                updated_pages.push(page);
            }

            for (page_number, page_index) in page_indexes.into_iter().enumerate() {
                EarthBitmap::<T>::insert(page_index, updated_pages[page_number]);
            }

            let root = RootBoxOf::<T>::new(id, bounding_box, delta);
            RootBoxes::<T>::insert(id, root);

            Self::deposit_event(RawEvent::RootCreated(id, who));
            Ok(())
        }
        
        /// TODO fix this trouble with types, RawCoord is a one big crutch
        #[weight = <T as Trait>::WeightInfo::root_add()]
        pub fn raw_root_add(origin, 
                            // Coords is SW {lat, lon, alt} NE {lat, lon, alt} 
                            raw_box: [T::RawCoord; 6],
                            raw_delta: T::RawCoord) -> dispatch::DispatchResult {
            let who = ensure_signed(origin.clone())?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);

            let south_west = Point3D::new(T::Coord::from_raw(raw_box[0].into()), 
                                          T::Coord::from_raw(raw_box[1].into()), 
                                          T::Coord::from_raw(raw_box[2].into()));
            let north_east = Point3D::new(T::Coord::from_raw(raw_box[3].into()), 
                                          T::Coord::from_raw(raw_box[4].into()), 
                                          T::Coord::from_raw(raw_box[5].into()));
            let bounding_box = Box3D::new(south_west, north_east);
            let delta = T::Coord::from_raw(raw_delta.into()); 

            Module::<T>::root_add(origin, bounding_box, delta)
        }

        /// Form index and store input to redzones, creates area struct if it doesnt exist
        #[weight = <T as Trait>::WeightInfo::zone_add()]
        pub fn zone_add(origin, 
                        rect: Rect2D<T::Coord>,
                        height: LightCoord,
                        root_id: RootId) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);
            ensure!(RootBoxes::<T>::contains_key(root_id), Error::<T>::RootDoesNotExist);
            ensure!(height < T::MaxHeight::get(), Error::<T>::InvalidData);
            // Check if zone lies in one single area 
            let area_id = RootBoxes::<T>::get(root_id).detect_intersected_area(rect.south_west);
            let se_area_id = RootBoxes::<T>::get(root_id).detect_intersected_area(rect.north_east);
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

        /// TODO fix this trouble with types, RawCoord is a one big crutch
        #[weight = <T as Trait>::WeightInfo::zone_add()]
        pub fn raw_zone_add(origin, 
                            raw_rect: [T::RawCoord; 4],
                            height: LightCoord,
                            root_id: RootId) -> dispatch::DispatchResult {
            let who = ensure_signed(origin.clone())?;
            ensure!(<accounts::Module<T>>::account_is(&who, REGISTRAR_ROLE.into()), Error::<T>::NotAuthorized);

            let south_west = Point2D::new(T::Coord::from_raw(raw_rect[0].into()), 
                                          T::Coord::from_raw(raw_rect[1].into()));
            let north_east = Point2D::new(T::Coord::from_raw(raw_rect[2].into()), 
                                          T::Coord::from_raw(raw_rect[3].into()));
            let rect = Rect2D::new(south_west, north_east);

            Module::<T>::zone_add(origin, rect, height, root_id)
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

            // Recursively clear all cells in bitmap
            let rootbox_boundary_cell_indexes = RootBox::<T::Coord>::get_boundary_cell_indexes(root_id);
            let sw_cell_row_index = rootbox_boundary_cell_indexes[0];
            let sw_cell_column_index = rootbox_boundary_cell_indexes[1];
            let ne_cell_row_index = rootbox_boundary_cell_indexes[2];
            let ne_cell_column_index = rootbox_boundary_cell_indexes[3];

            let amount_of_pages_to_extract = Page::<T::Coord>::get_amount_of_pages_to_extract(
                sw_cell_row_index, sw_cell_column_index,
                ne_cell_row_index, ne_cell_column_index,
            );
            let sw_page_index = Page::<T::Coord>::get_index(sw_cell_row_index, sw_cell_column_index);
            let ne_page_index = Page::<T::Coord>::get_index(ne_cell_row_index, ne_cell_column_index);

            let page_indexes: Vec<u32> = Page::<T::Coord>::get_pages_indexes_to_be_extracted(
                amount_of_pages_to_extract,
                sw_cell_row_index, sw_cell_column_index,
                sw_page_index, ne_page_index,
            );

            let mut updated_pages: Vec<Page<<T as Trait>::Coord>> = Vec::new();
            for page_index in page_indexes.clone() {
                let mut current_bitmap = EarthBitmap::<T>::get(page_index).bitmap;
                let page_boundary_cell_indexes = Page::<T::Coord>::get_boundary_cell_indexes(
                    page_index, sw_cell_row_index, ne_cell_column_index
                );

                let mut row_start = 0;
                if rootbox_boundary_cell_indexes[0] == page_boundary_cell_indexes[0] {
                    row_start = sw_cell_row_index % PAGE_LENGTH;
                }

                let mut row_end = PAGE_LENGTH;
                if rootbox_boundary_cell_indexes[2] == page_boundary_cell_indexes[2] {
                    row_end = ne_cell_row_index % PAGE_LENGTH;
                }

                for page_row in current_bitmap.iter_mut().take(row_end as usize).skip(row_start as usize) {
                    let mut column_start = 0;
                    if rootbox_boundary_cell_indexes[1] == page_boundary_cell_indexes[1] {
                        column_start = sw_cell_column_index % PAGE_WIDTH;
                    }

                    let mut column_end = PAGE_WIDTH;
                    if rootbox_boundary_cell_indexes[3] == page_boundary_cell_indexes[3] {
                        column_end = ne_cell_column_index % PAGE_WIDTH;
                    }

                    for cell in page_row.iter_mut().take(column_end as usize).skip(column_start as usize) {
                        ensure!(*cell != 0, Error::<T>::InvalidData);
                        *cell = 0;
                    }
                }
                let mut page = Page::new();
                page.bitmap = current_bitmap;
                updated_pages.push(page);
            }

            for (page_number, page_index) in page_indexes.into_iter().enumerate() {
                EarthBitmap::<T>::insert(page_index, updated_pages[page_number]);
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
    #[allow(dead_code)]
    fn coord_from_str<Coord> (s: &str) -> Coord
            where Coord: FromStr + Default {
        match Coord::from_str(s) {
            Ok(v) => v,
            Err(_) => Default::default(),
        }
    } 
    
    #[allow(dead_code)]
    fn get_root_index(raw_point: [i32; 2]) -> RootId {
        let lat = T::Coord::from_raw(raw_point[0]);
        let lon = T::Coord::from_raw(raw_point[1]);
        let alt = T::Coord::from_raw(0);

        let point = Point3D::<T::Coord>::new(lat, lon, alt);
        let (row, column) = Page::<T::Coord>::get_cell_indexes(point);
        let index = Page::<T::Coord>::get_index(row, column);
        let bitmap = EarthBitmap::<T>::get(index).bitmap;

        bitmap[(row % PAGE_LENGTH) as usize][(column % PAGE_WIDTH) as usize]
    }

    /// Form index for storing zones, wrapped in u128............limited by const in runtime
    /// v................root id here..............v v.....area id.....v v..child objects..v
    /// 0000 0000 0000 0000 .... 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000 0000
    fn pack_index(root: RootId, area: AreaId, children: u16) -> ZoneId {
        (root as ZoneId) << 64 |
        (area as ZoneId) << 16 | 
        children as ZoneId
    }

    /// Reverse function for pack_index()
    #[allow(dead_code)]
    fn unpack_index(index: ZoneId) -> (RootId, AreaId, u16) {
        let mask_u16: u128 = 0x0000_0000_0000_0000_0000_0000_ffff_ffff;
        let root: RootId = (index >> 64) as RootId;
        let area: AreaId = ((index >> 16) & mask_u16) as AreaId;
        let children: u16 = (index & mask_u16) as u16;
        
        (root, area, children)
    }
}

