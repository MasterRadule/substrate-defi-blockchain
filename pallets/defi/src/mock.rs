use crate as pallet_defi;
use frame_support::parameter_types;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use frame_support::sp_runtime::{FixedU128};
use hex_literal::hex;
use codec::Decode;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub type Balance = u128;
pub type AccountId = u128;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		DefiModule: pallet_defi::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
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
	type AccountId = AccountId;
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
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DustRemoval = ();
	/// The ubiquitous event type.
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub DepositRate: FixedU128 = FixedU128::from_inner(92828) / FixedU128::from_inner(10000000000000);
	pub BorrowingRate: FixedU128 = FixedU128::from_inner(128727) / FixedU128::from_inner(10000000000000);
	pub const NumberOfBlocksYearly: u32 = 5256000;
}

impl pallet_defi::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type DepositRate = DepositRate;
	type BorrowingRate = BorrowingRate;
	type NumberOfBlocksYearly = NumberOfBlocksYearly;
}

fn alice() -> AccountId {
	let bytes = hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
	AccountId::decode(&mut &bytes[..]).unwrap_or_default()
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![(alice(), 100)],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap();

		pallet_balances::GenesisConfig::<Test> {
			balances: self
				.endowed_accounts
				.iter()
				.map(|(acc, balance)| (*acc, *balance))
				.collect(),
		}
			.assimilate_storage(&mut storage)
			.unwrap();

		storage.into()
	}
}
