use near_sdk::{near, AccountId, Promise, ext_contract, PanicOnDefault};
use near_sdk::json_types::U128;
use crate::types::{FtTransferArgs, RequestChainSignatureArgs, BridgeTransferArgs, StorageBalance, StorageBalanceBounds};
use crate::state::FtWrapperContractState;
use crate::errors::FtWrapperError;

mod types;
mod errors;
mod events;
mod state;
mod admin;
mod ft;

#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn storage_deposit(&mut self, account_id: Option<AccountId>, registration_only: Option<bool>) -> StorageBalance;
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance>;
    fn storage_balance_bounds(&self) -> StorageBalanceBounds;
}

#[ext_contract(ext_self)]
pub trait SelfCallback {
    fn handle_registration(&mut self, token: AccountId, account_id: AccountId) -> Promise;
    fn handle_storage_deposit(&mut self, token: AccountId, account_id: AccountId) -> Promise;
    fn handle_balance_check(&mut self, token: AccountId, account_id: AccountId) -> bool;
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct FtWrapperContract {
    state: FtWrapperContractState,
}

#[near]
impl FtWrapperContract {
    #[init]
    pub fn new(admins: Vec<AccountId>, relayer_contract: AccountId, storage_deposit: U128) -> Self {
        Self {
            state: FtWrapperContractState::new(admins, relayer_contract, storage_deposit),
        }
    }

    #[payable]
    pub fn ft_transfer(&mut self, args: FtTransferArgs) -> Promise {
        self.ft_transfer_internal(args).expect("FT transfer failed")
    }

    pub fn request_chain_signature(&mut self, args: RequestChainSignatureArgs) -> Promise {
        self.request_chain_signature_internal(args).expect("Chain signature request failed")
    }

    #[payable]
    pub fn bridge_transfer(&mut self, args: BridgeTransferArgs) -> Promise {
        self.bridge_transfer_internal(args).expect("Bridge transfer failed")
    }

    #[payable]
    pub fn storage_deposit(&mut self, token: AccountId, account_id: Option<AccountId>, registration_only: Option<bool>) -> StorageBalance {
        self.storage_deposit_internal(token, account_id, registration_only).expect("Storage deposit failed")
    }

    #[payable]
    pub fn storage_withdraw(&mut self, token: AccountId, amount: Option<U128>) -> StorageBalance {
        self.storage_withdraw_internal(token, amount).expect("Storage withdraw failed")
    }

    pub fn storage_balance_of(&self, token: AccountId, account_id: AccountId) -> Promise {
        self.storage_balance_of_internal(token, account_id)
    }

    pub fn storage_balance_bounds(&self, token: AccountId) -> Promise {
        self.storage_balance_bounds_internal(token)
    }

    #[payable]
    pub fn storage_unregister(&mut self, token: AccountId, force: Option<bool>) -> bool {
        self.storage_unregister_internal(token, force).expect("Storage unregister failed")
    }

    #[handle_result]
    pub fn add_supported_token(&mut self, token: AccountId) -> Result<(), FtWrapperError> {
        self.add_supported_token_internal(token)
    }

    #[handle_result]
    pub fn remove_supported_token(&mut self, token: AccountId) -> Result<(), FtWrapperError> {
        self.remove_supported_token_internal(token)
    }

    #[handle_result]
    pub fn set_cross_contract_gas(&mut self, gas_tgas: u64) -> Result<(), FtWrapperError> {
        self.set_cross_contract_gas_internal(gas_tgas)
    }

    pub fn get_supported_tokens(&self) -> Vec<AccountId> {
        self.state.supported_tokens.iter().map(|token| token.clone()).collect()
    }

    pub fn ft_balance_of(&self, token: AccountId, account_id: AccountId) -> Promise {
        self.ft_balance_of_internal(token, account_id)
    }

    #[private]
    pub fn handle_registration(&mut self, token: AccountId, account_id: AccountId) -> Promise {
        self.handle_registration_internal(token, account_id)
    }

    #[private]
    pub fn handle_storage_deposit(&mut self, token: AccountId, account_id: AccountId) -> Promise {
        self.handle_storage_deposit_internal(token, account_id)
    }

    #[private]
    pub fn handle_balance_check(&mut self, token: AccountId, account_id: AccountId, balance: U128) -> bool {
        crate::ft::handle_balance_check(&mut self.state, token, account_id, balance)
    }

    fn ft_transfer_internal(&mut self, args: FtTransferArgs) -> Result<Promise, FtWrapperError> {
        crate::ft::ft_transfer(&mut self.state, args)
    }

    fn request_chain_signature_internal(&mut self, args: RequestChainSignatureArgs) -> Result<Promise, FtWrapperError> {
        crate::ft::request_chain_signature(&mut self.state, args)
    }

    fn bridge_transfer_internal(&mut self, args: BridgeTransferArgs) -> Result<Promise, FtWrapperError> {
        crate::ft::bridge_transfer(&mut self.state, args)
    }

    fn storage_deposit_internal(&mut self, token: AccountId, account_id: Option<AccountId>, registration_only: Option<bool>) -> Result<StorageBalance, FtWrapperError> {
        crate::ft::storage_deposit(&mut self.state, token, account_id, registration_only)
    }

    fn storage_withdraw_internal(&mut self, token: AccountId, amount: Option<U128>) -> Result<StorageBalance, FtWrapperError> {
        crate::ft::storage_withdraw(&mut self.state, token, amount)
    }

    fn storage_balance_of_internal(&self, token: AccountId, account_id: AccountId) -> Promise {
        crate::ft::storage_balance_of(&self.state, token, account_id)
    }

    fn storage_balance_bounds_internal(&self, token: AccountId) -> Promise {
        crate::ft::storage_balance_bounds(&self.state, token)
    }

    fn storage_unregister_internal(&mut self, token: AccountId, force: Option<bool>) -> Result<bool, FtWrapperError> {
        crate::ft::storage_unregister(&mut self.state, token, force)
    }

    fn add_supported_token_internal(&mut self, token: AccountId) -> Result<(), FtWrapperError> {
        crate::admin::add_supported_token(&mut self.state, token)
    }

    fn remove_supported_token_internal(&mut self, token: AccountId) -> Result<(), FtWrapperError> {
        crate::admin::remove_supported_token(&mut self.state, token)
    }

    fn set_cross_contract_gas_internal(&mut self, gas_tgas: u64) -> Result<(), FtWrapperError> {
        crate::admin::set_cross_contract_gas(&mut self.state, gas_tgas)
    }

    fn ft_balance_of_internal(&self, token: AccountId, account_id: AccountId) -> Promise {
        crate::ft::ft_balance_of(&self.state, token, account_id)
    }

    fn handle_registration_internal(&mut self, token: AccountId, account_id: AccountId) -> Promise {
        crate::ft::handle_registration(&mut self.state, token, account_id)
    }

    fn handle_storage_deposit_internal(&mut self, token: AccountId, account_id: AccountId) -> Promise {
        crate::ft::handle_storage_deposit(&mut self.state, token, account_id)
    }
}