use crate::{Module, Trait};
use frame_support::{
    impl_outer_event, impl_outer_origin, parameter_types,
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        DispatchClass, Weight
    },
};
use frame_system as system;
use system::limits::{BlockLength, BlockWeights};
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

const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub MockBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
    pub MockBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
    pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockLength = MockBlockLength;
    type BlockWeights = MockBlockWeights;
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
    type DbWeight = RocksDbWeight;
    type Version = ();
    type PalletInfo = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = DSAccountsModule;
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
}

parameter_types! {
    pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Test {
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
    fn register_uav() -> Weight {
        <() as crate::WeightInfo>::register_uav()
    }
}

impl Trait for Test {
    type Event = TestEvent;
    type AdminRole = AdminRole;
    type AccountRole = u8;
    type Currency = pallet_balances::Module<Self>;
    type WeightInfo = ();
    type MetaIPFS = Vec<u8>;        
    type SerialNumber = Vec<u8>;    //not sure which type use here, for simplicity will be string
}

parameter_types! {
    pub const MaxLocks: u32 = 50;
    pub const ExistentialDeposit: u64 = 100;
}

impl pallet_balances::Config for Test {
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
// impl pallet_transaction_payment::Config for Test {
//     type Currency = Balances;
//     type OnTransactionPayment = ();
//     type TransactionByteFee = TransactionByteFee;
//     type WeightToFee = IdentityFee<Balance>;
//     type FeeMultiplierUpdate = ();
// }

pub type DSAccountsModule = Module<Test>;
pub type Account = super::AccountOf<Test>;

static INITIAL: [(
    <Test as system::Config>::AccountId,
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
