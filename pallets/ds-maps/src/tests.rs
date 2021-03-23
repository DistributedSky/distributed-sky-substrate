use crate::mock::*;
use crate::{Point3D, Box3D, RootBox, 
            Point2D, Rect2D, Zone, 
            Area, GREEN_AREA};
use frame_support::{
    assert_noop, assert_ok,
};

type Error = super::Error<Test>;
type Coord = u32;
type LocalCoord = u16;
type RootId = u32;
type AreaId = u16;

// Constants to make tests more readable
const ADMIN_ACCOUNT_ID: u64 = 1;
const REGISTRAR_1_ACCOUNT_ID: u64 = 2;
const ROOT_ID: u32 = 1;
const AREA_ID: u16 = 1;
const DEFAULT_HEIGHT: u16 = 30;

fn construct_box() -> Box3D<Point3D<Coord>> {
    let point_1: Point3D<Coord> = Point3D::new(10, 20, 30);
    let point_2: Point3D<Coord> = Point3D::new(40, 25, 60);
    Box3D::new(point_1, point_2)
}
fn construct_rect() -> Rect2D<Point2D<Coord>> {
    let point_1: Point2D<Coord> = Point2D::new(10, 20);
    let point_2: Point2D<Coord> = Point2D::new(40, 25);
    Rect2D::new(point_1, point_2)
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
                234
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
                234
        ));
        assert_noop!(
            DSMapsModule::root_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_box(),
                234
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
                AREA_ID
            ),
            Error::NotAuthorized
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
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
                AREA_ID
        ));
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                construct_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
                AREA_ID
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
        let area = DSMapsModule::area_info(ROOT_ID, AREA_ID);
        assert!(area.child_amount == 0);
        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
                AREA_ID
        ));
        let area = DSMapsModule::area_info(ROOT_ID, AREA_ID);
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
        assert_noop!(
            DSMapsModule::change_area_role(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID), 
                ROOT_ID,
                AREA_ID,
                0
            )
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
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                construct_rect(),
                DEFAULT_HEIGHT, 
                ROOT_ID,
                AREA_ID
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
                AREA_ID
                ), 
            Error::ForbiddenArea
        );

    });
}