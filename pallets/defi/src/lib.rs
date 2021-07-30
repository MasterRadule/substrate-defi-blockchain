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
    /// The deposit balance of the account after last adjustment
    deposit_principal: Balance,
    /// The time (block height) at which the deposit balance was last adjusted
    deposit_date: BlockNumber,
    /// The borrowing balance of the account after last adjustment
    borrow_principal: Balance,
    /// The time (block height) at which the borrowing balance was last adjusted
    borrow_date: BlockNumber,
}

#[derive(Encode, Decode, Default, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BorrowingInfo<Balance> {
    /// The borrowing balance
    borrowing_balance: Balance,
    /// User's allowed borrowing amount
    allowed_borrowing_amount: Balance,
}

#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::*;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*, PalletId};
    use frame_support::traits::{Currency, ReservableCurrency, ExistenceRequirement};
    use frame_support::sp_runtime::traits::{Saturating, Zero, One};
    use frame_support::sp_runtime::sp_std::convert::TryInto;
    use frame_support::sp_runtime::{FixedU128, FixedPointNumber, SaturatedConversion};
    use sp_runtime::traits::AccountIdConversion;
    use pallet_defi_rpc_runtime_api::BalanceInfo;
    use crate::{AddressInfo, BorrowingInfo};

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
    #[pallet::metadata(AccountIdOf <T> = "AccountId", BalanceOf <T> = "Balance", T::BlockNumber = "BlockNumber")]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Funds deposited. [who, amount, block]
        Deposited(AccountIdOf<T>, BalanceOf<T>, T::BlockNumber),
        /// Funds withdrawn. [who, amount, block]
        Withdrawn(AccountIdOf<T>, BalanceOf<T>, T::BlockNumber),
        /// Loan repaid. [who, amount, block]
        LoanRepaid(AccountIdOf<T>, BalanceOf<T>, T::BlockNumber),
        /// Funds borrowed. [who, amount, block]
        Borrowed(AccountIdOf<T>, BalanceOf<T>, T::BlockNumber),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// User has no deposited balance
        NoFundsDeposited,
        /// Pallet has not enough funds to pay the user
        PalletHasNotEnoughFunds,
        /// User has not as much funds as he asked for
        UserHasNotEnoughFunds,
        /// Repay amount greater than borrowed amount
        RepayOverflow,
        /// Unallowed borrow amount
        UnallowedBorrowAmount,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Deposit funds
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn deposit(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let user = ensure_signed(origin)?;

            // Get address info of extrinsic caller
            let mut address_info = <Accounts<T>>::get(&user);

            // Get current block
            let current_block = frame_system::Pallet::<T>::block_number();

            // Deposit funds to pallet
            T::Currency::transfer(
                &user,
                &Self::account_id(),
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            // Set address info
            address_info.deposit_principal += amount;
            address_info.deposit_date = current_block;

            // Put updated address info into storage
            <Accounts<T>>::insert(&user, address_info);

            // Emit an event
            Self::deposit_event(Event::Deposited(user, amount, current_block));

            // Return a successful DispatchResult
            Ok(())
        }

        /// Withdraw funds
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn withdraw(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let user = ensure_signed(origin)?;

            // Get address info of extrinsic caller and check if it has deposited funds
            let mut address_info = <Accounts<T>>::get(&user);
            ensure!(address_info.deposit_principal != <BalanceOf<T>>::zero(), Error::<T>::NoFundsDeposited);

            // Check if user and pallet have enough funds
            let balance_info = Self::get_balance(user.clone());
            ensure!(amount <= balance_info.balance, Error::<T>::UserHasNotEnoughFunds);
            ensure!(amount <= T::Currency::free_balance(&Self::account_id()), Error::<T>::PalletHasNotEnoughFunds);

            // Get current block
            let current_block = frame_system::Pallet::<T>::block_number();

            // Withdraw funds from pallet
            T::Currency::transfer(
                &Self::account_id(),
                &user,
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            // Update address info
            address_info.deposit_principal -= amount;
            address_info.deposit_date = current_block;

            // Put updated address info into storage
            <Accounts<T>>::insert(&user, address_info);

            // Emit an event
            Self::deposit_event(Event::Withdrawn(user, amount, current_block));

            // Return a successful DispatchResult
            Ok(())
        }

        /// Borrow funds
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn borrow(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let user = ensure_signed(origin)?;

            // Check if pallet has enough funds
            ensure!(amount <= T::Currency::free_balance(&Self::account_id()), Error::<T>::PalletHasNotEnoughFunds);

            // Get address info of extrinsic caller
            let mut address_info = <Accounts<T>>::get(&user);

            // Get allowed borrowing amount
            let borrowing_info = Self::get_allowed_borrowing_amount(user.clone(), &address_info);
            ensure!(amount <= borrowing_info.allowed_borrowing_amount, Error::<T>::UnallowedBorrowAmount);

            // Get current block
            let current_block = frame_system::Pallet::<T>::block_number();

            // Borrow funds from pallet
            T::Currency::transfer(
                &Self::account_id(),
                &user,
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            // Update address info
            address_info.borrow_principal = borrowing_info.borrowing_balance + amount;
            address_info.borrow_date = current_block;

            // Put updated address info into storage
            <Accounts<T>>::insert(&user, address_info);

            // Emit an event
            Self::deposit_event(Event::Borrowed(user, amount, current_block));

            // Return a successful DispatchResult
            Ok(())
        }

        /// Repay loan
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn repay(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            let user = ensure_signed(origin)?;

            // Get address info of extrinsic caller
            let mut address_info = <Accounts<T>>::get(&user);

            // Check if there is repay overflow
            let balance_info = Self::get_loan(user.clone());
            ensure!(amount <= balance_info.balance, Error::<T>::RepayOverflow);

            // Get principal with accrued interest
            let current_block = frame_system::Pallet::<T>::block_number();

            // Transfer funds from user to pallet
            T::Currency::transfer(
                &user,
                &Self::account_id(),
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            // Update address info
            address_info.borrow_principal = balance_info.balance - amount;
            address_info.borrow_date = current_block;

            // Put updated address info into storage
            <Accounts<T>>::insert(&user, address_info);

            // Emit an event
            Self::deposit_event(Event::LoanRepaid(user, amount, current_block));

            // Return a successful DispatchResult
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get user's balance
        pub fn get_balance(user: T::AccountId) -> BalanceInfo<BalanceOf<T>> {
            // Get address info and check if deposit principal is zero
            let address_info = <Accounts<T>>::get(user);
            if address_info.deposit_principal == <BalanceOf<T>>::zero() {
                return BalanceInfo { balance: <BalanceOf<T>>::zero() };
            }

            // Calculate principal with accrued interest
            let current_block = frame_system::Pallet::<T>::block_number();
            let balance = Self::get_principal_with_accrued_interest(current_block, address_info.deposit_date, address_info.deposit_principal);

            return BalanceInfo { balance };
        }

        /// Get user's loan
        pub fn get_loan(user: T::AccountId) -> BalanceInfo<BalanceOf<T>> {
            // Get address info and check if borrow principal is zero
            let address_info = <Accounts<T>>::get(user);
            if address_info.borrow_principal == <BalanceOf<T>>::zero() {
                return BalanceInfo { balance: <BalanceOf<T>>::zero() };
            }

            // Calculate elapsed blocks
            let current_block = frame_system::Pallet::<T>::block_number();
            let balance = Self::get_principal_with_accrued_interest(current_block, address_info.borrow_date, address_info.borrow_principal);

            return BalanceInfo { balance };
        }

        /// Get user's allowed borrowing amount
        pub fn get_allowed_borrowing_amount(user: T::AccountId, address_info: &AddressInfo<BalanceOf<T>, T::BlockNumber>) -> BorrowingInfo<BalanceOf<T>> {
            // Get borrowing balance and deposit principal
            let borrowing_info = Self::get_loan(user.clone());
            let deposit_principal = FixedU128::from_inner(address_info.deposit_principal.saturated_into::<u128>());
            let borrowing_balance = FixedU128::from_inner(borrowing_info.balance.saturated_into::<u128>());

            // Calculate allowed borrowing amount
            let allowed_borrowing_amount = (deposit_principal.saturating_mul(FixedU128::from_inner(75) / FixedU128::from_inner(100))
                .saturating_sub(borrowing_balance)).into_inner().saturated_into();

            return BorrowingInfo { borrowing_balance: borrowing_info.balance, allowed_borrowing_amount };
        }

        /// Get principal with accrued interest
        fn get_principal_with_accrued_interest(current_block: T::BlockNumber, date: T::BlockNumber, principal: BalanceOf<T>) -> BalanceOf<T> {
            // Calculate elapsed blocks
            let elapsed_time_block_number = current_block - date;
            let elapsed_time: u32 = TryInto::try_into(elapsed_time_block_number)
                .ok()
                .expect("blockchain will not exceed 2^32 blocks; qed");

            // Calculate principal with accrued interest
            let rate = FixedU128::from_inner(5) / FixedU128::from_inner(100);
            let multiplier = (FixedU128::one() + rate).saturating_pow(elapsed_time as usize).saturating_sub(FixedU128::one());
            let principal_fixed = FixedU128::from_inner(principal.saturated_into::<u128>());

            return principal_fixed.saturating_add(principal_fixed.saturating_mul(multiplier)).into_inner().saturated_into();
        }

        /// The account ID of pallet
        fn account_id() -> T::AccountId {
            PALLET_ID.into_account()
        }
    }
}
