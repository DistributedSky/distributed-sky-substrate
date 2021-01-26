use crate::mock::*;
use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchInfo, DispatchResultWithPostInfo, GetDispatchInfo},
    traits::UnfilteredDispatchable,
    weights::{Pays, Weight},
};

// Learn more about testing substrate runtime modules
// https://substrate.dev/docs/en/knowledgebase/runtime/tests
// type Module = super::Module<Test>;
type Timestamp = pallet_timestamp::Module<Test>;
type System = frame_system::Module<Test>;
type Balances = pallet_balances::Module<Test>;
type Error = super::Error<Test>;

// get last events and reset
fn events() -> Vec<TestEvent> {
    let evt = System::events()
        .into_iter()
        .map(|evt| evt.event)
        .collect::<Vec<_>>();
    System::reset_events();
    evt
}
#[allow(dead_code)]
fn last_event() -> TestEvent {
    System::events().pop().expect("Event expected").event
}

#[test]
fn it_default_pallet_transaction_payment_multiplier() {
    use frame_support::sp_runtime::{traits::Convert, FixedPointNumber, FixedU128};

    type Multiplier = FixedU128;
    let multiplier: Multiplier = Multiplier::saturating_from_integer(1);

    let new_multiplier = <() as Convert<Multiplier, Multiplier>>::convert(multiplier);
    println!(
        "default multiplier convert from {:} to {:}",
        multiplier, new_multiplier
    );

    assert_eq!(new_multiplier, 0.into());
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
    println!(
        "identiry multiplier convert from {:} to {:}",
        multiplier, new_multiplier
    );

    assert_eq!(new_multiplier, multiplier);
}

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        System::set_block_number(2);
        // Dispatch a signed extrinsic.
        assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
        assert_eq!(
            events(),
            vec![TestEvent::template(super::RawEvent::SomethingStored(42, 1))]
        );

        // Read pallet storage and assert an expected result.
        assert_eq!(TemplateModule::something(), Some(42));
        System::set_block_number(3);
        assert_eq!(events(), vec![]);
    });
}

#[test]
fn it_create_new_account() {
    new_test_ext().execute_with(|| {
        let account = TemplateModule::account_registry(2);
        assert!(!account.is_enable());

        Timestamp::set_timestamp(5000);
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            2,
            super::USER_ROLE
        ));

        let account = TemplateModule::account_registry(2);
        assert!(account.is_enable());

        let age = account.age(20000);
        assert_eq!(age, 15000);

        assert_ok!(TemplateModule::do_something(Origin::signed(2), 20));
        System::set_block_number(2);
        assert_eq!(TemplateModule::something(), Some(20));
    });
}

#[test]
fn it_disable_account() {
    new_test_ext().execute_with(|| {
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            2,
            super::USER_ROLE
        ));
        assert_ok!(TemplateModule::account_disable(Origin::signed(1), 2));
        assert!(!TemplateModule::account_registry(2).is_enable());
    });
}

#[test]
fn it_try_disable_themself() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            TemplateModule::account_disable(Origin::signed(1), 1),
            Error::InvalidAction
        );
        assert!(TemplateModule::account_registry(1).is_enable());
    });
}

#[test]
fn it_try_create_by_user() {
    new_test_ext().execute_with(|| {
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            2,
            super::USER_ROLE
        ));
        assert_noop!(
            TemplateModule::account_add(Origin::signed(2), 3, super::USER_ROLE),
            Error::NotAuthorized
        );
    });
}

#[test]
fn it_account_reaped() {
    new_test_ext().execute_with(|| {
        assert_ok!(Balances::transfer(Origin::signed(1), 2, 10000));
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            2,
            super::USER_ROLE
        ));

        assert!(TemplateModule::account_registry(2).is_enable());
        assert_ok!(Balances::transfer(Origin::signed(2), 3, 10000));
        assert!(!TemplateModule::account_registry(2).is_enable());
    });
}

