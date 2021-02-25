use crate::mock::*;
use frame_support::{
    assert_noop, assert_ok,
};

// Learn more about testing substrate runtime modules
// https://substrate.dev/docs/en/knowledgebase/runtime/tests
// type Module = super::Module<Test>;
type Timestamp = pallet_timestamp::Module<Test>;
type Balances = pallet_balances::Module<Test>;
type Error = super::Error<Test>;

// Constants to make tests more readable
const ADMIN_ACCOUNT_ID: u64 = 1;
const REGISTRAR_1_ACCOUNT_ID: u64 = 2;
const REGISTRAR_2_ACCOUNT_ID: u64 = 3;
const PILOT_1_ACCOUNT_ID: u64 = 4;
const UAV_1_ACCOUNT_ID: u64 = 4294967295 + 1;   //u32::MAX + 1
//in u32 we can fit global coord w 6 numbers after comma
const POINT_COORDINATES: [u32; 6] = [12, 23, 34, 45, 56, 67];   
#[test]
fn it_try_disable_themself() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            DSMapsModule::zone_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                ZoneType::Green,
                POINT_COORDINATES,
            ),
            Error::InvalidAction
        );
        assert!(DSMapsModule::account_registry(ADMIN_ACCOUNT_ID).is_enabled());
    });
}