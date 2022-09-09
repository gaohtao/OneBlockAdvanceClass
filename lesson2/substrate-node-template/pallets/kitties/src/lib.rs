#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::{
		// sp_runtime::traits::Hash,
		traits::{ Randomness, Currency, tokens::ExistenceRequirement },
	};
	use sp_io::hashing::blake2_128;
	use scale_info::TypeInfo;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	#[codec(mel_bound())]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],   // Using 16 bytes to represent a kitty DNA
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>,
	}
	// Enum declaration for Gender.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency handler for the Kitties pallet.
		type Currency: Currency<Self::AccountId>;

		/// The maximum amount of Kitties a single account can own.
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;

		/// The type of Randomness we want to specify for this pallet.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		// ACTION #5a: Declare errors.
		/// Handles arithmetic overflow when incrementing the Kitty counter.
        KittyIndexOverflow,
        /// An account cannot own more Kitties than `MaxKittyCount`.
        ExceedMaxKittyOwned,
        /// Buyer cannot be the owner.
        BuyerIsKittyOwner,
        /// Cannot transfer a kitty to its owner.
        TransferToSelf,
        /// Handles checking whether the Kitty exists.
        KittyNotExist,
        KittyExists,  //Kitty exists.
 
        /// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.    
        NotKittyOwner,
        /// Ensures the Kitty is for sale.
        KittyNotForSale,
        /// Ensures that the buying price is greater than the asking price.
        KittyBidPriceTooLow,
        /// Ensures that an account has enough funds to purchase a Kitty.
        NotEnoughBalance,
	}

	// Events.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// ACTION #3: Declare events
		/// A new Kitty was successfully created. \[sender, kitty_id\]
        Created(T::AccountId, KittyIndex),
        /// Kitty price was successfully set. \[sender, kitty_id, new_price\]
        PriceSet(T::AccountId, KittyIndex, Option<BalanceOf<T>>),
        /// A Kitty was successfully transferred. \[from, to, kitty_id\]
        Transferred(T::AccountId, T::AccountId, KittyIndex),
        /// A Kitty was successfully bought. \[buyer, seller, kitty_id, bid_price\]
        Bought(T::AccountId, T::AccountId, KittyIndex, BalanceOf<T>),
		/// A Kitty was successfully bred. \[sender, new_kitty_id, parent1, parent2\], 
		BredSuccess(T::AccountId, KittyIndex, KittyIndex, KittyIndex),
	}

    // Storage items. 小猫序号类型， 
    type KittyIndex = u32;

    #[pallet::type_value]
    pub fn GetDefaultValue() -> KittyIndex {
        0_u32
    }
	
	// Storage items.
	
	#[pallet::storage]
	#[pallet::getter(fn get_kitty_id)]
	/// Keeps track of the number of Kitties in existence. 小猫序号， 默认从0开始, 也代表着小猫的总数
	pub(super) type NextKittyId<T: Config> = StorageValue<_, KittyIndex, ValueQuery, GetDefaultValue>;

	

	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	/// Keeps track of the number of Kitties in existence.
	pub(super) type KittyCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	/// Stores a Kitty's unique traits, owner and price.
	pub(super) type Kitties<T: Config> = StorageMap<_, Twox64Concat, KittyIndex, Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	/// Keeps track of what accounts own what Kitty.
	pub(super) type KittiesOwned<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<KittyIndex, T::MaxKittyOwned>, ValueQuery>;


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new unique kitty.
		///
		/// The actual kitty creation is done in the `mint()` function.
		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			// ACTION #1: create_kitty
			let sender = ensure_signed(origin)?; // <- add this line
			let kitty_id = Self::mint(&sender, None, None)?; // <- add this line
			// Logging to the console
			log::info!("A kitty is born with ID: {:?}.", kitty_id); // <- add this line

			// ACTION #4: Deposit `Created` event
			Self::deposit_event(Event::Created(sender, kitty_id));

			Ok(())
		}

		// TODO Part IV: set_price
		#[pallet::weight(100)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: KittyIndex,
			new_price: Option<BalanceOf<T>>
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;


			// ACTION #1a: Checking Kitty owner
			ensure!(Self::is_kitty_owner(kitty_id, &sender)?, <Error<T>>::NotKittyOwner);
			let mut kitty = Self::kitties(kitty_id).ok_or(<Error<T>>::KittyNotExist)?;


			// ACTION #2: Set the Kitty price and update new Kitty infomation to storage.
			kitty.price = new_price.clone();
			<Kitties<T>>::insert(kitty_id, kitty);


			// ACTION #3: Deposit a "PriceSet" event.
			Self::deposit_event(Event::PriceSet(sender, kitty_id, new_price));


			Ok(())
		}

		// TODO Part IV: transfer
		#[pallet::weight(100)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: KittyIndex
		) -> DispatchResult {
			let from = ensure_signed(origin)?;


			// Ensure the kitty exists and is called by the kitty owner
			ensure!(Self::is_kitty_owner(kitty_id, &from)?, <Error<T>>::NotKittyOwner);


			// Verify the kitty is not transferring back to its owner.
			ensure!(from != to, <Error<T>>::TransferToSelf);


			// Verify the recipient has the capacity to receive one more kitty
			let to_owned = <KittiesOwned<T>>::get(&to);
			ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);


			Self::transfer_kitty_to(kitty_id, &to)?;


			Self::deposit_event(Event::Transferred(from, to, kitty_id));


			Ok(())
		}

		// TODO Part IV: buy_kitty
		#[pallet::weight(100)]
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: KittyIndex,
			bid_price: BalanceOf<T>
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;


			// Check the kitty exists and buyer is not the current kitty owner
			let kitty = Self::kitties(&kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			ensure!(kitty.owner != buyer, <Error<T>>::BuyerIsKittyOwner);


			// ACTION #6: Check if the Kitty is for sale.
			// Check the kitty is for sale and the kitty ask price <= bid_price
			if let Some(ask_price) = kitty.price {
				ensure!(ask_price <= bid_price, <Error<T>>::KittyBidPriceTooLow);
			} else {
				Err(<Error<T>>::KittyNotForSale)?;
			}


			// Check the buyer has enough free balance
			ensure!(T::Currency::free_balance(&buyer) >= bid_price, <Error<T>>::NotEnoughBalance);


			// ACTION #7: Check if buyer can receive Kitty.
			// Verify the buyer has the capacity to receive one more kitty
			let to_owned = <KittiesOwned<T>>::get(&buyer);
			ensure!((to_owned.len() as u32) < T::MaxKittyOwned::get(), <Error<T>>::ExceedMaxKittyOwned);
			let seller = kitty.owner.clone();


			// ACTION #8: Update Balances using the Currency trait.
			// Transfer the amount from buyer to seller
			T::Currency::transfer(&buyer, &seller, bid_price, ExistenceRequirement::KeepAlive)?;


			// Transfer the kitty from seller to buyer
			Self::transfer_kitty_to(kitty_id, &buyer)?;


			// Deposit relevant Event
			Self::deposit_event(Event::Bought(buyer, seller, kitty_id, bid_price));


			Ok(())
		}

		// TODO Part IV: breed_kitty
		/// Breed two kitties to create a new generation
		/// of Kitties.
		#[pallet::weight(100)]
		pub fn breed_kitty(
			origin: OriginFor<T>,
			parent1: KittyIndex,
			parent2: KittyIndex
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;


			// Check: Verify `sender` owns both kitties (and both kitties exist).
			ensure!(Self::is_kitty_owner(parent1, &sender)?, <Error<T>>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(parent2, &sender)?, <Error<T>>::NotKittyOwner);


			// ACTION #9: Breed two Kitties using unique DNA
			let new_dna = Self::breed_dna(parent1, parent2)?;
			
			// ACTION #10: Mint new Kitty using new DNA
			let kitty_id = Self::mint(&sender, Some(new_dna), None)?;

            // Deposit relevant Event
            Self::deposit_event(Event::BredSuccess(sender, kitty_id, parent1, parent2));
			// Self::deposit_event(Event::Created(&sender, kitty_id));

			Ok(())
		}

			
	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {

		// // get next index of new Kitty, 获取下一个序号, ，到达最大值时就报告错误        
		fn get_next_id() -> Result<KittyIndex, Error<T>> {

			   return  Self::get_kitty_id().checked_add(1).ok_or(<Error<T>>::KittyIndexOverflow);
			
		}   

        // get kitty by id
        fn get_kitty(kitty_id:KittyIndex) -> Result<Kitty<T>,Error<T>> {
            match Self::kitties(kitty_id){
                Some(kitty) => Ok(kitty),
                None => Err(<Error<T>>::KittyNotExist),
            }
        }


		// Generate a random gender value
		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}

		// Generate a random DNA value
		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::extrinsic_index().unwrap_or_default(),
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		// Create new DNA with existing DNA
		pub fn breed_dna(parent1: KittyIndex, parent2: KittyIndex) -> Result<[u8; 16], Error<T>> {
			// let dna1 = Self::kitties(parent1).ok_or(<Error<T>>::KittyNotExist)?.dna;
			// let dna2 = Self::kitties(parent2).ok_or(<Error<T>>::KittyNotExist)?.dna;

			let dna1 = Self::get_kitty(parent1)?.dna;
			let dna2 = Self::get_kitty(parent2)?.dna;

			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len() {
				new_dna[i] = (new_dna[i] & dna1[i]) | (!new_dna[i] & dna2[i]);
			}
			Ok(new_dna)
		}

		// ACTION #2: Write mint function
		// Helper to mint a Kitty.
		pub fn mint(
			owner: &T::AccountId,
			dna: Option<[u8; 16]>,
			gender: Option<Gender>,
		) -> Result<KittyIndex, Error<T>> {
			let kitty = Kitty::<T> {
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gen_gender),
				owner: owner.clone(),
			};


			let kitty_id = Self::get_kitty_id();    //  当前序号，从0开始
			let next_id = Self::get_next_id()?;     //  递增序号

			// Check if the kitty does not already exist in our storage map
			ensure!(Self::kitties(&kitty_id) == None, <Error<T>>::KittyExists);


			// Performs this operation first because as it may fail
			<KittiesOwned<T>>::try_mutate(&owner, |kitty_vec| {
				kitty_vec.try_push(kitty_id)
			}).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;


			<Kitties<T>>::insert(kitty_id, kitty);

			<NextKittyId<T>>::put(next_id);

			Ok(kitty_id)
		}

		// Helper to check correct kitty owner
		pub fn is_kitty_owner(kitty_id: KittyIndex, acct: &T::AccountId) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *acct),
				None => Err(<Error<T>>::KittyNotExist)
			}
		}

		// TODO Part IV: Write transfer_kitty_to
		pub fn transfer_kitty_to(
			kitty_id: KittyIndex,
			to: &T::AccountId,
		) -> Result<(), Error<T>> {
			let mut kitty = Self::kitties(kitty_id).ok_or(<Error<T>>::KittyNotExist)?;


			let prev_owner = kitty.owner.clone();


			// Remove `kitty_id` from the KittyOwned vector of `prev_kitty_owner`
			<KittiesOwned<T>>::try_mutate(&prev_owner, |owned| {
				if let Some(ind) = owned.iter().position(|&id| id == kitty_id) {
					owned.swap_remove(ind);
					return Ok(());
				}
				Err(())
			}).map_err(|_| <Error<T>>::KittyNotExist)?;


			// Update the kitty owner
			kitty.owner = to.clone();
			// Reset the ask price so the kitty is not for sale until `set_price()` is called
			// by the current owner.
			kitty.price = None;


			<Kitties<T>>::insert(kitty_id, kitty);


			<KittiesOwned<T>>::try_mutate(to, |vec| {
				vec.try_push(kitty_id)
			}).map_err(|_| <Error<T>>::ExceedMaxKittyOwned)?;


			Ok(())
		}
	}
}