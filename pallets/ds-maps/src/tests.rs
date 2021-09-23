use crate::mock::*;
use crate::{
            Page,
            Point3D, Box3D,
            Point2D, Rect2D,
            Waypoint,
};
use frame_support::{
    assert_noop, assert_ok,
};
use substrate_fixed::types::I10F22;
use sp_std::str::FromStr;

// Explanation for all hardcoded values down here
//                             Root        P2(55.921, 37.901)
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
//       Origin(55.371, 37.371)
//
// (wp1, wp2) is a testing waypoints
//                   Area 58       (55.400, 37.390)
//    +-----------------------------------o
//    |                                   |
//    |                             (x)   |
//    |                            (wp2)  |
//    |                 rect2D            |
//    |                +--------o(55.396, |
//    |                |        | 37.386) |
//    |                |  (x)   |         |
//    |                | (wp1)  |         |
//    |                |        |         |
//    |                o--------+         |
//    |               (55.395,            |
//    |                37.385)            |
//    |                                   |
//    |                                   |
//    o-----------------------------------+
// (55.390, 37,380)

type Error = super::Error<Test>;
// TODO find out how to connect this types w mock
pub type Coord = I10F22;
type Moment = u64;

// Constants to make tests more readable
const ADMIN_ACCOUNT_ID: u64 = 1;
const REGISTRAR_1_ACCOUNT_ID: u64 = 2;
pub const ROOT_ID: u64 = 0b0001_0101_1010_0001_0000_1110_1001_1001_0001_0101_1101_1000_0000_1110_1100_1110;
// Values in construct_testing_..() pre-calculated
// construct_custom_..() same functionality, but custom numbers
// These consts also pre-calculated

const AREA_ID: u16 = 58;
const DEFAULT_HEIGHT: u32 = 30;

pub const DELTA: &str = "0.01";

// shortcut for &str -> Coord
pub fn coord<Coord>(s: &str) -> Coord
    where Coord: FromStr,
        <Coord as FromStr>::Err: std::fmt::Debug { Coord::from_str(s).unwrap() }

fn construct_testing_box() -> Box3D<Coord> {
    let south_west = Point3D::new(coord("55.371"),
                                  coord("37.371"),
                                  coord("1"));
    let north_east = Point3D::new(coord("55.921"),
                                  coord("37.901"),
                                  coord("3"));      
    Box3D::new(south_west, north_east)
}

pub fn construct_custom_box(sw_lat: &str, sw_lon: &str, ne_lat: &str, ne_lon: &str) -> Box3D<Coord> {
    let south_west = Point3D::new(coord(sw_lat),
                                  coord(sw_lon),
                                  coord("1"));
    let north_east = Point3D::new(coord(ne_lat),
                                  coord(ne_lon),
                                  coord("3"));      
    Box3D::new(south_west, north_east)
}

pub fn construct_testing_rect() -> Rect2D<Coord> {
    let south_west = Point2D::new(coord("55.395"),
                                  coord("37.385"));
    let north_east = Point2D::new(coord("55.396"),
                                  coord("37.386"));
    Rect2D::new(south_west, north_east)
}

pub fn construct_custom_rect(sw_lat: &str, sw_lon: &str, ne_lat: &str, ne_lon: &str) -> Rect2D<Coord> {
    let south_west = Point2D::new(coord(sw_lat),
                                  coord(sw_lon));
    let north_east = Point2D::new(coord(ne_lat),
                                  coord(ne_lon));
    Rect2D::new(south_west, north_east)
}

pub fn construct_testing_waypoints() -> Vec<Waypoint<Coord, Moment>> {
    let start_location = Point3D::new(coord("55.395"),
                                    coord("37.385"),
                                    coord("1"));
    let end_location = Point3D::new(coord("55.397"),
                                    coord("37.387"),
                                    coord("1"));
    let start_time = 100_u64; 
    let end_time = 110_u64; 
    let start_wp = Waypoint::new(start_location, start_time);
    let end_wp = Waypoint::new(end_location, end_time);
    vec![start_wp, end_wp]
}

// Assume that alt is const for now.
// TODO (n>2) replace &str in signature to an array [[&str; 4]; n]
pub fn construct_custom_waypoints(start_lat: &str, start_lon: &str,
                                  end_lat: &str, end_lon: &str, 
                                  start_time: u64, end_time: u64) -> Vec<Waypoint<Coord, Moment>> {
    let start_location = Point3D::new(coord(start_lat),
                                    coord(start_lon),
                                    coord("1"));
    let end_location = Point3D::new(coord(end_lat),
                                    coord(end_lon),
                                    coord("1"));
    let start_wp = Waypoint::new(start_location, start_time);
    let end_wp = Waypoint::new(end_location, end_time);
    vec![start_wp, end_wp]
}

