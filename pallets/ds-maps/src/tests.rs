use crate::mock::*;
use crate::{Point3D, Box3D, RootBox, 
            Point2D, Rect2D, Zone, };
use frame_support::{
    assert_noop, assert_ok,
};

type Error = super::Error<Test>;
type Coord = u32;
type LocalCoord = u16;
type RootId = u32;
// Constants to make tests more readable
const ADMIN_ACCOUNT_ID: u64 = 1;
const REGISTRAR_1_ACCOUNT_ID: u64 = 2;

fn construct_box() -> Box3D<Point3D<Coord>> {
    let point_1: Point3D<Coord> = Point3D::new(10, 20, 30);
    let point_2: Point3D<Coord> = Point3D::new(40, 25, 60);
    Box3D::new(point_1, point_2)
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

