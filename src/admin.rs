use near_sdk::{env, AccountId};
use crate::state::FtWrapperContractState;
use crate::errors::FtWrapperError;
use crate::events::FtWrapperEvent;

pub fn add_supported_token(state: &mut FtWrapperContractState, token: AccountId) -> Result<(), FtWrapperError> {
    let caller = env::predecessor_account_id();
    if !state.is_admin(&caller) {
        return Err(FtWrapperError::Unauthorized);
    }
    state.supported_tokens.insert(token.clone());
    FtWrapperEvent::TokenAdded { token }.emit();
    Ok(())
}

pub fn remove_supported_token(state: &mut FtWrapperContractState, token: AccountId) -> Result<(), FtWrapperError> {
    let caller = env::predecessor_account_id();
    if !state.is_admin(&caller) {
        return Err(FtWrapperError::Unauthorized);
    }
    if state.supported_tokens.remove(&token) {
        FtWrapperEvent::TokenRemoved { token }.emit();
        Ok(())
    } else {
        Err(FtWrapperError::TokenNotSupported)
    }
}

pub fn set_cross_contract_gas(state: &mut FtWrapperContractState, gas_tgas: u64) -> Result<(), FtWrapperError> {
    let caller = env::predecessor_account_id();
    if !state.is_admin(&caller) {
        return Err(FtWrapperError::Unauthorized);
    }
    state.cross_contract_gas = gas_tgas * 1_000_000_000_000; // Convert TGas to Gas
    FtWrapperEvent::GasUpdated { gas_tgas }.emit();
    Ok(())
}