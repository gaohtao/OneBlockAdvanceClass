use crate as pallet_kitties;
use frame_support::traits::{ConstU16,  ConstU64};
use frame_support::parameter_types;
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};


use pallet_randomness_collective_flip;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

type Balance = u128; //覆盖Runtime中定义的Balance类型，简化成u128

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},

		KittiesModule: pallet_kitties::{Pallet, Call, Storage, Event<T>},
		Randomness: pallet_randomness_collective_flip::{Pallet, Storage},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);


impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
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
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}


impl pallet_balances::Config for Test { // for Runtime修改为 for Test
	type MaxLocks = MaxLocks;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>; //Runtime修改为Test
}

// 从Runtime中将pallet_randomness_collective_flip相关配置复制到此处
impl pallet_randomness_collective_flip::Config for Test {}  


// 从Runtime中将pallet_balances相关配置复制到此处
parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
	pub const MaxLocks: u32 = 50;

    pub const MaxKittyOwned: u32 = 9;	// One can own at most 9Kitties
	pub const KittyStake: u128 = 1_000;   //定义创建每只Kitty时需要质押的原生token数量
}


impl pallet_kitties::Config for Test {
	type Event = Event;
	type Currency = Balances;   // 引入钱包类型
	type KittyRandomness = Randomness; // 引入Randomness类型
	type MaxKittyOwned = MaxKittyOwned; // <- add this line
	type KittyIndex = u32; //定义Kitty的索引ID类型
	type KittyStake = KittyStake; //引入KittyStake常量

}



// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = system::GenesisConfig::default().build_storage::<Test>().unwrap().into();

	// 定义创世账户的余额，账户为1，2，3，4
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(1, 10_000_000_000), (2, 10_000_000_000), (3, 500)],
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
