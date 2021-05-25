use crate::mock::*;
use crate::{Point3D, Box3D,
            Point2D, Rect2D,
            Page,
            PAGE_LENGTH, PAGE_WIDTH
};
use frame_support::{
    assert_noop, assert_ok,
};
use substrate_fixed::types::I10F22;
use sp_std::str::FromStr;

// Explanation for all hardcoded values down here
//                             Root        P2(55.92, 37.90)
//            +----+-------------------------+----O
//            |2861|                         |2915|
//            |    | Area 2861               |    |
//            +----+                         +----+
//            |                                   |
//            |                                   |
//            |                                   |
//            +----+        Moscow                |
//            |111 |                              |
//            |    |                              |
//            +----+----+----+  Zone(55.395,      |
//            | 56 | 57 | 58 |<----  37.385)      |
//            |    |    |    |                    |
//        +-> +----+----+----+               +----+
//        |   | 1  | 2  | 3  |               | 55 |
//  delta |   |    |    |    |               |    |
//  =0.01 +-> O----+----+----+---------------+----+
//       Origin(55.37, 37.37)
//
//
//                   Area 58       (55.400, 37.390)
//    +-----------------------------------o
//    |                                   |
//    |                                   |
//    |                                   |
//    |                 rect2D            |
//    |                +--------o(55.396, |
//    |                |        | 37.386) |
//    |                |testing |         |
//    |                |  zone  |         |
//    |                |        |         |
//    |                o--------+         |
//    |               (55.395,            |
//    |                37.385)            |
//    |                                   |
//    |                                   |
//    o-----------------------------------+
// (55.390, 37,380)

type Error = super::Error<Test>;
pub type Coord = I10F22;

// Constants to make tests more readable
const ADMIN_ACCOUNT_ID: u64 = 1;
const REGISTRAR_1_ACCOUNT_ID: u64 = 2;
pub const ROOT_ID: u32 = 1;
// this value, and values in construct_testing_..() was calculated
const AREA_ID: u16 = 58;
const DEFAULT_HEIGHT: u16 = 30;

const DELTA: &str = "0.01";

// shortcut for &str -> Coord
pub fn coord<Coord>(s: &str) -> Coord
    where Coord: FromStr,
        <Coord as FromStr>::Err: std::fmt::Debug { Coord::from_str(s).unwrap() }

fn construct_testing_box() -> Box3D<Coord> {
    let north_east = Point3D::new(coord("55.37"),
                                  coord("37.37"), 
                                  coord("1"));
    let south_west = Point3D::new(coord("55.92"),
                                  coord("37.90"),       
                                  coord("3"));      
    Box3D::new(north_east, south_west)
}

pub fn construct_custom_box(ne_lat: &str, ne_lon: &str, sw_lat: &str, sw_lon: &str) -> Box3D<Coord> {
    let north_east = Point3D::new(coord(ne_lat),
                                  coord(ne_lon),
                                  coord("1"));
    let south_west = Point3D::new(coord(sw_lat),
                                  coord(sw_lon),
                                  coord("3"));      
    Box3D::new(north_east, south_west)
}

pub fn construct_custom_rect(ne_lat: &str, ne_lon: &str, sw_lat: &str, sw_lon: &str) -> Rect2D<Coord> {
    let north_east = Point2D::new(coord(ne_lat),
                                  coord(ne_lon));
    let south_west = Point2D::new(coord(sw_lat),
                                  coord(sw_lon));
    Rect2D::new(north_east, south_west)
}

fn construct_testing_rect() -> Rect2D<Coord> {
    let north_east = Point2D::new(coord("55.395"),
                                  coord("37.385"));
    let south_west = Point2D::new(coord("55.396"),
                                  coord("37.386"));
    Rect2D::new(north_east, south_west)
}

#[test]
fn it_tries_to_add_root_unauthorized() {
    new_test_ext().execute_with(|| {
        let account = DSAccountsModule::account_registry(2);
        assert!(!account.is_enabled());

        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_testing_box(),
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_tries_to_add_root_by_registrar() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
            )
        );
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
            )
        );
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_testing_box(),
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_tries_to_add_too_big_root() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_box("250.0", "0.0",
                                     "0.0", "250.0"),
            ),
            Error::PageLimitExceeded
        );
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_box("45.1", "0.0",
                                     "0.0", "50.9"),
            ),
            Error::PageLimitExceeded
        );
    });
}

#[test]
fn it_tries_remove_root() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
        ));
        let root = DSMapsModule::root_box_data(ROOT_ID);
        assert!(root.is_active());
        assert_ok!(
            DSMapsModule::root_remove(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                ROOT_ID,
        ));
        let root = DSMapsModule::root_box_data(ROOT_ID);
        assert!(!root.is_active());
    });
}

