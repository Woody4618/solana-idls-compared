use borsh::{BorshDeserialize, BorshSerialize};
use codama::CodamaAccount;

#[derive(CodamaAccount, BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    pub count: u64,
}
