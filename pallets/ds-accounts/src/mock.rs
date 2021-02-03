use crate::{Module, Trait};
use frame_support::{
    impl_outer_event, impl_outer_origin, parameter_types,
    weights::{constants::RocksDbWeight, Weight},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

impl_outer_origin! {
    pub enum Origin for Test {}
}
mod template {
    pub use crate::Event;
}
mod balance {
    pub use pallet_balances::Event;
}

impl_outer_event! {
    pub enum TestEvent for Test {
        system<T>,
        template<T>,
        balance<T>,
    }
}

// Configure a mock runtime to test the pallet.
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
pub type Balance = u128;
pub type System = system::Module<Test>;

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = RocksDbWeight;
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = TemplateModule;
    type SystemWeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Trait for Test {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

// Assign module constant values
parameter_types! {
    pub const AdminRole: u8 = super::ADMIN_ROLE;
}

struct WeightInfo;
impl crate::WeightInfo for WeightInfo {
    fn account_disable() -> Weight {
        <() as crate::WeightInfo>::account_disable()
    }
    fn account_add() -> Weight {
        <() as crate::WeightInfo>::account_add()
    }

    fn register_pilot() -> Weight {
        <() as crate::WeightInfo>::register_pilot()
    }
}

impl Trait for Test {
    type Event = TestEvent;
    type AdminRole = AdminRole;
    type AccountRole = u8;
    type Currency = pallet_balances::Module<Self>;
    type WeightInfo = ();
}

parameter_types! {
    pub const MaxLocks: u32 = 50;
    pub const ExistentialDeposit: u64 = 100;
}

impl pallet_balances::Trait for Test {
    type Balance = Balance;
    type Event = TestEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = MaxLocks;
}

// pub type Balances = pallet_balances::Module<Test>;
// parameter_types! {
//     pub const TransactionByteFee: Balance = 1;
// }
//
// impl pallet_transaction_payment::Trait for Test {
//     type Currency = Balances;
//     type OnTransactionPayment = ();
//     type TransactionByteFee = TransactionByteFee;
//     type WeightToFee = IdentityFee<Balance>;
//     type FeeMultiplierUpdate = ();
// }

pub type TemplateModule = Module<Test>;
pub type Account = super::AccountOf<Test>;

static INITIAL: [(
    <Test as system::Trait>::AccountId,
    <Test as super::Trait>::AccountRole,
); 1] = [(1, super::ADMIN_ROLE)];

static INITIAL_BALANCE: super::BalanceOf<Test> = 100000;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        // Provide some initial balances
        balances: INITIAL.iter().map(|x| (x.0, INITIAL_BALANCE)).collect(),
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    super::GenesisConfig::<Test> {
        // First account is admin
        genesis_account_registry: INITIAL
            .iter()
            .map(|(acc, role)| {
                (
                    *acc,
                    Account {
                        roles: *role,
                        create_time: 0,
                        managed_by: Default::default(),
                    },
                )
            })
            .collect(),
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    storage.into()
}
