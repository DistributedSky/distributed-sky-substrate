use crate::mock::*;
use crate::{Point3D, Box3D, 
            Point2D, Rect2D};
use frame_support::{
    assert_noop, assert_ok,
};
use substrate_fixed::types::I9F23;
use sp_std::str::FromStr;

type Error = super::Error<Test>;
type Coord = I9F23;

// Constants to make tests more readable
const ADMIN_ACCOUNT_ID: u64 = 1;
const REGISTRAR_1_ACCOUNT_ID: u64 = 2;
const ROOT_ID: u32 = 0;
// this value, and values in construct() was calculated
const AREA_ID: u16 = 58;
const DEFAULT_HEIGHT: u16 = 30;

const DELTA: &str = "0.01";
// fn coord<Coord>(s: &str) -> Coord { Coord::from_str(s).unwrap() }
fn construct_box() -> Box3D<Coord> {
    let north_west: Point3D<Coord> = Point3D::new(Coord::from_str("55.37").unwrap(),
                                                  Coord::from_str("37.37").unwrap(), 
                                                  Coord::from_str("1").unwrap());
    let south_east: Point3D<Coord> = Point3D::new(Coord::from_str("55.92").unwrap(),
                                                  Coord::from_str("37.90").unwrap(),       
                                                  Coord::from_str("3").unwrap());      
    Box3D::new(north_west, south_east)
}

fn construct_huge_box() -> Box3D<Coord> {
    let north_west: Point3D<Coord> = Point3D::new(Coord::from_str("55.37").unwrap(),
                                                  Coord::from_str("37.37").unwrap(), 
                                                  Coord::from_str("1").unwrap());
    let south_east: Point3D<Coord> = Point3D::new(Coord::from_str("66.92").unwrap(),
                                                  Coord::from_str("37.90").unwrap(),       
                                                  Coord::from_str("3").unwrap());      
    Box3D::new(north_west, south_east)
}

fn construct_rect() -> Rect2D<Coord> {
    let north_west: Point2D<Coord> = Point2D::new(Coord::from_str("55.395").unwrap(),
                                                  Coord::from_str("37.385").unwrap());
    let south_east: Point2D<Coord> = Point2D::new(Coord::from_str("55.396").unwrap(),
                                                  Coord::from_str("37.386").unwrap());
    Rect2D::new(north_west, south_east)
}

#[test]
fn it_try_add_root_unauthorized() {
    new_test_ext().execute_with(|| {
        let account = DSAccountsModule::account_registry(2);
        assert!(!account.is_enabled());

        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_box(),
                Coord::from_str(DELTA).unwrap()
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_try_add_root_by_registrar() {
    new_test_ext().execute_with(|| {
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_box(),
                Coord::from_str(DELTA).unwrap()
        ));
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_box(),
                Coord::from_str(DELTA).unwrap()
            ),
            Error::NotAuthorized
        );
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
                construct_rect(),
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

        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_rect(),
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
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_box(),
                Coord::from_str(DELTA).unwrap()
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_increment_zone_counter_in_area() {    
    new_test_ext().execute_with(|| {
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_box(),
                Coord::from_str(DELTA).unwrap()
        ));
        let area = DSMapsModule::area_info(ROOT_ID, AREA_ID);
        assert!(area.child_count == 0);
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        let area = DSMapsModule::area_info(ROOT_ID, AREA_ID);
        assert!(area.child_count == 1);
    });
}

#[test]
fn it_changes_not_existing_area_type() {    
    new_test_ext().execute_with(|| {
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_box(),
                Coord::from_str(DELTA).unwrap()
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
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_huge_box(),
                Coord::from_str(DELTA).unwrap()
            ), 
            Error::BadDimesions
        );
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_box(),
                Coord::from_num(1)
            ), 
            Error::InvalidData
        );
    });
}

#[test]
fn it_changes_existing_area_type() {    
    new_test_ext().execute_with(|| {
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_box(),
                Coord::from_str(DELTA).unwrap()
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_rect(),
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
                construct_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
                ), 
            Error::ForbiddenArea
        );
    });
}
