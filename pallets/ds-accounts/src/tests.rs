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
// Should be changed later
const UAV_1_ACCOUNT_ID: u64 = 4294967295 + 1;   //u32::MAX + 1

#[test]
fn it_default_pallet_transaction_payment_multiplier() {
    use frame_support::sp_runtime::{traits::Convert, FixedPointNumber, FixedU128};

    type Multiplier = FixedU128;
    let multiplier: Multiplier = Multiplier::saturating_from_integer(1);

    let new_multiplier = <() as Convert<Multiplier, Multiplier>>::convert(multiplier);

    assert_eq!(
        new_multiplier, 
        0.into(),
        "default multiplier convert from {:} to {:}",
        multiplier, new_multiplier
    );
}

#[test]
#[cfg(feature = "payment")]
fn it_identity_pallet_transaction_payment_multiplier() {
    use frame_support::sp_runtime::{traits::Convert, FixedPointNumber, FixedU128};

    type Multiplier = FixedU128;
    let multiplier: Multiplier = Multiplier::saturating_from_integer(1);

    let new_multiplier = <crate::payment::IdentityMultiplierUpdater as Convert<
        Multiplier,
        Multiplier,
    >>::convert(multiplier);

    assert_eq!(
        new_multiplier, 
        multiplier,
        "identity multiplier convert from {:} to {:}",
        multiplier, new_multiplier
    );
}

#[test]
fn it_create_new_account() {
    new_test_ext().execute_with(|| {
        let account = DSAccountsModule::account_registry(2);
        assert!(!account.is_enabled());

        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();
        Timestamp::set_timestamp(5000);
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash,
        ));

        let account = DSAccountsModule::account_registry(REGISTRAR_1_ACCOUNT_ID);
        assert!(account.is_enabled());

        let age = account.age(20000);
        assert_eq!(age, 15000);
    });
}

#[test]
fn it_disable_account() {
    new_test_ext().execute_with(|| {
        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash,
        ));
        assert_ok!(DSAccountsModule::account_disable(
                Origin::signed(ADMIN_ACCOUNT_ID), 
                REGISTRAR_1_ACCOUNT_ID
        ));
        assert!(!DSAccountsModule::account_registry(REGISTRAR_1_ACCOUNT_ID).is_enabled());
    });
}

#[test]
fn it_try_disable_themself() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            DSAccountsModule::account_disable(
                Origin::signed(ADMIN_ACCOUNT_ID),
                ADMIN_ACCOUNT_ID
            ),
            Error::InvalidAction
        );
        assert!(DSAccountsModule::account_registry(ADMIN_ACCOUNT_ID).is_enabled());
    });
}

#[test]
fn it_try_create_account_with_role_pilot() {
    new_test_ext().execute_with(|| {
        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();

        assert_noop!(
            DSAccountsModule::account_add(
                Origin::signed(ADMIN_ACCOUNT_ID),
                PILOT_1_ACCOUNT_ID,
                super::PILOT_ROLE,
                account_metadata_ipfs_hash,
            ),
            Error::NotAllowedRole
        );
    });
}

#[test]
fn it_try_create_by_registrar() {
    new_test_ext().execute_with(|| {
        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();

        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash.to_owned(),
        ));

        assert_noop!(
            DSAccountsModule::account_add(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID), 
                REGISTRAR_2_ACCOUNT_ID, 
                super::REGISTRAR_ROLE,
                account_metadata_ipfs_hash,
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_register_pilot_by_registrar() {
    new_test_ext().execute_with(|| {
        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();
        Timestamp::set_timestamp(5000);

        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash.to_owned(),
        ));

        assert_ok!(DSAccountsModule::register_pilot(
            Origin::signed(REGISTRAR_1_ACCOUNT_ID), 
            PILOT_1_ACCOUNT_ID,
            account_metadata_ipfs_hash,
        ));

        let account = DSAccountsModule::account_registry(PILOT_1_ACCOUNT_ID);
        assert!(account.is_enabled());

        let age = account.age(20000);
        assert_eq!(age, 15000);
    });
}

