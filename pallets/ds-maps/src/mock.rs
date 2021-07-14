#![allow(clippy::from_over_into)]

use crate as pallet_ds_maps;
use crate::Trait;
use frame_support::{
    construct_runtime, parameter_types,
    weights::Weight,
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use substrate_fixed::types::I10F22;
use pallet_ds_accounts::ADMIN_ROLE;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        Timestamp: pallet_timestamp::{Module, Call, Storage},
        Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
        DSAccountsModule: pallet_ds_accounts::{Module, Call, Storage, Event<T>},
        DSMapsModule: pallet_ds_maps::{Module, Call, Storage, Event<T>},
    }
);

pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
pub type Balance = u128;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

impl system::Config for Test {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
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

struct WeightInfo;
impl crate::WeightInfo for WeightInfo {
    fn root_add() -> Weight {
        <() as crate::WeightInfo>::root_add()
    }
    fn zone_add() -> Weight {
        <() as crate::WeightInfo>::zone_add()
    }
    fn root_remove() -> Weight {
        <() as crate::WeightInfo>::root_remove()
    }
    fn zone_remove() -> Weight {
        <() as crate::WeightInfo>::zone_remove()
    }
    fn change_area_type() -> Weight {
        <() as crate::WeightInfo>::change_area_type()
    }
}

// After researches, consider placing here max grid sizes
parameter_types! {
    pub const MaxHeight: u16 = 400;
    pub const MaxBuildingsInArea: u16 = 2;
}

impl Trait for Test {
    type Event = Event;
    type WeightInfo = ();
    type Coord = I10F22;

    type RawCoord = i32;
    type MaxBuildingsInArea = MaxBuildingsInArea;
    type MaxHeight = MaxHeight;
}

parameter_types! {
    pub const MaxLocks: u32 = 50;
    pub const ExistentialDeposit: u64 = 100;
}

impl pallet_balances::Config for Test {
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = MaxLocks;
}

parameter_types! {
    pub const AdminRole: u8 = ADMIN_ROLE;
}

impl pallet_ds_accounts::Trait for Test {
    type Event = Event;
    type AdminRole = AdminRole;
    type AccountRole = u8;
    type Currency = pallet_balances::Module<Self>;
    type WeightInfo = ();
    type SerialNumber = Vec<u8>;
    type MetaIPFS = Vec<u8>;
}

static INITIAL: [(
    <Test as system::Config>::AccountId,
    <Test as pallet_ds_accounts::Trait>::AccountRole,
); 1] = [(1, ADMIN_ROLE)];

pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
        pallet_ds_accounts::GenesisConfig::<Test> {
            // First account is admin
            genesis_account_registry: INITIAL
                .iter()
                .map(|(acc, role)| {
                    (
                        *acc,
                        pallet_ds_accounts::Account {
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