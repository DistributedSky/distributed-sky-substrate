#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use frame_support::{
    codec::{Decode, Encode},
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    sp_runtime::{
        sp_std::ops::BitAnd,
        traits::{AtLeast32Bit, Zero},
    },
    traits::{
        Currency, ExistenceRequirement::AllowDeath, Get, LockIdentifier, LockableCurrency,
        OnKilledAccount, WithdrawReason, WithdrawReasons,
    },
    weights::{DispatchClass, Pays, Weight},
    Parameter,
};
use frame_system::ensure_signed;

mod default_weight;
#[cfg(test)]
mod mock;
#[cfg(feature = "payment")]
mod payment;
#[cfg(test)]
mod tests;

pub mod prelude {
    #[cfg(feature = "payment")]
    pub use crate::payment::IdentityMultiplierUpdater;
}

/// Structure, specific for each role
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default)]
pub struct Account<Moment, AccountRole> {
    pub roles: AccountRole,
    pub create_time: Moment,
}

impl<
        Moment: Default + AtLeast32Bit + Copy,
        AccountRole: Zero + Copy + From<u8> + BitAnd<Output = AccountRole>,
    > Account<Moment, AccountRole>
{
    pub fn is_admin(&self) -> bool {
        !(self.roles & ADMIN_ROLE.into()).is_zero()
    }

    pub fn is_registrar(&self) -> bool {
        !(self.roles & PILOT_ROLE.into()).is_zero()
    }

    pub fn is_enable(&self) -> bool {
        !self.roles.is_zero()
    }

    pub fn is_role_correct(_role: AccountRole) -> bool {
        true
    }

    #[allow(dead_code)]
    pub fn age(&self, now: Moment) -> Moment {
        now - self.create_time
    }

    pub fn new_admin() -> Self {
        Account {
            roles: ADMIN_ROLE.into(),
            create_time: Default::default(),
        }
    }
}

pub type AccountOf<T> = Account<<T as pallet_timestamp::Trait>::Moment, <T as Trait>::AccountRole>;
const FEE_LOCK_ID: LockIdentifier = *b"fee lock";

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait + pallet_timestamp::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    // Describe pallet constants.
    // Lean more https://substrate.dev/docs/en/knowledgebase/runtime/metadata
    type AdminRole: Get<Self::AccountRole>;
    /// Change  to u16 if it requires more than 8 distinct values
    type AccountRole: Default
        + Parameter
        + PartialOrd
        + Zero
        + From<u8>
        + Copy
        + BitAnd<Output = Self::AccountRole>;
    type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
    type WeightInfo: WeightInfo;
}

pub trait WeightInfo {
    fn update_something() -> Weight;
    fn account_transfer_and_lock() -> Weight;
    fn account_disable() -> Weight;
    fn account_add() -> Weight;
    fn register_pilot() -> Weight;
}

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Account roles. Add additional values if required
#[allow(dead_code)]
const NONE_ROLE: u8 = 0x00;
pub const ADMIN_ROLE: u8 = 0x01;
#[allow(dead_code)]
pub const PILOT_ROLE: u8 = 0x02;
#[allow(dead_code)]
pub const REGISTRAR_ROLE: u8 = 0x03;

// Storage, Events, Errors are declared using rust macros
// How to use macros see
// https://substrate.dev/docs/en/knowledgebase/runtime/macros

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    // A unique name is used to ensure that the pallet's storage items are isolated.
    // This name may be updated, but each pallet in the runtime must use a unique name.
    // ---------------------------------vvvvvvvvvvvvvv
    trait Store for Module<T: Trait> as TemplateModule {
        // Learn more about declaring storage items:
        // https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
        Something get(fn something): Option<u32>;

        AccountRegistry
            get(fn account_registry)
            config(genesis_account_registry):
            map hasher(blake2_128_concat) T::AccountId => AccountOf<T>;

    }
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Balance = BalanceOf<T>,
        AccountRole = <T as Trait>::AccountRole,
    {
        // Event documentation should end with an array that provides descriptive names for event parameters.
        /// Something store value has been updated. [value, who]
        SomethingStored(u32, AccountId),
        /// New account has been created [who, account, role]
        AccountCreated(AccountId, AccountId, AccountRole),
        /// Account has been disabled [who, account]
        AccountDisabled(AccountId, AccountId),
        /// Lock balance [who, balance]
        BalanceLocked(AccountId, Balance),
        /// Pilot has been registered [who, account]
        PilotRegistered(AccountId, AccountId),
        // add other events here
    }
);

