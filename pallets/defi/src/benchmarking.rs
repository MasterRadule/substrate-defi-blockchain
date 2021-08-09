//! Benchmarking setup for pallet-defi
use super::*;

#[allow(unused)]
use crate::Pallet as DeFiPallet;
use frame_benchmarking::benchmarks;
use frame_system::{RawOrigin, EventRecord};
use crate::pallet::BalanceOf;
use hex_literal::hex;
use sp_runtime::traits::One;
use sp_runtime::SaturatedConversion;

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
		frame_system::Pallet::<T>::set_block_number(1u32.into());
	}: _(RawOrigin::Signed(caller.clone()), amount)
	verify {
		assert_last_event::<T>(Event::Deposited(caller, amount, 1u32.into()).into());
	}

	withdraw {
		let caller = alice::<T>();
		let amount = BalanceOf::<T>::one();
		frame_system::Pallet::<T>::set_block_number(1u32.into());
		_ = DeFiPallet::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), amount);
	}: _(RawOrigin::Signed(caller.clone()), amount)
	verify {
		assert_last_event::<T>(Event::Withdrawn(caller, amount, 1u32.into()).into());
	}

	borrow {
		let caller = alice::<T>();
		let amount = BalanceOf::<T>::one();
		frame_system::Pallet::<T>::set_block_number(1u32.into());
		_ = DeFiPallet::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), 2u128.saturated_into());
	}: _(RawOrigin::Signed(caller.clone()), amount)
	verify {
		assert_last_event::<T>(Event::Borrowed(caller, amount, 1u32.into()).into());
	}

	repay {
		let caller = alice::<T>();
		let amount = BalanceOf::<T>::one();
		frame_system::Pallet::<T>::set_block_number(1u32.into());
		_ = DeFiPallet::<T>::deposit(RawOrigin::Signed(caller.clone()).into(), 2u128.saturated_into());
		_ = DeFiPallet::<T>::borrow(RawOrigin::Signed(caller.clone()).into(), 1u128.saturated_into());
	}: _(RawOrigin::Signed(caller.clone()), amount)
	verify {
		assert_last_event::<T>(Event::LoanRepaid(caller, amount, 1u32.into()).into());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{Test, ExtBuilder};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(test_benchmark_deposit::<Test>());
			assert_ok!(test_benchmark_withdraw::<Test>());
			assert_ok!(test_benchmark_borrow::<Test>());
			assert_ok!(test_benchmark_repay::<Test>());
		});
	}
}
