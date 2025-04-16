use near_sdk::{env, AccountId, Promise, Gas, NearToken};
use near_sdk::json_types::U128;
use crate::state::FtWrapperContractState;
use crate::types::{FtTransferArgs, RequestChainSignatureArgs, BridgeTransferArgs, StorageBalance};
use crate::errors::FtWrapperError;
use crate::events::FtWrapperEvent;
use crate::{ext_ft, ext_self};

pub fn ft_transfer(state: &mut FtWrapperContractState, args: FtTransferArgs) -> Result<Promise, FtWrapperError> {
    state.assert_not_paused()?;
    if !state.supported_tokens.contains(&args.token) {
        return Err(FtWrapperError::TokenNotSupported);
    }
    if args.amount.0 == 0 {
        return Err(FtWrapperError::AmountTooLow);
    }

    let sender_id = env::predecessor_account_id();
    
    // Batch registration and transfer
    let sender_promise = ensure_registered(state, args.token.clone(), sender_id.clone())?;
    let receiver_promise = ensure_registered(state, args.token.clone(), args.receiver_id.clone())?;
    let transfer_promise = ext_ft::ext(args.token.clone())
        .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
        .ft_transfer(args.receiver_id.clone(), args.amount, args.memo.clone());

    FtWrapperEvent::FtTransfer {
        token: args.token,
        sender: sender_id,
        receiver: args.receiver_id,
        amount: args.amount,
    }.emit();

    Ok(sender_promise.and(receiver_promise).then(transfer_promise))
}

pub fn ft_balance_of(state: &FtWrapperContractState, token: AccountId, account_id: AccountId) -> Promise {
    if !state.supported_tokens.contains(&token) {
        env::panic_str("Token not supported");
    }
    ext_ft::ext(token)
        .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
        .ft_balance_of(account_id)
}

pub fn storage_deposit(
    state: &mut FtWrapperContractState,
    token: AccountId,
    account_id: Option<AccountId>,
    registration_only: Option<bool>,
) -> Result<StorageBalance, FtWrapperError> {
    state.assert_not_paused()?;
    if !state.supported_tokens.contains(&token) {
        return Err(FtWrapperError::TokenNotSupported);
    }

    let account_id = account_id.unwrap_or_else(|| env::predecessor_account_id());
    let registration_only = registration_only.unwrap_or(false);
    let attached_deposit = env::attached_deposit();

    // Check current registration status
    let storage_balance = state.storage_balances.get(&(token.clone(), account_id.clone()));

    if let Some(balance) = storage_balance {
        if attached_deposit.as_yoctonear() == 0 {
            return Ok(balance.clone());
        }
    }

    // Execute deposit asynchronously
    let deposit_amount = if registration_only {
        state.storage_deposit.0
    } else {
        attached_deposit.as_yoctonear().max(state.storage_deposit.0)
    };

    let deposit_promise = ext_ft::ext(token.clone())
        .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
        .with_attached_deposit(NearToken::from_yoctonear(deposit_amount))
        .storage_deposit(Some(account_id.clone()), Some(registration_only))
        .then(
            ext_self::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
                .handle_storage_deposit(token.clone(), account_id.clone()),
        );

    // Chain the promise to ensure execution
    deposit_promise.then(Promise::new(env::current_account_id()));

    FtWrapperEvent::StorageDeposited {
        token,
        account_id,
        amount: U128(deposit_amount),
    }.emit();

    // Refund excess deposit
    if attached_deposit.as_yoctonear() > deposit_amount {
        let refund = attached_deposit.as_yoctonear() - deposit_amount;
        Promise::new(env::predecessor_account_id()).transfer(NearToken::from_yoctonear(refund));
    }

    Ok(StorageBalance {
        total: U128(deposit_amount),
        available: U128(0),
    })
}

pub fn storage_withdraw(
    state: &mut FtWrapperContractState,
    token: AccountId,
    amount: Option<U128>,
) -> Result<StorageBalance, FtWrapperError> {
    state.assert_not_paused()?;
    if !state.supported_tokens.contains(&token) {
        return Err(FtWrapperError::TokenNotSupported);
    }

    let account_id = env::predecessor_account_id();
    let attached_deposit = env::attached_deposit();

    if attached_deposit != NearToken::from_yoctonear(1) {
        return Err(FtWrapperError::InvalidDeposit);
    }

    let storage_balance = state.storage_balances.get(&(token.clone(), account_id.clone()))
        .ok_or(FtWrapperError::AccountNotRegistered)?;

    let available = storage_balance.available.0;
    let withdraw_amount = amount.map(|a| a.0).unwrap_or(available);

    if withdraw_amount > available {
        return Err(FtWrapperError::InsufficientStorageBalance);
    }

    let new_balance = StorageBalance {
        total: storage_balance.total,
        available: U128(available - withdraw_amount),
    };
    state.storage_balances.insert((token.clone(), account_id.clone()), new_balance.clone());

    if withdraw_amount > 0 {
        Promise::new(account_id.clone()).transfer(NearToken::from_yoctonear(withdraw_amount));
    }

    FtWrapperEvent::StorageWithdrawn {
        token,
        account_id,
        amount: U128(withdraw_amount),
    }.emit();

    Ok(new_balance)
}

pub fn storage_balance_of(state: &FtWrapperContractState, token: AccountId, account_id: AccountId) -> Promise {
    if !state.supported_tokens.contains(&token) {
        env::panic_str("Token not supported");
    }
    ext_ft::ext(token)
        .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
        .storage_balance_of(account_id)
}

