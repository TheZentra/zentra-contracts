#![cfg(test)]

use crate::StreamTokenClient;

use soroban_sdk::{Address, Env};

pub fn register_test_contract(e: &Env) -> Address {
    e.register_contract(None, crate::StreamToken {})
}

pub struct StreamToken {
    env: Env,
    contract_id: Address,
}

impl StreamToken {
    #[must_use]
    pub fn client(&self) -> StreamTokenClient {
        StreamTokenClient::new(&self.env, &self.contract_id)
    }

    #[must_use]
    pub fn new(env: &Env, contract_id: Address) -> Self {
        Self {
            env: env.clone(),
            contract_id,
        }
    }
}
