use soroban_sdk::{Address, Env, contracttype};
use crate::errors::LeverageError;

#[derive(Clone)]
#[contracttype]
pub struct Config {
    pub owner: Address,
    pub blend_pool: Address,
    pub collateral_asset: Address,
    pub debt_asset: Address,
    pub reward_token: Address,
    pub swap_router: Address,
    pub target_c_factor: i128,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Config,
}

pub fn set_config(e: &Env, config: &Config) {
    e.storage().instance().set(&DataKey::Config, config);
}

pub fn get_config(e: &Env) -> Config {
    e.storage()
        .instance()
        .get(&DataKey::Config)
        .unwrap_optimized()
}