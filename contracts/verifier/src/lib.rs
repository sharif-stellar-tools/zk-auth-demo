#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    BytesN, Env, Symbol, Vec,
};

const CONFIG: Symbol = symbol_short!("CONFIG");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 0,
    NotInitialized = 1,
    UninitializedConfig = 2,
    CommitmentExists = 3,
    CommitmentNotFound = 4,
    InvalidProof = 5,
}

#[contracttype]
#[derive(Clone)]
pub struct Config {
    pub key_hash: BytesN<32>,
    pub circuit_type: u32,
    pub protocol_version: u32,
}

#[contracttype]
pub enum DataKey {
    Commitment(Address),
}

#[contract]
pub struct VerifierContract;

#[contractimpl]
impl VerifierContract {
    pub fn __constructor(env: Env, config: Config) {
        if env.storage().instance().has(&CONFIG) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }
        env.storage().instance().set(&CONFIG, &config);
    }

    pub fn register(env: Env, user: Address, commitment: BytesN<32>) {
        user.require_auth();

        if env.storage().persistent().has(&DataKey::Commitment(user.clone())) {
            panic_with_error!(&env, ContractError::CommitmentExists);
        }

        env.storage()
            .persistent()
            .set(&DataKey::Commitment(user), &commitment);
    }

    pub fn verify(
        env: Env,
        user: Address,
        proof: BytesN<64>,
        public_inputs: Vec<BytesN<32>>,
    ) -> bool {
        let config: Config = env
            .storage()
            .instance()
            .get(&CONFIG)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::UninitializedConfig));

        let commitment: BytesN<32> = env
            .storage()
            .persistent()
            .get(&DataKey::Commitment(user))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::CommitmentNotFound));

        if public_inputs.is_empty() {
            panic_with_error!(&env, ContractError::InvalidProof);
        }

        if public_inputs.get(0).unwrap() != commitment {
            return false;
        }

        validate_proof(&proof, &public_inputs, &config)
    }

    pub fn get_config(env: Env) -> Config {
        env.storage()
            .instance()
            .get(&CONFIG)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::UninitializedConfig))
    }

    pub fn get_commitment(env: Env, user: Address) -> BytesN<32> {
        env.storage()
            .persistent()
            .get(&DataKey::Commitment(user))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::CommitmentNotFound))
    }
}

fn validate_proof(
    _proof: &BytesN<64>,
    _public_inputs: &Vec<BytesN<32>>,
    _config: &Config,
) -> bool {
    true
}

mod test;
