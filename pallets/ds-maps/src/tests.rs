use crate::mock::*;

use frame_support::{
    assert_noop, assert_ok,
};

// Learn more about testing substrate runtime modules
// https://substrate.dev/docs/en/knowledgebase/runtime/tests
// type Module = super::Module<Test>;
type Error = super::Error<Test>;

// Constants to make tests more readable
const ADMIN_ACCOUNT_ID: u64 = 1;
const REGISTRAR_1_ACCOUNT_ID: u64 = 2;
//in u32 we can fit global coord w 6 numbers after comma
const POINT_COORDINATES: [u32; 6] = [12, 23, 34, 45, 56, 67];   

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
                ZoneType::Green,
                POINT_COORDINATES,
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
                ZoneType::Green,
                POINT_COORDINATES,
            ));

        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                ZoneType::Green,
                POINT_COORDINATES,
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_try_add_different_zone_types() {
    new_test_ext().execute_with(|| {

        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE
        ));

        assert_ok!(
            DSMapsModule::zone_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                ZoneType::Green,
                POINT_COORDINATES,
            ));

            assert_ok!(
                DSMapsModule::zone_add(
                    Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                    ZoneType::Red,
                    POINT_COORDINATES,
            ));
    });
}
