use soroban_sdk::{vec, Address, Env, Vec};
use crate::{errors::LeverageError, storage::Config};

mod blend {
    soroban_sdk::contractimport!(file = "./pool.wasm");
}

pub use blend::Client as PoolClient;
pub use blend::{Request, RequestType, Positions};

/// Deposit collateral to Blend pool
pub fn deposit(
    e: &Env,
    config: &Config,
    from: &Address,
    amount: i128,
) -> Result<Positions, LeverageError> {
    if amount <= 0 {
        return Err(LeverageError::InvalidAmount);
    }

    let pool_client = PoolClient::new(e, &config.blend_pool);
    let request = Request {
        request_type: RequestType::SupplyCollateral as u32,
        address: config.collateral_asset.clone(),
        amount,
    };

    Ok(pool_client.submit(from, from, from, &vec![e, request]))
}

/// Withdraw collateral from Blend pool
pub fn withdraw(
    e: &Env,
    config: &Config,
    from: &Address,
    to: &Address,
    amount: i128,
) -> Result<Positions, LeverageError> {
    if amount <= 0 {
        return Err(LeverageError::InvalidAmount);
    }

    let pool_client = PoolClient::new(e, &config.blend_pool);
    let request = Request {
        request_type: RequestType::WithdrawCollateral as u32,
        address: config.collateral_asset.clone(),
        amount,
    };

    Ok(pool_client.submit(from, from, to, &vec![e, request]))
}

/// Borrow debt asset from Blend pool
pub fn borrow(
    e: &Env,
    config: &Config,
    from: &Address,
    to: &Address,
    amount: i128,
) -> Result<Positions, LeverageError> {
    if amount <= 0 {
        return Err(LeverageError::InvalidAmount);
    }

    let pool_client = PoolClient::new(e, &config.blend_pool);
    let request = Request {
        request_type: RequestType::Borrow as u32,
        address: config.debt_asset.clone(),
        amount,
    };

    Ok(pool_client.submit(from, from, to, &vec![e, request]))
}

/// Repay debt to Blend pool
pub fn repay(
    e: &Env,
    config: &Config,
    from: &Address,
    amount: i128,
) -> Result<Positions, LeverageError> {
    if amount <= 0 {
        return Err(LeverageError::InvalidAmount);
    }

    let pool_client = PoolClient::new(e, &config.blend_pool);
    let request = Request {
        request_type: RequestType::Repay as u32,
        address: config.debt_asset.clone(),
        amount,
    };

    Ok(pool_client.submit(from, from, from, &vec![e, request]))
}

/// Get positions for an address
pub fn get_positions(
    e: &Env,
    config: &Config,
    address: &Address,
) -> Positions {
    let pool_client = PoolClient::new(e, &config.blend_pool);
    pool_client.get_positions(address)
}

/// Claim rewards from the pool
pub fn claim(
    e: &Env,
    config: &Config,
    from: &Address,
    reserve_token_ids: &Vec<u32>,
    to: &Address,
) -> i128 {
    let pool_client = PoolClient::new(e, &config.blend_pool);
    pool_client.claim(from, reserve_token_ids, to)
}