#[test]
fn it_lock_balance() {
    new_test_ext().execute_with(|| {
        assert_ok!(TemplateModule::account_transfer_and_lock(
            Origin::signed(1),
            2,
            10000
        ));
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            2,
            super::USER_ROLE
        ));

        assert!(TemplateModule::account_registry(2).is_enable());
        assert_eq!(Balances::free_balance(2), 10000);
        assert_noop!(
            Balances::transfer(Origin::signed(2), 3, 5000),
            pallet_balances::Error::<Test, _>::LiquidityRestrictions
        );
    });
}

#[test]
fn it_balance() {
    new_test_ext().execute_with(|| {
        //total_issuance
        assert_eq!(Balances::total_issuance(), 100000);
        assert_eq!(Balances::free_balance(1), 100000);
        assert_eq!(Balances::free_balance(2), 0);
        assert_ok!(TemplateModule::account_add(
            Origin::signed(1),
            3,
            super::USER_ROLE
        ));

        assert_ok!(Balances::transfer(Origin::signed(1), 2, 50000));

        assert_eq!(Balances::free_balance(1), 50000);
        assert_eq!(Balances::free_balance(2), 50000);
        assert_eq!(Balances::total_issuance(), 100000);
        assert_ok!(Balances::transfer(Origin::signed(1), 2, 49990));
        // Account 1 has been removed from balances and dust remaining 10
        assert_eq!(Balances::total_issuance(), 99990);
    });
}

#[test]
fn it_dispatchable_weight() {
    // pre-dispatch weights
    fn assert_dispatch(call: crate::Call<Test>, weight: Weight, pay: Pays) -> DispatchInfo {
        let dispatch_info = call.get_dispatch_info();

        assert_eq!(dispatch_info.weight, weight);
        assert_eq!(dispatch_info.pays_fee, pay);
        dispatch_info
    }
    let call = crate::Call::<Test>::account_add(3, 1);
    assert_dispatch(call, <() as super::WeightInfo>::account_add(), Pays::Yes);

    let call = crate::Call::<Test>::update_something(3);
    assert_dispatch(
        call,
        <() as super::WeightInfo>::update_something(),
        Pays::Yes,
    );

    // post-dispatch weights
    fn assert_call(
        origin: Origin,
        call: crate::Call<Test>,
        weight: Weight,
        pay: Pays,
    ) -> DispatchResultWithPostInfo {
        let dispatch_info = call.get_dispatch_info();
        let dispatch_post_info = call.dispatch_bypass_filter(origin)?;

        assert_eq!(dispatch_post_info.pays_fee(&dispatch_info), pay);
        assert_eq!(
            dispatch_post_info.calc_actual_weight(&dispatch_info),
            weight
        );
        Ok(dispatch_post_info)
    }

    new_test_ext().execute_with(|| {
        let call = crate::Call::<Test>::update_something(300);
        assert_ok!(assert_call(
            Origin::signed(1),
            call,
            <() as super::WeightInfo>::update_something(),
            Pays::Yes
        ));

        let call = crate::Call::<Test>::update_something(100);
        assert_ok!(assert_call(
            Origin::signed(1),
            call,
            <() as super::WeightInfo>::update_something(),
            Pays::No
        ));

        let call = crate::Call::<Test>::account_add(3, 1);
        assert_ok!(assert_call(
            Origin::signed(1),
            call,
            <() as super::WeightInfo>::account_add(),
            Pays::Yes
        ));

        let call = crate::Call::<Test>::account_disable(3);
        assert_ok!(assert_call(
            Origin::signed(1),
            call,
            <() as super::WeightInfo>::account_disable(),
            Pays::Yes
        ));

        let call = crate::Call::<Test>::update_something(300);
        assert_noop!(
            assert_call(Origin::signed(2), call, 0, Pays::Yes),
            Error::NotAuthorized
        );
    });
}