#[test]
fn it_tries_add_zone_unauthorized() {
    new_test_ext().execute_with(|| {
        let account = DSAccountsModule::account_registry(2);
        assert!(!account.is_enabled());
        
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_tries_add_zone_to_not_existing_root() {
    new_test_ext().execute_with(|| {
        let account = DSAccountsModule::account_registry(2);
        assert!(!account.is_enabled());
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
            ),
            Error::RootDoesNotExist
        );
    });
}

#[test]
fn it_tries_add_zone_by_registrar() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_tries_get_zone() {
    new_test_ext().execute_with(|| {
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        let zone = DSMapsModule::zone_data(DSMapsModule::pack_index(ROOT_ID, AREA_ID, 0));
        assert!(construct_testing_rect() == zone.rect);
    });
}

#[test]
fn it_tries_remove_zone() {
    new_test_ext().execute_with(|| {
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        let zone_index = DSMapsModule::pack_index(ROOT_ID, AREA_ID, 0);
        let zone = DSMapsModule::zone_data(zone_index);
        assert!(construct_testing_rect() == zone.rect);
        assert_ok!(
            DSMapsModule::zone_remove(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                zone_index,
        ));
        let zone = DSMapsModule::zone_data(zone_index);
        // Guess, not the best way to check, but it works
        assert!(construct_custom_rect("0", "0", "0", "0") == zone.rect);
    });
}

#[test]
fn it_tries_add_zone_which_lies_in_different_areas() {
    new_test_ext().execute_with(|| {
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
        ));
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_rect("55.395", "37.385",
                                      "56.396", "37.901"),
                DEFAULT_HEIGHT, 
                ROOT_ID,
            ),
            Error::ZoneDoesntFit
        );
    });
}

#[test]
fn it_tries_add_overlapping_zones() {
    new_test_ext().execute_with(|| {
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
            ),
            Error::OverlappingZone
        );
    });
}

#[test]
fn it_tries_add_not_overlapping_zones() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_rect("55.391", "37.381", 
                                      "55.392", "37.382"),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
    });
}

#[test]
fn it_tries_add_more_than_max_zones() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_rect("55.391", "37.381", 
                                      "55.392", "37.382"),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_rect("55.393", "37.383", 
                                      "55.394", "37.384"),
                DEFAULT_HEIGHT, 
                ROOT_ID,
            ), 
            Error::AreaFull
        );
    });
}

#[test]
fn it_changes_not_existing_area_type() {    
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
        ));
        assert_noop!(
            DSMapsModule::change_area_type(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),  
                ROOT_ID,
                AREA_ID,
                0
            ),
            Error::NotExists
        );
    });
}

#[test]
fn it_adds_restricted_size_root() {    
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_box("55.37", "37.37", "56.92", "37.90"),
            ),
            Error::BadDimesions
        );
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
            ),
            Error::InvalidData
        );
    });
}

#[test]
fn it_changes_existing_area_type() {    
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_ok!(
            DSMapsModule::change_area_type(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID), 
                ROOT_ID,
                AREA_ID,
                0
        ));
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
            ), 
            Error::ForbiddenArea
        );
    });
}

#[test]
fn it_calculates_cell_indexes() {
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

#[test]
fn it_gets_amount_of_pages_to_extract() {
    let bounding_box = construct_custom_box("0.301", "0.0", "0.0", "0.491");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 1);

    let bounding_box = construct_custom_box("0.301", "0.0", "0.0", "1.011");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 3);

    let bounding_box = construct_custom_box("0.641", "0.0", "0.0", "1.011");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 4);

    let bounding_box = construct_custom_box("0.641", "0.0", "0.211", "1.011");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 4);
}

#[test]
fn it_extracts_values_from_page_index() {
    let point: Point3D<Coord> = Point3D::new(coord("0.011"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_page_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0010_0000_0000_0000_0011_0010);
    let (row_index, column_index) = Page::<Coord>::extract_values_from_page_index(page_index);
    assert_eq!(row_index, PAGE_LENGTH as u32);
    assert_eq!(column_index, PAGE_WIDTH as u32);

    let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1225);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_page_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0100_1110_0000_0000_0000_0011_0010);
    let (row_index, column_index) = Page::<Coord>::extract_values_from_page_index(page_index);
    assert_eq!(row_index, 1248);
    assert_eq!(column_index, PAGE_WIDTH as u32);

    let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("235.211"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1225);
    assert_eq!(cell_column_index, 23521);
    let page_index = Page::<Coord>::get_page_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0100_1110_0000_0101_1011_1111_1110);
    let (row_index, column_index) = Page::<Coord>::extract_values_from_page_index(page_index);
    assert_eq!(row_index, 1248);
    assert_eq!(column_index, 23550);
}

#[test]
fn it_gets_page_index() {
    // The formula for getting page index from rows and columns is the same,
    // except for the shift, so only several cases are considered.
    // The entry 1-1 means 1 digit in the row index and 1 digit in the column index, and so on.

    // case 1-1
    let point: Point3D<Coord> = Point3D::new(coord("0.011"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_page_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0010_0000_0000_0000_0011_0010);

    // case 2-1
    let point: Point3D<Coord> = Point3D::new(coord("0.251"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 25);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_page_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0010_0000_0000_0000_0011_0010);

    // case 3-1
    let point: Point3D<Coord> = Point3D::new(coord("2.251"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 225);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_page_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0001_0000_0000_0000_0000_0011_0010);

    // case 4-1
    let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1225);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_page_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0100_1110_0000_0000_0000_0011_0010);

    // case 5-1
    let point: Point3D<Coord> = Point3D::new(coord("133.251"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 13325);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_page_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0011_0100_0010_0000_0000_0000_0011_0010);

    // case 4-5
    let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("235.211"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1225);
    assert_eq!(cell_column_index, 23521);
    let page_index = Page::<Coord>::get_page_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0100_1110_0000_0101_1011_1111_1110);
}
