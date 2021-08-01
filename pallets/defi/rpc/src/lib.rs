use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use std::sync::Arc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use codec::Codec;
use sp_runtime::traits::{Block as BlockT, MaybeDisplay, MaybeFromStr};
use sp_runtime::generic::BlockId;
use jsonrpc_derive::rpc;

pub use pallet_defi_rpc_runtime_api::DefiModuleAPI as DefiRuntimeAPI;
use pallet_defi_rpc_runtime_api::{BalanceInfo, BorrowingInfo};

#[rpc]
pub trait DefiModuleAPI<
    BlockHash,
    AccountId,
    Balance,
    BalanceType,
    BorrowingType,
>
{
    #[rpc(name = "defiModule_getBalance")]
    fn get_balance(
        &self,
        user: AccountId,
        at: Option<BlockHash>,
    ) -> Result<BalanceType>;

    #[rpc(name = "defiModule_getDebt")]
    fn get_debt(
        &self,
        user: AccountId,
        at: Option<BlockHash>,
    ) -> Result<BalanceType>;

    #[rpc(name = "defiModule_getAllowedBorrowingAmount")]
    fn get_allowed_borrowing_amount(
        &self,
        user: AccountId,
        at: Option<BlockHash>,
    ) -> Result<BorrowingType>;
}

pub struct DefiModuleClient<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> DefiModuleClient<C, B> {
    /// Construct default `Template`.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
        }
    }
}

impl<C, Block, AccountId, Balance> DefiModuleAPI
<
    <Block as BlockT>::Hash,
    AccountId,
    Balance,
    BalanceInfo<Balance>,
    BorrowingInfo<Balance>
> for DefiModuleClient<C, Block>
    where
        Block: BlockT,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: DefiRuntimeAPI<Block, AccountId, Balance>,
        AccountId: Codec,
        Balance: Codec + MaybeFromStr + MaybeDisplay,
{
    fn get_balance(&self, user: AccountId, at: Option<<Block as BlockT>::Hash>) -> Result<BalanceInfo<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or(
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash,
        ));
        api.get_balance(&at, user)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError.into()),
                message: "Unable to get balance.".into(),
                data: Some(format!("{:?}", e).into()),
            })
    }

    fn get_debt(&self, user: AccountId, at: Option<<Block as BlockT>::Hash>) -> Result<BalanceInfo<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or(
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash,
        ));
        api.get_debt(&at, user)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError.into()),
                message: "Unable to get debt.".into(),
                data: Some(format!("{:?}", e).into()),
            })
    }

    fn get_allowed_borrowing_amount(&self, user: AccountId, at: Option<<Block as BlockT>::Hash>) -> Result<BorrowingInfo<Balance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or(
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash,
        ));
        api.get_allowed_borrowing_amount(&at, user)
            .map_err(|e| RpcError {
                code: ErrorCode::ServerError(Error::RuntimeError.into()),
                message: "Unable to get allowed borrowing amount".into(),
                data: Some(format!("{:?}", e).into()),
            })
    }
}
