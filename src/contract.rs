use soroban_sdk::{contract, contractimpl, Address, Env, token, vec, Vec, panic_with_error};
use crate::{
    blend,
    swap,
    errors::LeverageError,
    storage::{Config, set_config, get_config},
};

#[contract]
pub struct LeverageContract;

#[contractimpl]
impl LeverageContract {
    /// Initializes the leverage contract
    pub fn __constructor(
        env: Env,
        owner: Address,
        blend_pool: Address,
        collateral_asset: Address,
        debt_asset: Address,
        reward_token: Address,
        swap_router: Address,
        target_c_factor: i128,
    ) {
        let config = Config {
            owner,
            blend_pool,
            collateral_asset,
            debt_asset,
            reward_token,
            swap_router,
            target_c_factor,
        };
        set_config(&env, &config);
    }

    /// Flash loan receiver - exact signature as required
    pub fn exec_op(
        env: Env,
        caller: Address,
        token: Address,
        amount: i128,
        fee: i128,
    ) {
        caller.require_auth();
        let config = get_config(&env);

        // Ensure the owner authorizes this operation
        config.owner.require_auth();
        let current_contract = env.current_contract_address();

        if token == config.collateral_asset {
            // LEVERAGE UP: Received collateral via flash loan
            Self::handle_leverage_up(
                &env,
                &config,
                amount,
                fee,
            );
        } else if token == config.debt_asset {
            // DELEVERAGE: Received debt token via flash loan
            Self::handle_deleverage(
                &env,
                &config,
                amount,
                fee,
            );
        } else {
            panic_with_error!(&env, LeverageError::BadRequest);
        }

        // Send tokens back to repay flash loan
        let token_client = token::Client::new(&env, &token);
        let repay_amount = amount + fee;
        token_client.transfer(&current_contract, &caller, &repay_amount);
    }

    /// Claims rewards from Blend (similar to harvest in blend strategy)
    pub fn claim(env: Env, from: Address) -> Result<(), LeverageError> {
        from.require_auth();

        let config = get_config(&env);
        config.owner.require_auth();

        let current_contract = env.current_contract_address();

        // Claim rewards
        let rewards_claimed = blend::claim(
            &env,
            &config,
            &current_contract,
            &vec![&env, 0u32, 1u32],
            &current_contract
        );

        if rewards_claimed > 0 {
            // Transfer rewards to the caller
            let reward_client = token::Client::new(&env, &config.reward_token);
            reward_client.transfer(&current_contract, &from, &rewards_claimed);
        }

        Ok(())
    }

    // Internal helper functions

    fn handle_leverage_up(
        env: &Env,
        config: &Config,
        flash_amount: i128,
        fee: i128,
    ) {
        let current_contract = env.current_contract_address();
        let collateral_client = token::Client::new(env, &config.collateral_asset);
        // Get total collateral balance (user deposit + flash loan)
        let total_collateral = collateral_client.balance(&current_contract);

        // Supply all collateral to Blend
        let _ = blend::deposit(
            env,
            config,
            &current_contract,
            total_collateral,
        );

        //TODO: Calculate how much to borrow based on target c-factor
        let max_borrow = 0;

        // Borrow debt tokens from Blend
        blend::borrow(
            env,
            config,
            &current_contract,
            &current_contract,
            max_borrow,
        );

        // Now we have debt tokens, need to swap to collateral tokens to repay flash loan
        let required_collateral = flash_amount + fee;

        // Swap debt tokens for collateral tokens to repay flash loan
        let path = vec![env, config.debt_asset.clone(), config.collateral_asset.clone()];
        let amounts = swap::swap_exact_tokens_for_tokens(
            env,
            config,
            max_borrow,
            required_collateral, // We need at least this amount
            path,
            &current_contract,
        );

        // Verify we got enough collateral
        let collateral_received = amounts.get(1).unwrap_or(0);
        if collateral_received < required_collateral {
            panic_with_error!(env, LeverageError::BadRequest);
        }
    }

    fn handle_deleverage(
        env: &Env,
        config: &Config,
        flash_amount: i128,
        fee: i128,
    ) {
        let current_contract = env.current_contract_address();
        let collateral_client = token::Client::new(env, &config.collateral_asset);

        // Repay debt with flash loaned tokens
        let positions_after_repay = blend::repay(
            env,
            config,
            &current_contract,
            flash_amount,
        );

        //TODO:
        // Calculate remaining debt after repayment
        let remaining_debt = positions_after_repay.debt.get(0).unwrap_or(0);

        let withdraw_amount = if remaining_debt == 0 {
            // No debt left, withdraw all collateral
            positions_after_repay.supply.get(0).unwrap_or(0)
        } else {
            // Calculate withdrawal to maintain target c-factor
            let current_collateral = positions_after_repay.supply.get(0).unwrap_or(0);
            let required_collateral = (remaining_debt * config.target_c_factor) / 10000;

            if current_collateral > required_collateral {
                current_collateral - required_collateral
            } else {
                0
            }
        };

        if withdraw_amount > 0 {
            // Withdraw collateral
            blend::withdraw(
                env,
                config,
                &current_contract,
                &current_contract,
                withdraw_amount,
            );
        }

        // Now we have collateral tokens, need to swap some to debt tokens to repay flash loan
        let required_debt = flash_amount + fee;

        // Calculate how much collateral we need to swap
        let path = vec![env, config.collateral_asset.clone(), config.debt_asset.clone()];
        let amounts_in = swap::get_amounts_in(env, config, required_debt, path.clone());
        let collateral_needed = amounts_in.get(0).unwrap_or(0);

        // Add slippage buffer
        let collateral_to_swap = swap::calculate_max_amount_in(collateral_needed, 50); // 0.5% slippage

        // Swap collateral for debt tokens to repay flash loan
        swap::swap_exact_tokens_for_tokens(
            env,
            config,
            collateral_to_swap,
            required_debt, // Minimum we need
            path,
            &current_contract,
        );

        // Transfer remaining collateral to owner (not caller)
        let final_collateral_balance = collateral_client.balance(&current_contract);
        if final_collateral_balance > 0 {
            collateral_client.transfer(&current_contract, &config.owner, &final_collateral_balance);
        }
    }
}