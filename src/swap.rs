use soroban_sdk::{
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    panic_with_error, vec, Address, Env, IntoVal, Symbol, Vec
};
use soroban_sdk::unwrap::UnwrapOptimized;
use crate::{storage::Config};

/// Swaps exact amount of input tokens for a minimum amount of output tokens
///
/// This is a simplified version matching the blend strategy implementation
pub fn swap_exact_tokens_for_tokens(
    e: &Env,
    config: &Config,
    amount_in: i128,
    amount_out_min: i128,
    path: Vec<Address>,
    to: &Address,
) -> Vec<i128> {
    let deadline = e.ledger().timestamp() + 1;

    let swap_args = vec![
        e,
        amount_in.into_val(e),
        amount_out_min.into_val(e),
        path.into_val(e),
        to.to_val(),
        deadline.into_val(e),
    ];

    // Get the pair address from router
    let pair_address = e.invoke_contract::<Address>(
        &config.swap_router,
        &Symbol::new(&e, "router_pair_for"),
        path.into_val(e),
    );

    // Authorize token transfer to pair
    e.authorize_as_current_contract(vec![
        &e,
        InvokerContractAuthEntry::Contract(SubContractInvocation {
            context: ContractContext {
                contract: path.get(0).unwrap_optimized(),
                fn_name: Symbol::new(&e, "transfer"),
                args: (
                    e.current_contract_address(),
                    pair_address,
                    amount_in,
                ).into_val(e),
            },
            sub_invocations: vec![&e],
        }),
    ]);

    // Execute swap
    e.invoke_contract::<Vec<i128>>(
        &config.swap_router,
        &Symbol::new(&e, "swap_exact_tokens_for_tokens"),
        swap_args.into_val(e),
    )
}

/// Helper to get the expected output amount for a given input
pub fn get_amounts_out(
    e: &Env,
    config: &Config,
    amount_in: i128,
    path: Vec<Address>,
) -> Vec<i128> {
    e.invoke_contract::<Vec<i128>>(
        &config.swap_router,
        &Symbol::new(&e, "router_get_amounts_out"),
        vec![e, amount_in.into_val(e), path.into_val(e)].into_val(e),
    )
}

/// Helper to get the required input amount for a desired output
pub fn get_amounts_in(
    e: &Env,
    config: &Config,
    amount_out: i128,
    path: Vec<Address>,
) -> Vec<i128> {
    e.invoke_contract::<Vec<i128>>(
        &config.swap_router,
        &Symbol::new(&e, "router_get_amounts_in"),
        vec![e, amount_out.into_val(e), path.into_val(e)].into_val(e),
    )
}

/// Calculate minimum output with slippage protection
pub fn calculate_min_amount_out(
    amount_out_expected: i128,
    slippage_bps: i128,
) -> i128 {
    let slippage_factor = 10000 - slippage_bps;
    (amount_out_expected * slippage_factor) / 10000
}

/// Calculate maximum input with slippage protection
pub fn calculate_max_amount_in(
    amount_in_expected: i128,
    slippage_bps: i128,
) -> i128 {
    let slippage_factor = 10000 + slippage_bps;
    (amount_in_expected * slippage_factor) / 10000
}