use crate::mock::*;
use crate::{
            Page,
            Point3D, Box3D,
            Point2D, Rect2D,
            PAGE_LENGTH, PAGE_WIDTH,
            RootBox,
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
pub const ROOT_ID: u64 = 1;
// this value, and values in construct_testing_..() was calculated
const AREA_ID: u16 = 58;
const DEFAULT_HEIGHT: u16 = 30;

const DELTA: &str = "0.01";

// shortcut for &str -> Coord
pub fn coord<Coord>(s: &str) -> Coord
    where Coord: FromStr,
        <Coord as FromStr>::Err: std::fmt::Debug { Coord::from_str(s).unwrap() }

fn construct_testing_box() -> Box3D<Coord> {
    let south_west = Point3D::new(coord("55.37"),
                                  coord("37.37"), 
                                  coord("1"));
    let north_east = Point3D::new(coord("55.92"),
                                  coord("37.90"),       
                                  coord("3"));      
    Box3D::new(south_west, north_east)
}

pub fn construct_custom_box(nw_lat: &str, nw_lon: &str, se_lat: &str, se_lon: &str) -> Box3D<Coord> {
    let south_west = Point3D::new(coord(nw_lat),
                                  coord(nw_lon), 
                                  coord("1"));
    let north_east = Point3D::new(coord(se_lat),
                                  coord(se_lon),       
                                  coord("3"));      
    Box3D::new(south_west, north_east)
}

fn construct_testing_rect() -> Rect2D<Coord> {
    let south_west = Point2D::new(coord("55.395"),
                                  coord("37.385"));
    let north_east = Point2D::new(coord("55.396"),
                                  coord("37.386"));
    Rect2D::new(south_west, north_east)
}

pub fn construct_custom_rect(nw_lat: &str, nw_lon: &str, se_lat: &str, se_lon: &str) -> Rect2D<Coord> {
    let south_west = Point2D::new(coord(nw_lat),
                                  coord(nw_lon));
    let north_east = Point2D::new(coord(se_lat),
                                  coord(se_lon));
    Rect2D::new(south_west, north_east)
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
                coord(DELTA),
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
                construct_custom_box("55.37", "37.90", "55.92", "37.37"),
                coord(DELTA),
            )
        );
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_testing_box(),
                coord(DELTA),
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_tries_to_add_raw_root_with_exceeded_page_limit() {
      new_test_ext().execute_with(|| {
          assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
          ));
          let raw_coords: [i32; 6] = [
              465587600,
              318558719,
              8388608,
              469815744,
              312529919,
              16777216
          ];
          let delta: i32 = 838860;
          assert_noop!(
            DSMapsModule::raw_root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                raw_coords,
                delta
            ),
            Error::PageLimitExceeded
          );

          let root = DSMapsModule::root_box_data(ROOT_ID);
          assert!(!root.is_active());
      });
}

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
                construct_custom_box("0.0", "250.0", "250.0", "0.0",),
                coord(DELTA),
            ),
            Error::PageLimitExceeded
        );
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_box("0.0", "50.9", "45.1", "0.0"),
                coord(DELTA),
            ),
            Error::PageLimitExceeded
        );
    });
}

#[test]
fn it_tries_to_add_root_with_incorrect_coordinates() {
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
                construct_custom_box("250.0", "0.0", "0.0", "250.0"),
                coord(DELTA),
            ),
            Error::InvalidCoords
        );
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_box("100.0", "50.9", "0.1", "0.0"),
                coord(DELTA),
            ),
            Error::InvalidCoords
        );
    });
}

// 4 pages as square (2x2)
// +++++++++
// |___+___|
// |___+___|
// |+++++++|
// |___+___|
// |___+___|
// +++++++++
#[test]
fn it_tries_to_add_root_as_square_2x2() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));

        let bounding_box = construct_custom_box("0.051", "0.75", "0.5", "0.0", );
        let (sw_cell_row_index, sw_cell_column_index) = Page::get_cell_indexes(bounding_box.south_west);
        assert_eq!(sw_cell_row_index, 5);
        assert_eq!(sw_cell_column_index, 75);
        let (ne_cell_row_index, ne_cell_column_index) = Page::get_cell_indexes(bounding_box.north_east);
        assert_eq!(ne_cell_row_index, 50);
        assert_eq!(ne_cell_column_index, 0);

        let amount_of_pages_to_extract = Page::get_amount_of_pages_to_extract(bounding_box);
        assert_eq!(amount_of_pages_to_extract, 4);

        let sw_page_index = Page::<Coord>::get_index(sw_cell_row_index, sw_cell_column_index);
        let ne_page_index = Page::<Coord>::get_index(ne_cell_row_index, ne_cell_column_index);

        let pages_indexes = Page::<Coord>::get_pages_indexes_to_be_extracted(
            amount_of_pages_to_extract,
            sw_cell_row_index, sw_cell_column_index,
            sw_page_index, ne_page_index,
        );

        let root_id = RootBox::<Coord>::get_index(sw_cell_row_index, sw_cell_column_index,
                                                     ne_cell_row_index, ne_cell_column_index);
        let root = RootBox::<Coord>::new(root_id, bounding_box, coord(DELTA));

        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                bounding_box,
                coord(DELTA),
        ));
    });
}