#[test]
fn it_try_to_add_root_unauthorized() {
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
fn it_try_to_add_root_by_registrar() {
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
fn it_try_to_add_raw_root_with_exceeded_page_limit() {
      new_test_ext().execute_with(|| {
          assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
          ));
          let raw_coords: [i32; 6] = [
              465587600,
              312529919,
              8388608,
              469815744,
              318558719,
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

#[test]
fn it_try_to_add_too_big_root() {
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
                construct_custom_box("0.0", "0.0", "250.0", "250.0",),
                coord(DELTA),
            ),
            Error::PageLimitExceeded
        );
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_box("0.0", "0.0", "45.1", "50.9"),
                coord(DELTA),
            ),
            Error::PageLimitExceeded
        );
    });
}

#[test]
fn it_try_to_add_root_with_incorrect_coordinates() {
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
fn it_try_to_add_root_as_square_2x2() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));

        let bounding_box = construct_custom_box("0.051", "0.0", "0.5", "0.75");
        let (sw_cell_row_index, sw_cell_column_index) = Page::get_cell_indexes(bounding_box.south_west);
        assert_eq!(sw_cell_row_index, 5);
        assert_eq!(sw_cell_column_index, 0);
        let (ne_cell_row_index, ne_cell_column_index) = Page::get_cell_indexes(bounding_box.north_east);
        assert_eq!(ne_cell_row_index, 50);
        assert_eq!(ne_cell_column_index, 75);

        let amount_of_pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(amount_of_pages_to_extract, 4);

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
fn it_try_to_add_root_as_rectangle_4x1() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));

        let bounding_box = construct_custom_box("0.051", "0.0", "1.271", "0.011");
        let (sw_cell_row_index, sw_cell_column_index) = Page::get_cell_indexes(bounding_box.south_west);
        assert_eq!(sw_cell_row_index, 5);
        assert_eq!(sw_cell_column_index, 0);
        let (ne_cell_row_index, ne_cell_column_index) = Page::get_cell_indexes(bounding_box.north_east);
        assert_eq!(ne_cell_row_index, 127);
        assert_eq!(ne_cell_column_index, 1);

        let amount_of_pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(amount_of_pages_to_extract, 4);

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
fn it_try_to_add_root_as_rectangle_1x4() {
    new_test_ext().execute_with(|| {
        assert_ok!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                REGISTRAR_1_ACCOUNT_ID,
                super::REGISTRAR_ROLE
        ));

        let bounding_box = construct_custom_box("0.0", "0.0", "0.011", "1.751");
        let (sw_cell_row_index, sw_cell_column_index) = Page::get_cell_indexes(bounding_box.south_west);
        assert_eq!(sw_cell_row_index, 0);
        assert_eq!(sw_cell_column_index, 0);
        let (ne_cell_row_index, ne_cell_column_index) = Page::get_cell_indexes(bounding_box.north_east);
        assert_eq!(ne_cell_row_index, 1);
        assert_eq!(ne_cell_column_index, 175);

        let amount_of_pages_to_extract = Page::get_amount_of_pages_to_extract_using_box(bounding_box);
        assert_eq!(amount_of_pages_to_extract, 4);

        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                bounding_box,
                coord(DELTA),
        ));
    });
}

#[test]
fn it_try_to_remove_root() {
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
fn it_try_to_add_zone_unauthorized() {
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
fn it_try_to_add_zone_to_not_existing_root() {
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
fn it_try_to_add_zone_by_registrar() {
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
fn it_try_to_get_zone() {
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
fn it_try_to_remove_zone() {
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
fn it_try_to_add_zone_which_lies_in_different_areas() {
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
fn it_try_to_add_overlapping_zones() {
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
fn it_try_to_add_not_overlapping_zones() {
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
fn it_try_to_add_more_than_max_zones() {
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
fn it_change_not_existing_area_type() {
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
fn it_change_existing_area_type() {
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
fn it_dispatchable_get_root_index() {
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
        // 55.395 - 232343470
        // 37.385 - 156804055
        let root_id = DSMapsModule::get_root_index([232343470, 156804055]);
        assert_eq!(root_id, 1558542996706168526);
        // Proof, that everything is right: by this index we get active root
        let root = DSMapsModule::root_box_data(root_id);
        assert!(root.is_active());
    });
}

// Not sure if there's need to try this unauthorized 
#[test]
fn it_add_route_by_registrar() {
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
        let waypoints = construct_testing_waypoints();
        assert_ok!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints,
                ROOT_ID,
        ));
    });
}

#[test]
fn it_add_route_wrong_root() {
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
        let waypoints = construct_testing_waypoints();
        assert_noop!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints,
                ROOT_ID + 1,
            ),
            Error::RootDoesNotExist
        );
    });
}

#[test]
fn it_add_route_wrong_timestamps() {
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
        // coords are same as in testing wp
        let waypoints = construct_custom_waypoints(
            "55.395", "37.385",
            "55.397", "37.387",
            120, 100);
        assert_noop!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints,
                ROOT_ID,
            ), 
            Error::WrongTimeSupplied
        );
    });
}

