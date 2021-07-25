#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use codec::{Decode, Encode};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Default, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AddressInfo<Balance, BlockNumber> {
    /// The balance of the account after last adjustment
    deposit_principal: Balance,
    /// The time (block height) at which the deposit balance was last adjusted
    deposit_date: BlockNumber,
    /// Borrowed balance
    borrow_principal: Balance,
    /// The time (block height) at which the borrowing balance was last adjusted
    borrow_date: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::*;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*, PalletId};
    use frame_support::traits::{Currency, ReservableCurrency, ExistenceRequirement};
    use crate::AddressInfo;
    use frame_support::sp_runtime::traits::{Saturating, Zero, One};
    use frame_support::sp_runtime::sp_std::convert::TryInto;
    use frame_support::sp_runtime::{FixedU128, FixedPointNumber, SaturatedConversion};
    use sp_runtime::traits::AccountIdConversion;

    const PALLET_ID: PalletId = PalletId(*b"defisrvc");

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency in which deposit/borrowing work
        type Currency: ReservableCurrency<Self::AccountId>;
    }

    type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub(super) type Accounts<T: Config> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, AddressInfo<BalanceOf<T>, T::BlockNumber>, ValueQuery>;

    #[pallet::event]
    #[pallet::metadata(AccountIdOf<T> = "AccountId", BalanceOf<T> = "Balance", T::BlockNumber = "BlockNumber")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Funds deposited. [who, amount, block]
        Deposited(AccountIdOf<T>, BalanceOf<T>, T::BlockNumber),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// User's borrowed amount is not zero
        UserInDebt,
        /// User has no deposited balance
        NoFundsDeposited,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Deposit funds
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn deposit(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let user = ensure_signed(origin)?;

            // Get account info of extrinsic caller and check if it has borrowed funds
            let mut account_info = <Accounts<T>>::get(&user);
            ensure!(account_info.borrow_principal == <BalanceOf<T>>::zero(), Error::<T>::UserInDebt);

            let current_block = frame_system::Pallet::<T>::block_number();

            // Deposit funds to pallet
            T::Currency::transfer(
                &user,
                &Self::account_id(),
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            // Set account info
            account_info.deposit_principal += amount;
            account_info.deposit_date = current_block;

            // Put updated account info into storage
            <Accounts<T>>::insert(&user, account_info);

            // Emit an event
            Self::deposit_event(Event::Deposited(user, amount, current_block));

            // Return a successful DispatchResult
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// The account ID of pallet
        pub fn account_id() -> T::AccountId {
            PALLET_ID.into_account()
        }

        pub fn get_balance(user: &T::AccountId) -> BalanceOf<T> {
            let account_info = <Accounts<T>>::get(user);
            if account_info.deposit_principal == <BalanceOf<T>>::zero() {
                return <BalanceOf<T>>::zero();
            }

            // Calculate elapsed blocks
            let current_block = frame_system::Pallet::<T>::block_number();
            let elapsed_time_block_number = current_block - account_info.deposit_date;
            let elapsed_time: u32 = TryInto::try_into(elapsed_time_block_number)
                .ok()
                .expect("blockchain will not exceed 2^32 blocks; qed");

            let rate = FixedU128::from_inner(5) / FixedU128::from_inner(100);
            let multiplier = (FixedU128::one() + rate).saturating_pow(elapsed_time as usize).saturating_sub(FixedU128::one());
            let deposit_principal_fixed = FixedU128::from_inner(account_info.deposit_principal.saturated_into::<u128>());

            return deposit_principal_fixed.saturating_add(deposit_principal_fixed.saturating_mul(multiplier)).into_inner().saturated_into();
        }
    }
}
