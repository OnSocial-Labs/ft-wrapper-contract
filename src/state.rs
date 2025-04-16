use near_sdk::{AccountId};
use near_sdk::store::{IterableSet, Vector, LookupMap};
use near_sdk::json_types::U128;
use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize};
use near_sdk_macros::NearSchema;
use crate::errors::FtWrapperError;
use crate::types::StorageBalance;

#[derive(BorshSerialize, BorshDeserialize, NearSchema)]
#[abi(borsh)]
pub struct FtWrapperContractState {
    pub admins: Vector<AccountId>,
    pub relayer_contract: AccountId,
    pub supported_tokens: IterableSet<AccountId>,
    pub storage_deposit: U128,
    pub paused: bool,
    pub cross_contract_gas: u64,
    pub storage_balances: LookupMap<(AccountId, AccountId), StorageBalance>,
}

impl FtWrapperContractState {
    pub fn new(admins: Vec<AccountId>, relayer_contract: AccountId, storage_deposit: U128) -> Self {
        let mut admin_vec = Vector::new(b"a".to_vec());
        for admin in admins {
            admin_vec.push(admin);
        }
        Self {
            admins: admin_vec,
            relayer_contract,
            supported_tokens: IterableSet::new(b"t".to_vec()),
            storage_deposit,
            paused: false,
            cross_contract_gas: 100_000_000_000_000,
            storage_balances: LookupMap::new(b"s".to_vec()),
        }
    }

    pub fn is_admin(&self, account_id: &AccountId) -> bool {
        self.admins.iter().any(|admin| admin == account_id)
    }

    pub fn assert_not_paused(&self) -> Result<(), FtWrapperError> {
        if self.paused {
            Err(FtWrapperError::ContractPaused)
        } else {
            Ok(())
        }
    }
}