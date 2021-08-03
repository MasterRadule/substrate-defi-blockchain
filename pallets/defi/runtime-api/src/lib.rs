//! Runtime API definition for defi pallet.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::{Codec, Encode, Decode};
#[cfg(feature = "std")]
use common::utils::string_serialization;
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
use sp_std::prelude::*;

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct BalanceInfo<Balance> {
	#[cfg_attr(
		feature = "std",
		serde(
			bound(
				serialize = "Balance: std::fmt::Display",
				deserialize = "Balance: std::str::FromStr"
			),
			with = "string_serialization"
		)
	)]
    pub balance: Balance,
}

#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct BorrowingInfo<Balance> {
	#[cfg_attr(
		feature = "std",
		serde(
			bound(
				serialize = "Balance: std::fmt::Display",
				deserialize = "Balance: std::str::FromStr"
			),
			with = "string_serialization"
		)
	)]
	pub borrowing_balance: Balance,
	#[cfg_attr(
		feature = "std",
		serde(
			bound(
				serialize = "Balance: std::fmt::Display",
				deserialize = "Balance: std::str::FromStr"
			),
			with = "string_serialization"
		)
	)]
	pub allowed_borrowing_amount: Balance,
}

sp_api::decl_runtime_apis! {
	pub trait DefiModuleAPI<AccountId, Balance> where
		AccountId: Codec,
		Balance: Codec + MaybeFromStr + MaybeDisplay,
	{
		fn get_balance(user: AccountId) -> BalanceInfo<Balance>;
		fn get_debt(user: AccountId) -> BalanceInfo<Balance>;
		fn get_allowed_borrowing_amount(user: AccountId) -> BorrowingInfo<Balance>;
		fn get_deposit_apy() -> BalanceInfo<Balance>;
		fn get_borrowing_apy() -> BalanceInfo<Balance>;
	}
}