pub fn storage_balance_bounds(state: &FtWrapperContractState, token: AccountId) -> Promise {
    if !state.supported_tokens.contains(&token) {
        env::panic_str("Token not supported");
    }
    ext_ft::ext(token)
        .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
        .storage_balance_bounds()
}

pub fn storage_unregister(
    state: &mut FtWrapperContractState,
    token: AccountId,
    force: Option<bool>,
) -> Result<bool, FtWrapperError> {
    state.assert_not_paused()?;
    if !state.supported_tokens.contains(&token) {
        return Err(FtWrapperError::TokenNotSupported);
    }

    let account_id = env::predecessor_account_id();
    let attached_deposit = env::attached_deposit();

    if attached_deposit != NearToken::from_yoctonear(1) {
        return Err(FtWrapperError::InvalidDeposit);
    }

    let storage_balance = state.storage_balances.get(&(token.clone(), account_id.clone()));

    if storage_balance.is_none() {
        return Ok(false);
    }

    let force = force.unwrap_or(false);
    let balance = storage_balance.unwrap();

    if !force {
        // Check FT balance before unregistration
        ext_ft::ext(token.clone())
            .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
            .ft_balance_of(account_id.clone())
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
                    .handle_balance_check(token.clone(), account_id.clone()),
            );
        return Ok(false); // Defer to callback
    }

    // Proceed with force unregistration
    if balance.total.0 > 0 {
        Promise::new(account_id.clone()).transfer(NearToken::from_yoctonear(balance.total.0));
    }

    state.storage_balances.remove(&(token.clone(), account_id.clone()));

    FtWrapperEvent::StorageUnregistered { token, account_id }.emit();

    Ok(true)
}

pub fn handle_balance_check(
    state: &mut FtWrapperContractState,
    token: AccountId,
    account_id: AccountId,
    balance: U128,
) -> bool {
    if balance.0 != 0 {
        env::log_str("Non-zero balance detected, unregistration aborted");
        return false;
    }

    let storage_balance = state.storage_balances.get(&(token.clone(), account_id.clone()));
    if let Some(balance) = storage_balance {
        if balance.total.0 > 0 {
            Promise::new(account_id.clone()).transfer(NearToken::from_yoctonear(balance.total.0));
        }
        state.storage_balances.remove(&(token.clone(), account_id.clone()));
        FtWrapperEvent::StorageUnregistered { token, account_id }.emit();
        return true;
    }

    false
}

fn ensure_registered(state: &FtWrapperContractState, token: AccountId, account_id: AccountId) -> Result<Promise, FtWrapperError> {
    if !state.supported_tokens.contains(&token) {
        return Err(FtWrapperError::TokenNotSupported);
    }
    let registration_check = ext_ft::ext(token.clone())
        .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
        .storage_balance_of(account_id.clone())
        .then(
            ext_self::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
                .handle_registration(token.clone(), account_id.clone()),
        );
    Ok(registration_check)
}

pub fn handle_registration(
    state: &mut FtWrapperContractState,
    token: AccountId,
    account_id: AccountId,
) -> Promise {
    let storage_balance = state.storage_balances.get(&(token.clone(), account_id.clone()));
    if storage_balance.is_none() {
        let bounds_promise = ext_ft::ext(token.clone())
            .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
            .storage_balance_bounds()
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
                    .handle_storage_deposit(token.clone(), account_id.clone()),
            );
        bounds_promise
    } else {
        Promise::new(env::current_account_id())
    }
}

pub fn handle_storage_deposit(
    state: &mut FtWrapperContractState,
    token: AccountId,
    account_id: AccountId,
) -> Promise {
    let deposit_amount = state.storage_deposit.0;
    let deposit_promise = ext_ft::ext(token.clone())
        .with_static_gas(Gas::from_tgas(state.cross_contract_gas))
        .with_attached_deposit(NearToken::from_yoctonear(deposit_amount))
        .storage_deposit(Some(account_id.clone()), Some(true));
    
    FtWrapperEvent::StorageDeposited {
        token,
        account_id,
        amount: U128(deposit_amount),
    }.emit();

    deposit_promise
}

pub fn request_chain_signature(state: &mut FtWrapperContractState, args: RequestChainSignatureArgs) -> Result<Promise, FtWrapperError> {
    state.assert_not_paused()?;
    if !state.supported_tokens.contains(&args.token) {
        return Err(FtWrapperError::TokenNotSupported);
    }
    let sender_id = env::predecessor_account_id();
    let promise = ensure_registered(state, args.token.clone(), sender_id)?;
    Ok(promise.then(Promise::new(state.relayer_contract.clone())
        .function_call(
            "relay_meta_transaction".to_string(),
            vec![],
            NearToken::from_yoctonear(0),
            Gas::from_tgas(state.cross_contract_gas),
        )))
}

pub fn bridge_transfer(state: &mut FtWrapperContractState, args: BridgeTransferArgs) -> Result<Promise, FtWrapperError> {
    state.assert_not_paused()?;
    if !state.supported_tokens.contains(&args.token) {
        return Err(FtWrapperError::TokenNotSupported);
    }
    if args.amount.0 == 0 {
        return Err(FtWrapperError::AmountTooLow);
    }
    let sender_id = env::predecessor_account_id();
    let promise = ensure_registered(state, args.token.clone(), sender_id)?;
    Ok(promise.then(Promise::new(state.relayer_contract.clone())
        .function_call(
            "relay_meta_transaction".to_string(),
            vec![],
            env::attached_deposit(),
            Gas::from_tgas(state.cross_contract_gas),
        )))
}