#[test]
fn it_add_route_wrong_waypoints() {
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
        let location = Point3D::new(coord("55.395"),
                                coord("37.385"),
                                coord("1"));
        let single_waypoint = vec![Waypoint::new(location, 10)];
        assert_noop!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                single_waypoint,
                ROOT_ID,
            ), 
            Error::InvalidData
        );
            
        let waypoints = construct_custom_waypoints(
            "55.395", "37.385",
            "10", "37",
            100, 120);
        assert_noop!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints,
                ROOT_ID,
            ), 
            Error::RouteDoesNotFitToRoot
        );
    });
}

// There is no zones to block the way, so it'll pass
#[test]
fn it_add_route_without_zones() {
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
        let waypoints = construct_testing_waypoints();
        assert_ok!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints,
                ROOT_ID,
        ));
    });
}

// This test contain two zones, one is intersecting route, and another is not 
#[test]
fn it_add_route_one_area() {
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
        let waypoints = construct_testing_waypoints();
        // This one do not block the way, and test will pass
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_rect("55.391", "37.381", "55.392", "37.382"),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_ok!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints.clone(),
                ROOT_ID,
        ));
        // But this one will fail, as it blocks the way
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_noop!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints,
                ROOT_ID,
            ), Error::RouteIntersectRedZone
        );
    });
}

#[test]
fn it_add_route_multiple_areas() {
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
        // Route starts at 1 area, and ends in testing rect
        let waypoints = construct_custom_waypoints("55.373", "37.373", "55.396", "37.386", 10, 20);
        // This one located far from our route, so no intersection
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_custom_rect("55.411", "37.372", "55.416", "37.375"),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_ok!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints.clone(),
                ROOT_ID,
        ));
        // But this one will
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_noop!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints,
                ROOT_ID,
            ), Error::RouteIntersectRedZone
        );
    });
}

// Here we add zone, and then we remove it from storage
#[test]
fn it_add_route_and_remove_zone() {
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
                coord(DELTA),
            )
        );
        let waypoints = construct_custom_waypoints("55.373", "37.373", "55.396", "37.386", 10, 20);
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
            )
        );
        // Can't add it, zone is blocking the way
        assert_noop!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints.clone(),
                ROOT_ID,
            ), 
            Error::RouteIntersectRedZone
        );
        let zone_index = DSMapsModule::pack_index(ROOT_ID, AREA_ID, 0);
        // Now we remove it, clearing the path
        assert_ok!(
            DSMapsModule::zone_remove(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                zone_index,
            )
        );
        assert_ok!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints,
                ROOT_ID,
            )
        );
    });
}

#[test]
fn it_add_lots_of_zones() {
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
                coord(DELTA),
            )
        );
        let delta: Coord = coord(DELTA);
        let waypoints = construct_custom_waypoints("55.373", "37.373", "55.396", "37.386", 10, 20);
        let mut testing_rect: Rect2D<Coord> = construct_testing_rect();
        for _n in 1..11 {
            assert_ok!(
                DSMapsModule::zone_add(
                    Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                    testing_rect,
                    DEFAULT_HEIGHT, 
                    ROOT_ID,
                )
            );
            testing_rect.north_east.lon += delta;
            testing_rect.south_west.lon += delta;
        }
        // Can't add it, one of this zones is blocking the way
        assert_noop!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints.clone(),
                ROOT_ID,
            ), 
            Error::RouteIntersectRedZone
        );
        let zone_index = DSMapsModule::pack_index(ROOT_ID, AREA_ID, 0);
        // Now we remove it, clearing the path
        assert_ok!(
            DSMapsModule::zone_remove(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                zone_index,
            )
        );
        assert_ok!(
            DSMapsModule::route_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                waypoints,
                ROOT_ID,
            )
        );
    });
}