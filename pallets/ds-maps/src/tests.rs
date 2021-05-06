use crate::mock::*;
use crate::{Point3D, Box3D, 
            Point2D, Rect2D,
            Page
};
use frame_support::{
    assert_noop, assert_ok,
};
use substrate_fixed::types::I9F23;
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
pub type Coord = I9F23;

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

pub fn construct_custom_box(nw_lat: &str, nw_lon: &str, se_lat: &str, se_lon: &str) -> Box3D<Coord> {
    let north_east = Point3D::new(coord(nw_lat),
                                  coord(nw_lon), 
                                  coord("1"));
    let south_west = Point3D::new(coord(se_lat),
                                  coord(se_lon),       
                                  coord("3"));      
    Box3D::new(north_east, south_west)
}

pub fn construct_custom_rect(nw_lat: &str, nw_lon: &str, se_lat: &str, se_lon: &str) -> Rect2D<Coord> {
    let north_east = Point2D::new(coord(nw_lat),
                                  coord(nw_lon));
    let south_west = Point2D::new(coord(se_lat),
                                  coord(se_lon));
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
fn it_try_add_root_unauthorized() {
    new_test_ext().execute_with(|| {
        let account = DSAccountsModule::account_registry(2);
        assert!(!account.is_enabled());

        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_testing_box(),
                coord(DELTA)
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_try_add_root_by_registrar() {
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
                coord(DELTA)
        ));
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_testing_box(),
                coord(DELTA)
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_try_remove_root() {    
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
                coord(DELTA)
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
fn it_try_add_zone_unauthorized() {
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
fn it_try_add_zone_to_not_existing_root() {
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
fn it_try_add_zone_by_registrar() {
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
                coord(DELTA)
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
fn it_try_get_zone() {    
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
                coord(DELTA)
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
fn it_try_remove_zone() {    
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
                coord(DELTA)
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
fn it_try_add_zone_which_lies_in_different_areas() {
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
                coord(DELTA)
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
fn it_try_add_overlapping_zones() {
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
                coord(DELTA)
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
fn it_try_add_not_overlapping_zones() {
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
                coord(DELTA)
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
fn it_try_add_more_than_max_zones() {
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
                coord(DELTA)
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
                coord(DELTA)
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
                coord(DELTA)
            ), 
            Error::BadDimesions
        );
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_testing_box(),
                Coord::from_num(1)
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
                coord(DELTA)
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
    let point: Point3D<Coord> = Point3D::new(coord("55.37"), coord("33.37"), coord("1"));
    let page: Page<Coord> = Page::new();
    let (cell_row_index, cell_column_index) = page.get_cell_indexes(point);
    assert_eq!(cell_row_index, 5537);
    assert_eq!(cell_column_index, 3337);

    let point: Point3D<Coord> = Point3D::new(coord("13.37"), coord("255.37"), coord("1"));
    let page: Page<Coord> = Page::new();
    let (cell_row_index, cell_column_index) = page.get_cell_indexes(point);
    assert_eq!(cell_row_index, 1337);
    assert_eq!(cell_column_index, 25537);

    let point: Point3D<Coord> = Point3D::new(coord("1.0"), coord("2.0"), coord("1"));
    let page: Page<Coord> = Page::new();
    let (cell_row_index, cell_column_index) = page.get_cell_indexes(point);
    assert_eq!(cell_row_index, 10);
    assert_eq!(cell_column_index, 20);
}