// 4 pages as rectangle (4x1)
// +++++++++++++++++
// |___+___+___+___|
// |___+___+___+___|
// +++++++++++++++++
#[test]
fn it_tries_to_add_root_as_rectangle_4x1() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));

        let bounding_box = construct_custom_box("0.051", "0.011", "1.271", "0.0");
        let (sw_cell_row_index, sw_cell_column_index) = Page::get_cell_indexes(bounding_box.south_west);
        assert_eq!(sw_cell_row_index, 5);
        assert_eq!(sw_cell_column_index, 1);
        let (ne_cell_row_index, ne_cell_column_index) = Page::get_cell_indexes(bounding_box.north_east);
        assert_eq!(ne_cell_row_index, 127);
        assert_eq!(ne_cell_column_index, 0);

        let amount_of_pages_to_extract = Page::get_amount_of_pages_to_extract(bounding_box);
        assert_eq!(amount_of_pages_to_extract, 4);

        let sw_page_index = Page::<Coord>::get_index(sw_cell_row_index, sw_cell_column_index);
        let ne_page_index = Page::<Coord>::get_index(ne_cell_row_index, ne_cell_column_index);

        let pages_indexes = Page::<Coord>::get_pages_indexes_to_be_extracted(
            amount_of_pages_to_extract,
            sw_cell_row_index, sw_cell_column_index,
            sw_page_index, ne_page_index,
        );

        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                bounding_box,
                coord(DELTA),
        ));
    });
}

// 4 pages as rectangle (1x4)
// +++++
// |___|
// |___|
// |+++|
// |___|
// |___|
// |+++|
// |___|
// |___|
// |+++|
// |___|
// |___|
// +++++
#[test]
fn it_tries_to_add_root_as_rectangle_1x4() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));

        let bounding_box = construct_custom_box("0.0", "1.751", "0.011", "0.0");
        let (sw_cell_row_index, sw_cell_column_index) = Page::get_cell_indexes(bounding_box.south_west);
        assert_eq!(sw_cell_row_index, 0);
        assert_eq!(sw_cell_column_index, 175);
        let (ne_cell_row_index, ne_cell_column_index) = Page::get_cell_indexes(bounding_box.north_east);
        assert_eq!(ne_cell_row_index, 1);
        assert_eq!(ne_cell_column_index, 0);

        let amount_of_pages_to_extract = Page::get_amount_of_pages_to_extract(bounding_box);
        assert_eq!(amount_of_pages_to_extract, 4);

        let sw_page_index = Page::<Coord>::get_index(sw_cell_row_index, sw_cell_column_index);
        let ne_page_index = Page::<Coord>::get_index(ne_cell_row_index, ne_cell_column_index);

        let pages_indexes = Page::<Coord>::get_pages_indexes_to_be_extracted(
            amount_of_pages_to_extract,
            sw_cell_row_index, sw_cell_column_index,
            sw_page_index, ne_page_index,
        );

        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                bounding_box,
                coord(DELTA),
        ));
    });
}

