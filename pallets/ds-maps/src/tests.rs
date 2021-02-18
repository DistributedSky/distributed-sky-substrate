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
//should be changed later
const UAV_1_ACCOUNT_ID: u64 = 4294967295 + 1;   //u32::MAX + 1
