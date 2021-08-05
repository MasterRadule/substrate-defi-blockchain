//! Benchmarking setup for pallet-defi

use super::*;

#[allow(unused)]
use crate::Pallet as DeFiPallet;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::{RawOrigin, EventRecord};
use crate::pallet::BalanceOf;
use hex_literal::hex;
use sp_runtime::traits::One;

fn alice<T: Config>() -> T::AccountId {
	let bytes = hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d");
	T::AccountId::decode(&mut &bytes[..]).unwrap_or_default()
}


fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::Event = generic_event.into();
	// compare to the last event record
	let EventRecord { event, .. } = events.last().unwrap();
	assert_eq!(event, &system_event);
}

benchmarks! {
	deposit {
		let caller = alice::<T>();
		let amount = BalanceOf::<T>::one();
		let current_block: T::BlockNumber = frame_system::Pallet::<T>::block_number();
	}: _(RawOrigin::Signed(caller.clone()), amount)
	verify {
		assert_last_event::<T>(Event::Deposited(caller, amount, current_block).into());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{Test, ExtBuilder};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		ExtBuilder::build().execute_with(|| {
			assert_ok!(test_benchmark_deposit::<Test>());
		});
	}
}
