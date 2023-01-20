#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use core::ops::{Mul, Sub};
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, WithdrawReasons},
	};
	use frame_system::pallet_prelude::*;
	use pallet_ethereum::Transaction;
	use pallet_evm::{AddressMapping, Runner};
	use sp_arithmetic::traits::SaturatedConversion;
	use sp_core::{H160, U256};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_evm::Config {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		#[pallet::call_index(0)]
		pub fn call_on_behalf(
			origin: OriginFor<T>,
			on_behalf_of: H160,
			nonce: U256,
			gas_price: U256,
			gas_limit: U256,
			to: H160,
			value: U256,
		) -> DispatchResult {
			let executor = ensure_signed(origin).expect("signed origin");
			let msg_sender = T::AddressMapping::into_account_id(on_behalf_of);

			let prev_total_issuance = T::Currency::active_issuance();

			frame_support::log::warn!("executor: {:?}", executor);
			frame_support::log::warn!("msg_sender: {:?}", msg_sender);

			frame_support::log::warn!(
				"executor balance: {:?}",
				T::Currency::free_balance(&executor)
			);

			let spendable_gas = gas_price.mul(gas_limit);

			frame_support::log::warn!("spendable_gas: {:?}", spendable_gas);

			T::Currency::transfer(
				&executor,
				&msg_sender,
				spendable_gas.as_u128().saturated_into(),
				frame_support::traits::ExistenceRequirement::AllowDeath,
			)
			.expect("not fail");

			// T::Currency::withdraw(
			// 	&executor,
			// 	spendable_gas.as_u128().saturated_into(),
			// 	WithdrawReasons::FEE,
			// 	frame_support::traits::ExistenceRequirement::KeepAlive,
			// )
			// .unwrap();

			// T::Currency::deposit_creating(&msg_sender, spendable_gas.as_u128().saturated_into());

			let is_transactional = true;

			let pre_trx_total_issuance = T::Currency::active_issuance();
			let result = T::Runner::call(
				on_behalf_of,
				to,
				Default::default(),
				value,
				gas_limit.as_u64(),
				Some(gas_price),
				None,
				Some(nonce),
				Default::default(),
				is_transactional,
				false,
				&pallet_evm::EvmConfig::frontier(),
			);
			let post_trx_total_issuance = T::Currency::active_issuance();

			let inner_diff = if post_trx_total_issuance.gt(&pre_trx_total_issuance) {
				(true, post_trx_total_issuance.sub(pre_trx_total_issuance))
			} else {
				(false, pre_trx_total_issuance.sub(post_trx_total_issuance))
			};

			let used_gas = match result {
				Ok(info) => info.used_gas,
				Err(_) => U256::from(0),
			};

			frame_support::log::info!("gas used by {:?}", used_gas);

			let refund = spendable_gas.sub(used_gas.as_u128().mul(gas_price.as_u128()));

			T::Currency::transfer(
				&msg_sender,
				&executor,
				refund.as_u128().saturated_into(),
				frame_support::traits::ExistenceRequirement::AllowDeath,
			)
			.expect("not fail");

			// T::Currency::withdraw(
			// 	&msg_sender,
			// 	refund.as_u128().saturated_into(),
			// 	WithdrawReasons::FEE,
			// 	frame_support::traits::ExistenceRequirement::KeepAlive,
			// )
			// .unwrap();
			// T::Currency::burn(refund.as_u128().saturated_into());

			// T::Currency::deposit_creating(&executor, refund.as_u128().saturated_into());

			let post_total_issuance = T::Currency::active_issuance();

			let outer_diff = if post_total_issuance.gt(&prev_total_issuance) {
				(true, post_total_issuance.sub(prev_total_issuance))
			} else {
				(false, prev_total_issuance.sub(post_total_issuance))
			};

			frame_support::log::info!("refund {:?}", refund);

			frame_support::log::info!("has been burnt: {:?}", outer_diff.0);
			assert_eq!(inner_diff.0, outer_diff.0);
			assert_eq!(inner_diff.1, outer_diff.1);

			Ok(())
		}
	}
}
