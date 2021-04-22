use crate::mock::*;
use crate::{Point3D, Box3D, RootBox, 
            Point2D, Rect2D};
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
type Coord = I9F23;

// Constants to make tests more readable
const ADMIN_ACCOUNT_ID: u64 = 1;
const REGISTRAR_1_ACCOUNT_ID: u64 = 2;
const ROOT_ID: u32 = 1;
// this value, and values in construct_testing_..() was calculated
const AREA_ID: u16 = 58;
const DEFAULT_HEIGHT: u16 = 30;

const DELTA: &str = "0.01";

// shortcut for &str -> Coord
fn coord<Coord>(s: &str) -> Coord
    where Coord: FromStr,
         <Coord as FromStr>::Err: std::fmt::Debug { Coord::from_str(s).unwrap() }

fn construct_testing_box() -> Box3D<Coord> {
    let north_west = Point3D::new(coord("55.37"),
                                  coord("37.37"), 
                                  coord("1"));
    let south_east = Point3D::new(coord("55.92"),
                                  coord("37.90"),       
                                  coord("3"));      
    Box3D::new(north_west, south_east)
}

fn construct_custom_box(nw_lat: &str, nw_lon: &str,
                        se_lat: &str, se_lon: &str) -> Box3D<Coord> {
    let north_west = Point3D::new(coord(nw_lat),
                                  coord(nw_lon), 
                                  coord("1"));
    let south_east = Point3D::new(coord(se_lat),
                                  coord(se_lon),       
                                  coord("3"));      
    Box3D::new(north_west, south_east)
}

fn construct_custom_rect(nw_lat: &str, nw_lon: &str,
                         se_lat: &str, se_lon: &str) -> Rect2D<Coord> {
    let north_west = Point2D::new(coord(nw_lat),
                                  coord(nw_lon));
    let south_east = Point2D::new(coord(se_lat),
                                  coord(se_lon));
    Rect2D::new(north_west, south_east)
}

fn construct_testing_rect() -> Rect2D<Coord> {
    let north_west = Point2D::new(coord("55.395"),
                                  coord("37.385"));
    let south_east = Point2D::new(coord("55.396"),
                                  coord("37.386"));
    Rect2D::new(north_west, south_east)
}

// 4 rects from test              
//          +-----o  +-----o
//          |     |  |     |
//          |     |  |     |
//          |   +-+--+-+   |
//          o---+-+  +-+---+
//              |      |
//          +---+-+  +-+---o 
//          |   +-+--+-+   |
//          |     |  |     |
//          |     |  |     |
//          o-----+  +-----+ 

#[test]
fn it_zone_intersection_works() {
    new_test_ext().execute_with(|| {
        // Can't add two same zones
        let mid_rect = construct_custom_rect("1", "1", "3", "3");
        assert!(mid_rect.zone_intersects(mid_rect));
        // Doesn't intersect
        let far_away_rect = construct_custom_rect("10", "10", "30", "30");
        assert!(!mid_rect.zone_intersects(far_away_rect));
        // Checking from 4 sides (as on comment above)
        let down_right_rect = construct_custom_rect("0", "0", "2", "2");
        assert!(mid_rect.zone_intersects(down_right_rect));
        let top_right_rect = construct_custom_rect("0", "2", "2", "4");
        assert!(mid_rect.zone_intersects(top_right_rect));
        let top_left_rect = construct_custom_rect("2", "2", "4", "4");
        assert!(mid_rect.zone_intersects(top_left_rect));
        let down_right_rect = construct_custom_rect("2", "0", "4", "2");
        assert!(mid_rect.zone_intersects(down_right_rect));
        // We can place zone riiiight near another, but not on it 
        let rect_near_edge = construct_custom_rect("0", "0", "0.999999", "5");
        assert!(!mid_rect.zone_intersects(rect_near_edge));
        let rect_on_edge = construct_custom_rect("0", "0", "1", "5");
        assert!(mid_rect.zone_intersects(rect_on_edge));
    });
}

#[test]
fn it_max_area_counts_right() {
    new_test_ext().execute_with(|| {
        let bbox = construct_custom_box("0", "0", "2", "3");
        let root = RootBox::new(ROOT_ID, bbox, I9F23::from_num(1));
        assert_eq!(root.get_max_area(), 6); 
        // This values will not be possible in extinsics, btw
        let bbox = construct_custom_box("-90", "-180", "0", "0");
        let root = RootBox::new(ROOT_ID, bbox, I9F23::from_num(1));
        assert_eq!(root.get_max_area(), 16_200); 
    });
}

#[test]
fn it_detects_right_area() {
    new_test_ext().execute_with(|| {
        let bbox = construct_custom_box("0", "0", "2", "3");
        let root = RootBox::new(100, bbox, I9F23::from_num(1));
        let touch = Point2D::new(coord("0.5"),
                                coord("0.5"));
        assert_eq!(root.detect_intersected_area(touch), 1);
        let touch = Point2D::new(coord("1.5"),
                                coord("1.5"));
        assert_eq!(root.detect_intersected_area(touch), 4); 
        // not sure, is it better to throw an error, or have a special index?
        let far_away_touch = Point2D::new(coord("50"),
                                        coord("50"));
        assert_eq!(root.detect_intersected_area(far_away_touch), 0); 
    });
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
