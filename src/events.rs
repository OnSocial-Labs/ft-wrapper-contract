use near_sdk::{near, AccountId};
use near_sdk::json_types::{U128};

#[near(event_json(standard = "nep297"))]
pub enum FtWrapperEvent {
    #[event_version("1.0.0")]
    TokenAdded { token: AccountId },
    #[event_version("1.0.0")]
    TokenRemoved { token: AccountId },
    #[event_version("1.0.0")]
    FtTransfer { token: AccountId, sender: AccountId, receiver: AccountId, amount: U128 },
    #[event_version("1.0.0")]
    StorageDeposited { token: AccountId, account_id: AccountId, amount: U128 },
    #[event_version("1.0.0")]
    StorageWithdrawn { token: AccountId, account_id: AccountId, amount: U128 },
    #[event_version("1.0.0")]
    StorageUnregistered { token: AccountId, account_id: AccountId },
    #[event_version("1.0.0")]
    GasUpdated { gas_tgas: u64 },
    #[event_version("1.0.0")]
    LowBalance { balance: u128 },
    #[event_version("1.0.0")]
    StorageDepositUpdated { storage_deposit: U128 },
}