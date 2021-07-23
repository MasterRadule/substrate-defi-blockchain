#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use codec::{Decode, Encode};
use substrate_fixed::{types::I32F32};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Default, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct AccountInfo<BlockNumber> {
    /// The balance of the account after last adjustment
    deposit_principal: I32F32,
    /// The time (block height) at which the deposit balance was last adjusted
    deposit_date: BlockNumber,
    /// Borrowed balance
    borrow_principal: I32F32,
    /// The time (block height) at which the borrowing balance was last adjusted
    borrow_date: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use crate::{AccountInfo, I32F32};
    use frame_support::traits::{Currency, ReservableCurrency, ExistenceRequirement};
    use sp_runtime::{SaturatedConversion, ModuleId};
    use sp_runtime::traits::AccountIdConversion;

    const PALLET_ID: ModuleId = ModuleId(*b"crsearly");

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
    pub(super) type Accounts<T: Config> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, AccountInfo<T::BlockNumber>, ValueQuery>;

    #[pallet::event]
    #[pallet::metadata(AccountIdOf < T > = "AccountId", T::BlockNumber = "BlockNumber")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Funds deposited. [who, amount, block]
        Deposited(AccountIdOf<T>, I32F32, T::BlockNumber),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// User's borrowed amount is not zero
        UserInDebt,
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
            ensure!(account_info.borrow_principal == 0, Error::<T>::UserInDebt);

            let current_block = frame_system::Pallet::<T>::block_number();
            let deposit_amount = I32F32::from_num(amount.saturated_into::<u128>());

            // Deposit funds to pallet
            T::Currency::transfer(
                &user,
                &Self::account_id(),
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            // Set account info
            account_info.deposit_principal += deposit_amount;
            account_info.deposit_date = current_block;

            // Put updated account info into storage
            <Accounts<T>>::insert(&user, account_info);

            // Emit an event
            Self::deposit_event(Event::Deposited(user, deposit_amount, current_block));

            // Return a successful DispatchResult
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// The account ID of pallet
        fn account_id() -> T::AccountId {
            PALLET_ID.into_account()
        }
    }
}
