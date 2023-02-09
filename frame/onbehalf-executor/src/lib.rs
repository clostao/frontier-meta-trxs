#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_evm::Runner;

	use frame_support::sp_std::vec::Vec;
	use sp_core::{H160, U256};

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
	}
}