// Errors inform users that something went wrong.
// learn more https://substrate.dev/docs/en/knowledgebase/runtime/errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        NoneValue,
        /// Operation is not valid
        InvalidAction,
        /// Incorrect data provided
        InvalidData,
        /// Origin do not have sufficient privileges to perform the operation
        NotAuthorized,
        /// Account doesn't exist
        NotExists,
        // add additional errors below
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Make module constants visible in Node's metadata
        const AdminRole: T::AccountRole = T::AdminRole::get();

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        /// Create or update an entry in account registry with specific role.
        #[weight = <T as Trait>::WeightInfo::account_add()]
        pub fn account_add(origin, account: T::AccountId, role: T::AccountRole) -> dispatch::DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;
            ensure!(AccountOf::<T>::is_role_correct(role), Error::<T>::InvalidData);
            ensure!(Self::account_is_admin(&who), Error::<T>::NotAuthorized);

            // Update storage.
            AccountRegistry::<T>::mutate(&account, |acc|{
                debug::info!("account_add: roles={:?} create_time={:?}", acc.roles, acc.create_time);
                acc.roles = role;
                if acc.create_time.is_zero(){
                    // Get current timestamp using pallet-timestamp module
                    acc.create_time = <pallet_timestamp::Module<T>>::get();
                }
            });

            // Emit an event.
            Self::deposit_event(RawEvent::AccountCreated(who, account, role));
            // Return a successful DispatchResult
            Ok(())
        }

        #[weight = <T as Trait>::WeightInfo::register_pilot()]
        pub fn register_pilot(origin, account: T::AccountId, role: T::AccountRole) -> dispatch::DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;
            ensure!(AccountOf::<T>::is_role_correct(role), Error::<T>::InvalidData);
            ensure!(Self::account_is_registrar(&who), Error::<T>::NotAuthorized);

            // Update storage.
            AccountRegistry::<T>::mutate(&account, |acc|{
                debug::info!("register_pilot: roles:{:?} create_time={:?}", acc.roles, acc.create_time);
                acc.roles = PILOT_ROLE.into();
                if acc.create_time.is_zero() {
                    // Get current timestamp using pallet-timestamp module
                    acc.create_time = <pallet_timestamp::Module<T>>::get();
                }
            });

            // Emit an event.
            Self::deposit_event(RawEvent::PilotRegistered(who, account));
            // Return a successful DispatchResult
            Ok(())
        }

        /// Disable account entry by removing it from registry.
        /// Transaction fee for this dispatchable is made up from 3 parts.
        /// 1. base_part. is set by  frame-system::ExtrinsicBaseWeight (default value is 125000000)
        ///    together with pallet-transaction-payment::WeightToFee converter.
        ///    The default WeightToFee implementation is IdentityFee that makes one to one conversion.
        /// 2. tx length part. is set by  pallet-transaction-payment::TransactionByteFee. default value is 1
        ///    Each intrinsic byte give us 1 weight.
        /// 3. weight part. is set by pallet-transaction-payment::WeightToFee  and
        ///    pallet-transaction-payment::FeeMultiplierUpdate  implementations
        /// https://substrate.dev/docs/en/knowledgebase/runtime/fees
        #[weight = <T as Trait>::WeightInfo::account_disable()]
        pub fn account_disable(origin, whom: T::AccountId) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            // Ensure origin has associated account with admin privileges.
            ensure!(Self::account_is_admin(&who), Error::<T>::NotAuthorized);
            // Self disabling is prohibited.
            ensure!(who != whom, Error::<T>::InvalidAction);
            // Raise error if the account doesn't exist or has been disabled already.
            ensure!(AccountRegistry::<T>::contains_key(&whom), Error::<T>::NotExists);
            // Delete a value from storage.
            AccountRegistry::<T>::remove(&whom);
            // or just disable it by assigning 0 value to the role
            // AccountRegistry::<T>::mutate(&whom, |item|{
            // 	item.roles = NONE_ROLE;
            // });
            Self::deposit_event(RawEvent::AccountDisabled(who, whom));
            Ok(())
        }

        /// An example dispatchable that demonstrates `pallet_balances` capability to  froze
        /// account balance for specific purpose.
        /// After `account_transfer_and_lock` was called the account can be put his balance only to pay off fees.
        #[weight = <T as Trait>::WeightInfo::account_transfer_and_lock()]
        pub fn account_transfer_and_lock(origin, whom: T::AccountId, amount: BalanceOf<T>) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;
            // Ensure origin has associated account with admin privileges.
            ensure!(Self::account_is_admin(&sender), Error::<T>::NotAuthorized);

            T::Currency::transfer(&sender, &whom, amount, AllowDeath)?;
            let amount = T::Currency::free_balance(&whom);
            T::Currency::set_lock(
                FEE_LOCK_ID,
                &whom,
                amount,
                WithdrawReasons::except(WithdrawReason::TransactionPayment),
            );
            Self::deposit_event(RawEvent::BalanceLocked(whom, amount));

            Ok(())
        }

        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        /// Origin doesn't pay fee for this transaction and can call it with zero balance.
        #[weight = (10_000_000, DispatchClass::Normal, Pays::No)]
        pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;

            // Update storage.
            Something::put(something);
            debug::info!("do_something: who={:?} what={:?}", who, something);
            // Emit an event.
            Self::deposit_event(RawEvent::SomethingStored(something, who));
            // Return a successful DispatchResult
            Ok(())
        }

        /// An example dispatchable that can require various fee depends of runtime logic.
        /// On increase storage value it requires standard fee value.
        /// On decrease origin doesn't have to pay fee.
        /// See more about fees https://substrate.dev/docs/en/knowledgebase/runtime/fees.
        #[weight = <T as Trait>::WeightInfo::update_something()]
        pub fn update_something(origin, something: u32) -> dispatch::DispatchResultWithPostInfo{
            let who = ensure_signed(origin)?;
            ensure!(Self::account_is_admin(&who), Error::<T>::NotAuthorized);
            Something::try_mutate(|v|->dispatch::DispatchResultWithPostInfo{
                let res = match v {
                    // disable pay
                    Some(ref prev) if *prev>something => (Pays::No).into(),
                    // default weight and pay value
                    _ => None.into(),
                };
                *v = Some(something);
                // Event emission should be perform after the storage has been updated.
                // Here we can ensure that update will succeed.
                Self::deposit_event(RawEvent::SomethingStored(something, who));
                Ok(res)
            })
        }
    }
}
// Module allows  use  common functionality by dispatchables
impl<T: Trait> Module<T> {
    // Implement module function.
    // Public functions can be called from other runtime modules.
    /// Check if an account has ADMIN role
    pub fn account_is_admin(acc: &T::AccountId) -> bool {
        AccountRegistry::<T>::get(acc).is_admin()
    }
    /// Check if an account has PILOT role
    pub fn account_is_registrar(acc: &T::AccountId) -> bool {
        AccountRegistry::<T>::get(acc).is_registrar()
    }
}

impl<T: Trait> OnKilledAccount<T::AccountId> for Module<T> {
    fn on_killed_account(who: &T::AccountId) {
        AccountRegistry::<T>::remove(&who);
    }
}
