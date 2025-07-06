use soroban_sdk::{contract, contractimpl, Address, Env, token, vec, Vec, panic_with_error};
use crate::{
    blend,
    swap::SwapOperations,
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

    /// Flash loan receiver
    pub fn exec_op(
        env: Env,
        caller: Address,
        token: Address,
        amount: i128,
        fee: i128,
    ) {
        // Require the caller to authorize the invocation
        caller.require_auth();

        // Load config once
        let config = get_config(&env);

        // Also require owner authorization for safety
        config.owner.require_auth();

        let current_contract = env.current_contract_address();

        if token == config.collateral_asset {
            // LEVERAGE UP: Received collateral via flash loan
            Self::handle_leverage_up(
                &env,
                &config,
                &caller,
                amount,
                fee,
            );
        } else if token == config.debt_asset {
            // DELEVERAGE: Received debt token via flash loan
            Self::handle_deleverage(
                &env,
                &config,
                &caller,
                amount,
                fee,
            );
        } else {
            panic_with_error!(e, LeverageError::BadRequest);
        }

        // Send tokens back to repay flash loan
        let token_client = token::Client::new(&env, &token);
        let repay_amount = amount + fee;
        token_client.transfer(&current_contract, &caller, &repay_amount);
    }

    /// Claims rewards from Blend (similar to harvest in blend strategy)
    pub fn claim(env: Env, from: Address) -> i128 {
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

        rewards_claimed
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
        let debt_client = token::Client::new(env, &config.debt_asset);

        // Get total collateral balance (user deposit + flash loan)
        let total_collateral = collateral_client.balance(&current_contract);

        // Supply all collateral to Blend
        let positions = blend::deposit(
            env,
            config,
            &current_contract,
            total_collateral,
        ).unwrap_optimized();

        // Calculate how much debt token we can borrow (85% of collateral value)
        // Assuming 1:1 price ratio for simplicity - in production use an oracle
        let max_borrow = (total_collateral * 8500) / 10000;

        // Borrow debt tokens from Blend
        blend::borrow(
            env,
            config,
            &current_contract,
            &current_contract,
            max_borrow,
        ).unwrap_optimized();

        // Now we have debt tokens, need to swap to collateral tokens to repay flash loan
        let required_collateral = flash_amount + fee;

        // Approve router to spend debt tokens
        debt_client.approve(
            &current_contract,
            &config.swap_router,
            &max_borrow,
            &(env.ledger().sequence() + 1000)
        );

        // Swap debt tokens for collateral tokens to repay flash loan
        let swap_ops = SwapOperations::new(env, config);
        swap_ops.swap_exact_out(
            required_collateral,  // We need exactly this amount of collateral
            max_borrow,          // Maximum debt tokens we're willing to spend
            &config.debt_asset,
            &config.collateral_asset,
            &current_contract,
        ).unwrap_optimized();
    }

    fn handle_deleverage(
        env: &Env,
        config: &Config,
        caller: &Address,
        flash_amount: i128,
        fee: i128,
    ) {
        let current_contract = env.current_contract_address();
        let collateral_client = token::Client::new(env, &config.collateral_asset);
        let debt_client = token::Client::new(env, &config.debt_asset);

        // Repay debt with flash loaned tokens
        let positions_after_repay = blend::repay(
            env,
            config,
            &current_contract,
            flash_amount,
        ).unwrap_optimized();

        // Calculate how much collateral to withdraw
        // Get remaining debt
        let remaining_debt = positions_after_repay.liabilities
            .iter()
            .find(|l| l.asset == config.debt_asset)
            .map(|l| l.amount)
            .unwrap_or(0);

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
            ).unwrap_optimized();
        }

        // Now we have collateral tokens, need to swap some to debt tokens to repay flash loan
        let required_debt = flash_amount + fee;

        // Calculate how much collateral we need to swap
        let swap_ops = SwapOperations::new(env, config);
        let collateral_needed_for_swap = swap_ops.get_amount_in(
            required_debt,
            &config.collateral_asset,
            &config.debt_asset,
        ).unwrap_optimized();

        // Add slippage buffer
        let collateral_to_swap = SwapOperations::calculate_max_amount_in(
            collateral_needed_for_swap,
            50  // 0.5% slippage
        );

        // Approve router to spend collateral tokens
        collateral_client.approve(
            &current_contract,
            &config.swap_router,
            &collateral_to_swap,
            &(env.ledger().sequence() + 1000)
        );

        // Swap collateral for debt tokens to repay flash loan
        swap_ops.swap_exact_out(
            required_debt,        // We need exactly this amount of debt tokens
            collateral_to_swap,   // Maximum collateral we're willing to spend
            &config.collateral_asset,
            &config.debt_asset,
            &current_contract,
        ).unwrap_optimized();

        // Transfer remaining collateral to user
        let final_collateral_balance = collateral_client.balance(&current_contract);
        if final_collateral_balance > 0 {
            collateral_client.transfer(&current_contract, caller, &final_collateral_balance);
        }
    }
}