#[test]
fn it_try_register_pilot_not_by_registrar() {
    new_test_ext().execute_with(|| {
        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();
        assert_noop!(
            DSAccountsModule::register_pilot(
                Origin::signed(ADMIN_ACCOUNT_ID), 
                PILOT_1_ACCOUNT_ID,
                account_metadata_ipfs_hash,
            ),
            Error::NotAuthorized,
        );
    });
}

#[test]
fn it_try_register_same_pilot_twice() {
    new_test_ext().execute_with(|| {
        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();
        Timestamp::set_timestamp(5000);
 
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash.to_owned(),
        ));
 
        assert_ok!(DSAccountsModule::register_pilot(
            Origin::signed(REGISTRAR_1_ACCOUNT_ID),
            PILOT_1_ACCOUNT_ID,
            account_metadata_ipfs_hash.to_owned(),
        ));
 
        let account = DSAccountsModule::account_registry(PILOT_1_ACCOUNT_ID);
        assert!(account.is_enabled());
 
        let age = account.age(20000);
        assert_eq!(age, 15000);

        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_2_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash.to_owned(),
        ));
 
        assert_noop!(
            DSAccountsModule::register_pilot(
                Origin::signed(REGISTRAR_2_ACCOUNT_ID),
                PILOT_1_ACCOUNT_ID,
                account_metadata_ipfs_hash,
            ),
            Error::AlreadyRegistered,
        );
    });
}

#[test]
fn it_try_to_add_admin_account_role_pilot() {
    new_test_ext().execute_with(|| {
        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash.to_owned(),
        ));

        assert_ok!(
            DSAccountsModule::register_pilot(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID), 
                ADMIN_ACCOUNT_ID, 
                account_metadata_ipfs_hash,
            )
        );
    });
}

#[test]
fn it_account_reaped() {
    new_test_ext().execute_with(|| {
        assert_ok!(Balances::transfer(
                Origin::signed(ADMIN_ACCOUNT_ID), 
                REGISTRAR_1_ACCOUNT_ID, 
                10000)
        );

        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash,
        ));

        assert!(DSAccountsModule::account_registry(REGISTRAR_1_ACCOUNT_ID).is_enabled());
        assert_ok!(Balances::transfer(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID), 
                3, 
                10000
        ));
        assert!(!DSAccountsModule::account_registry(REGISTRAR_1_ACCOUNT_ID).is_enabled());
    });
}

#[test]
fn it_balance() {
    new_test_ext().execute_with(|| {
        //total_issuance
        assert_eq!(Balances::total_issuance(), 100000);
        assert_eq!(Balances::free_balance(ADMIN_ACCOUNT_ID), 100000);
        assert_eq!(Balances::free_balance(REGISTRAR_1_ACCOUNT_ID), 0);

        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();

        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(1),
            3,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash,
        ));

        assert_ok!(Balances::transfer(
                Origin::signed(ADMIN_ACCOUNT_ID), 
                REGISTRAR_1_ACCOUNT_ID, 
                50000)
        );

        assert_eq!(Balances::free_balance(ADMIN_ACCOUNT_ID), 50000);
        assert_eq!(Balances::free_balance(REGISTRAR_1_ACCOUNT_ID), 50000);
        assert_eq!(Balances::total_issuance(), 100000);
        assert_ok!(Balances::transfer(
                Origin::signed(ADMIN_ACCOUNT_ID), 
                REGISTRAR_1_ACCOUNT_ID, 
                49990
        ));
        // Account 1 has been removed from balances and dust remaining 10
        assert_eq!(Balances::total_issuance(), 99990);
    });
}
#[test]
fn it_try_to_add_new_uav_with_allowed_roles() {
    new_test_ext().execute_with(|| {
        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();
        let uav_metadata_ipfs_hash: Vec<u8> = vec![1, 2, 3, 4];
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash.to_owned(),
        ));

        assert_ok!(DSAccountsModule::register_pilot(
            Origin::signed(REGISTRAR_1_ACCOUNT_ID),
            PILOT_1_ACCOUNT_ID,
            account_metadata_ipfs_hash,
        ));

        assert_ok!(DSAccountsModule::register_uav(
            Origin::signed(PILOT_1_ACCOUNT_ID),
            b"1234-IDG-AF".to_vec(),
            uav_metadata_ipfs_hash.to_owned(),
            UAV_1_ACCOUNT_ID,
        ));
        assert_ok!(DSAccountsModule::register_uav(
            Origin::signed(REGISTRAR_1_ACCOUNT_ID),
            b"1234-IDG-AF".to_vec(),
            uav_metadata_ipfs_hash,
            UAV_1_ACCOUNT_ID,
        ));
    });
}