#[test]
fn it_tries_to_remove_root() {
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
                coord(DELTA),
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
fn it_tries_to_add_zone_unauthorized() {
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
fn it_tries_to_add_zone_to_not_existing_root() {
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
fn it_tries_to_add_zone_by_registrar() {
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
                coord(DELTA),
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
fn it_tries_to_get_zone() {
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
                coord(DELTA),
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
fn it_tries_to_remove_zone() {
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
                coord(DELTA),
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
fn it_tries_to_add_zone_which_lies_in_different_areas() {
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
                coord(DELTA),
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
fn it_tries_to_add_overlapping_zones() {
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
                coord(DELTA),
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
fn it_tries_to_add_not_overlapping_zones() {
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
                coord(DELTA),
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
fn it_tries_to_add_more_than_max_zones() {
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
                coord(DELTA),
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
                coord(DELTA),
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
                coord(DELTA),
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

// These tests are built taking into account all possible rectangles from 4 Pages
#[test]
fn it_gets_amount_of_pages_to_extract() {
    // 1 x 1
    let bounding_box = construct_custom_box( "0.0", "0.491", "0.301", "0.0");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 1);

    // 1 x 2
    let bounding_box = construct_custom_box("0.0", "1.0", "0.301", "0.0");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 2);

    // 2 x 1
    let bounding_box = construct_custom_box( "0.0", "0.01", "0.331", "0.0");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 2);

    // 1 x 3
    let bounding_box = construct_custom_box("0.0", "1.011", "0.301", "0.0");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 3);

    // 3 x 1
    let bounding_box = construct_custom_box("0.011", "0.011", "0.651", "0.0");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 3);

    // 1 x 4
    let bounding_box = construct_custom_box("0.0", "1.511", "0.301", "0.0");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 4);

    // 4 x 1
    let bounding_box = construct_custom_box( "0.011", "0.011", "0.981", "0.0");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 4);

    // 4 x 1
    let bounding_box = construct_custom_box( "0.051", "0.011", "1.271", "0.0");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 4);

    // 2 x 2
    let bounding_box = construct_custom_box("0.211", "0.991", "0.631", "0.011");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 4);

    // 2 x 2
    let bounding_box = construct_custom_box("0.05", "0.75", "0.5", "0.0");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 4);

    // 2 x 2
    let bounding_box = construct_custom_box( "55.37", "37.90", "55.92", "37.37");
    let pages_to_extract = Page::<Coord>::get_amount_of_pages_to_extract(bounding_box);
    assert_eq!(pages_to_extract, 4);
}

#[test]
fn it_extracts_values_from_page_index() {
    let point: Point3D<Coord> = Point3D::new(coord("0.011"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0010_0000_0000_0000_0011_0010);
    let (row_index, column_index) = Page::<Coord>::extract_values_from_page_index(page_index);
    assert_eq!(row_index, PAGE_LENGTH as u32);
    assert_eq!(column_index, PAGE_WIDTH as u32);

    let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1225);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0100_1110_0000_0000_0000_0011_0010);
    let (row_index, column_index) = Page::<Coord>::extract_values_from_page_index(page_index);
    assert_eq!(row_index, 1248);
    assert_eq!(column_index, PAGE_WIDTH as u32);

    let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("235.211"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1225);
    assert_eq!(cell_column_index, 23521);
    let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
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
    let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0010_0000_0000_0000_0011_0010);

    // case 2-1
    let point: Point3D<Coord> = Point3D::new(coord("0.251"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 25);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0010_0000_0000_0000_0011_0010);

    // case 3-1
    let point: Point3D<Coord> = Point3D::new(coord("2.251"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 225);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0001_0000_0000_0000_0000_0011_0010);

    // case 4-1
    let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1225);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0100_1110_0000_0000_0000_0011_0010);

    // case 5-1
    let point: Point3D<Coord> = Point3D::new(coord("133.251"), coord("0.011"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 13325);
    assert_eq!(cell_column_index, 1);
    let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0011_0100_0010_0000_0000_0000_0011_0010);

    // case 4-5
    let point: Point3D<Coord> = Point3D::new(coord("12.251"), coord("235.211"), coord("1"));
    let (cell_row_index, cell_column_index) = Page::get_cell_indexes(point);
    assert_eq!(cell_row_index, 1225);
    assert_eq!(cell_column_index, 23521);
    let page_index = Page::<Coord>::get_index(cell_row_index, cell_column_index);
    assert_eq!(page_index, 0b0100_1110_0000_0101_1011_1111_1110);
}

#[test]
fn it_extracts_values_from_rootbox_index() {
    let rootbox_index = 0b0101_0000_0000_0000_1010_0000_0000_0000_1111_0000_0000_0000_0010;
    let indexes: [u32; 4] = RootBox::<Coord>::get_boundary_cells_indexes(rootbox_index);
    assert_eq!(indexes[0], 5);
    assert_eq!(indexes[1], 10);
    assert_eq!(indexes[2], 15);
    assert_eq!(indexes[3], 2);

    let rootbox_index = 0b0101_0000_0101_0011_1001_0000_0000_0000_1111_0111_1001_0001_1000;
    let indexes: [u32; 4] = RootBox::<Coord>::get_boundary_cells_indexes(rootbox_index);
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
}
