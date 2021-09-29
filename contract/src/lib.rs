use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen};
use near_sdk::collections::UnorderedMap;

near_sdk::setup_alloc!();

#[derive(BorshDeserialize, BorshSerialize)]
pub enum Status {
    VALID = 0x00,
    DEACTIVATED = 0x01,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Status is used to store the state of account. There are two states of account, valid and invalid.
    pub status: UnorderedMap<String, Status>,
    /// Used to store the creation time of account
    pub created: UnorderedMap<String, u64>,
    /// Used to store the update time of account
    pub updated: UnorderedMap<String, u64>,

    /// Used to store the books records
    pub records: UnorderedMap<String, u64>,
    /// Used to store the user files
    pub lists:  UnorderedMap<String,Vec<String>>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            status: UnorderedMap::new(b"r".to_vec()),
            created: UnorderedMap::new(b"j".to_vec()),
            updated: UnorderedMap::new(b"h".to_vec()),
            records: UnorderedMap::new(b"c".to_vec()),
            lists: UnorderedMap::new(b"a".to_vec()),
        }
    }
}

#[near_bindgen]
impl Contract {
    ///reg_account  Sign up for an account with Near ID
    pub fn reg_account(&mut self) {
        let account_id = env::signer_account_id();
        let account = gen_account(&account_id);

        let status = self.status.get(&account);
        assert!(status.is_none());

        self.status.insert(&account, &Status::VALID);
        self.created.insert(&account, &env::block_timestamp());

        let log_message = format!("reg_account: {}", &account);
        env::log(log_message.as_bytes());
    }

    /// deactivate_account  Log out of an account with Near ID
    pub fn deactivate_account(&mut self) {
        let account_id = env::signer_account_id();
        let account = gen_account(&account_id);

        let status = self.status.get(&account);
        assert!(status.is_some());

        self.status.insert(&account, &Status::DEACTIVATED);
        self.created.remove(&account);
        self.updated.remove(&account);

        let log_message = format!("deactivate_account: {}", &account);
        env::log(log_message.as_bytes());
    }

    ///record  Record the hash of the user storage file, and record the storage date with the timestamp
    pub fn record(&mut self, file_hash: String) {
        let account_id = env::signer_account_id();
        let account = gen_account(&account_id);
        self.check_account_status(&account);

        let block_timestamp = env::block_timestamp();
        // Use env::log to record logs permanently to the blockchain!
        env::log(format!("Stamping file '{}' at '{}'", file_hash, block_timestamp,).as_bytes());
        //let timestamped_file_hash = env::keccak256(format!("{}{}",file_hash,block_timestamp.to_string()).as_bytes());
        self.records.insert(&file_hash,&block_timestamp);

        let mut list = self.lists.get(&account).unwrap_or(vec![]);
        if list.contains(&file_hash) {
            env::panic(b"record contract, file hash exists")
        };
        list.push(file_hash);
        self.lists.insert(&account,&list);
        self.updated.insert(&account, &env::block_timestamp());
    }

    ///get_record Obtain the timestamp stored by the user using the file hash
    pub fn get_record(&self, file_hash: String) -> u64 {
        match self.records.get(&file_hash) {
            Some(stamp) => stamp.clone(),
            None => 0,
        }
    }

    ///get_lists Gets a list of all file stores
    pub fn get_lists(&self) -> Vec<String> {
        let account_id = env::signer_account_id();
        let account = gen_account(&account_id);
        self.check_account_status(&account);

        match self.lists.get(&account) {
            Some(list) => list.clone(),
            None => vec![],
        }
    }

    fn check_account_status(&self, account: &String) {
        let status = self.status.get(account).unwrap();
        match status {
            Status::VALID => (),
            _ => env::panic(b"account status is not valid"),
        };
    }
}

pub fn gen_account(account_id: &str) -> String {
    String::from("books:") + account_id
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool, block_timestamp:u64) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn reg_account() {
        let context = get_context(vec![], true,block_timestamp);
        testing_env!(context);
        let contract = ProofOfTimestamp::default();
        contract.reg_account();
    }

    #[test]
    fn deactivate_account() {
        let context = get_context(vec![], true,block_timestamp);
        testing_env!(context);
        let contract = ProofOfTimestamp::default();
        contract.deactivate_account();
    }

    #[test]
    fn record() {
        let context = get_context(vec![], true,block_timestamp);
        testing_env!(context);

        let file_hash = "sample file hash".to_string();
        contract.record(file_hash);
    }

    #[test]
    fn get_record() {
        let context = get_context(vec![], true,block_timestamp);
        testing_env!(context);
        let contract = ProofOfTimestamp::default();
        assert_eq!(
            contract.get_record(file_hash),
            block_timestamp
        );
    }

    #[test]
    fn get_lists() {
        let context = get_context(vec![], true,block_timestamp);
        testing_env!(context);
        let file_hash = "sample file hash".to_string();
        let contract = ProofOfTimestamp::default();
        assert_eq!(
            file_hash,
            contract.get_lists("howdy".to_string())
        );
    }
}