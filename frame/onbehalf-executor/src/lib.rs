#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_evm::Runner;

	use frame_support::sp_std::vec::Vec;
	use sha3::Digest;
	use sp_core::{H160, H256, U256};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {}

	// data: 70a08231000000000000000000000000a58482131a8d67725e996af72d91a849acc0f4a1

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		#[pallet::call_index(0)]
		pub fn call_on_behalf(
			_origin: OriginFor<T>,
			from: H160,
			to: H160,
			input: Vec<u8>,
		) -> DispatchResult {
			frame_support::log::info!("input: {:?}", input);

			let result = T::Runner::call(
				from,
				to,
				input,
				U256::from(0),
				u64::MAX,
				None,
				None,
				Some(1.into()),
				Default::default(),
				false,
				false,
				&pallet_evm::EvmConfig::istanbul(),
			);

			match result {
				Ok(output) => {
					frame_support::log::info!("output: {:?}", output.value);
				}
				Err(_) => {
					frame_support::log::warn!("call failed =(");
				}
			};

			Ok(())
		}

		#[pallet::weight(0)]
		#[pallet::call_index(1)]
		pub fn set_storage_to(
			_origin: OriginFor<T>,
			to: H160,
			slot: H256,
			value: H256,
		) -> DispatchResult {
			pallet_evm::AccountStorages::<T>::insert(to, slot, value);

			Ok(())
		}

		#[pallet::weight(0)]
		#[pallet::call_index(2)]
		pub fn set_erc20_balance_to(
			_origin: OriginFor<T>,
			erc20_address: H160,
			erc20_balance_slot: H256,
			address: H160,
			value: H256,
		) -> DispatchResult {
			let hasher = sha3::Keccak256::new();

			let p_address = &address;

			let u256_address = H256::from(*p_address);

			let address_bytes = u256_address.as_bytes();
			let slot_bytes = erc20_balance_slot.as_bytes();

			let input = &[&address_bytes[..], &slot_bytes[..]].concat();

			let storage_slot = hasher
				.chain_update(input)
				.finalize()
				.using_encoded(|x| H256::from_slice(&x[1..]));

			pallet_evm::AccountStorages::<T>::insert(erc20_address, storage_slot, value);

			Ok(())
		}

		#[pallet::weight(0)]
		#[pallet::call_index(3)]
		pub fn get_storage(_origin: OriginFor<T>, to: H160, slot: H256) -> DispatchResult {
			let value = pallet_evm::AccountStorages::<T>::get(to, slot);

			frame_support::log::info!("value: {:?}", value);

			Ok(())
		}
	}
}
