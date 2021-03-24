use crate::mock::*;
use crate::{Point3D, Box3D, 
            Point2D, Rect2D, };
use frame_support::{
    assert_noop, assert_ok,
};

type Error = super::Error<Test>;
type Coord = u32;

// Constants to make tests more readable
const ADMIN_ACCOUNT_ID: u64 = 1;
const REGISTRAR_1_ACCOUNT_ID: u64 = 2;
const ROOT_ID: u32 = 0;
// this value, and values in construct() was calculated
const AREA_ID: u16 = 36;
const DEFAULT_HEIGHT: u16 = 30;
const DELTA: u32 = 20;

fn construct_box() -> Box3D<Point3D<Coord>> {
    let north_west: Point3D<Coord> = Point3D::new(123, 456, 30);
    let south_east: Point3D<Coord> = Point3D::new(789, 1011, 600);
    Box3D::new(north_west, south_east)
}
fn construct_rect() -> Rect2D<Point2D<Coord>> {
    let north_west: Point2D<Coord> = Point2D::new(170, 480);
    let south_east: Point2D<Coord> = Point2D::new(180, 485);
    Rect2D::new(north_west, south_east)
}

#[test]
fn it_try_add_root_unauthorized() {
    new_test_ext().execute_with(|| {
        let account = DSAccountsModule::account_registry(2);
        assert!(!account.is_enabled());

        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_box(),
                DELTA
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
                DELTA
        ));
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_box(),
                DELTA
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

        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
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
                DELTA
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
                DELTA
        ));
        let area = DSMapsModule::area_info(ROOT_ID, 35);
        assert!(area.child_amount == 0);
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        let area = DSMapsModule::area_info(ROOT_ID, 36);
        assert!(area.child_amount == 1);
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
                DELTA
        ));
        assert_noop!(
            DSMapsModule::change_area_role(
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
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));
        assert_ok!(
            DSMapsModule::root_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_box(),
                DELTA
        ));
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
        ));
        assert_ok!(
            DSMapsModule::change_area_role(
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