#[test]
fn it_try_register_uav_not_by_allowed_users() {
    new_test_ext().execute_with(|| {
        let uav_metadata_ipfs_hash: Vec<u8> = vec![1, 2, 3, 4];

        assert_noop!(
            DSAccountsModule::register_uav(
                Origin::signed(ADMIN_ACCOUNT_ID),
                b"1234-IDG-AF".to_vec(),
                uav_metadata_ipfs_hash,
                UAV_1_ACCOUNT_ID,
            ),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_try_register_uav_on_wrong_addr() {
    new_test_ext().execute_with(|| {
        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();
        let uav_metadata_ipfs_hash: Vec<u8> = vec![1, 2, 3, 4];
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash.to_owned(),
        ));
        assert_ok!(DSAccountsModule::register_pilot(
            Origin::signed(REGISTRAR_1_ACCOUNT_ID),
            PILOT_1_ACCOUNT_ID,
            account_metadata_ipfs_hash,
        ));
        assert_noop!(
            DSAccountsModule::register_uav(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                b"1234-IDG-AF".to_vec(),
                uav_metadata_ipfs_hash.to_owned(),
                PILOT_1_ACCOUNT_ID,            
            ),
            Error::AddressAlreadyUsed
        );
        assert_noop!(
            DSAccountsModule::register_uav(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                b"1234-IDG-AF".to_vec(),
                uav_metadata_ipfs_hash,
                REGISTRAR_1_ACCOUNT_ID,            
            ),
            Error::AddressAlreadyUsed
        );
    });
}
#[test]
fn it_try_register_user_on_uav_addr() {
    new_test_ext().execute_with(|| {
        // Just a random IPFS hash
        let account_metadata_ipfs_hash: Vec<u8> = "QmT78zSuBmuS4z925WZfrqQ1qHaJ56DQaTfyMUF7F8ff5o".as_bytes().to_vec();
        let uav_metadata_ipfs_hash: Vec<u8> = vec![1, 2, 3, 4];
        assert_ok!(DSAccountsModule::account_add(
            Origin::signed(ADMIN_ACCOUNT_ID),
            REGISTRAR_1_ACCOUNT_ID,
            super::REGISTRAR_ROLE,
            account_metadata_ipfs_hash.to_owned(),
        ));

        assert_ok!(DSAccountsModule::register_uav(
            Origin::signed(REGISTRAR_1_ACCOUNT_ID),
            b"1234-IDG-AF".to_vec(),
            uav_metadata_ipfs_hash,
            UAV_1_ACCOUNT_ID,            
        ));

        assert_noop!(
            DSAccountsModule::register_pilot(
                Origin::signed(REGISTRAR_1_ACCOUNT_ID),
                UAV_1_ACCOUNT_ID,
                account_metadata_ipfs_hash,
            ), 
            Error::AddressAlreadyUsed
        );
    });
}
