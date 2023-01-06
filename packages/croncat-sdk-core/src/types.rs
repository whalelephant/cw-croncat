use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw20::Cw20CoinVerified;

use crate::{balancer::RoundRobinBalancer, error::CoreError};

use self::gas_price_defaults::{
    GAS_ADJUSTMENT_NUMERATOR_DEFAULT, GAS_DENOMINATOR, GAS_NUMERATOR_DEFAULT,
};

pub mod gas_price_defaults {
    pub const GAS_NUMERATOR_DEFAULT: u64 = 4;
    pub const GAS_ADJUSTMENT_NUMERATOR_DEFAULT: u64 = 150;
    pub const GAS_DENOMINATOR: u64 = 100;
}


/// We can't store gas_price as floats inside cosmwasm
/// so instead of having 0.04 we use GasPrice {4/100}
/// and after that multiply Gas by `gas_adjustment` {150/100} (1.5)
#[cw_serde]
pub struct GasPrice {
    pub numerator: u64,
    /// Denominator is shared
    pub denominator: u64,
    pub gas_adjustment_numerator: u64,
}

impl GasPrice {
    pub fn is_valid(&self) -> bool {
        self.denominator != 0 && self.numerator != 0 && self.gas_adjustment_numerator != 0
    }

    pub fn calculate(&self, gas_amount: u64) -> Result<u128, CoreError> {
        let gas_adjusted = gas_amount
            .checked_mul(self.gas_adjustment_numerator)
            .and_then(|g| g.checked_div(self.denominator))
            .ok_or(CoreError::InvalidGas {})?;

        let price = gas_adjusted
            .checked_mul(self.numerator)
            .and_then(|g| g.checked_div(self.denominator))
            .ok_or(CoreError::InvalidGas {})?;

        Ok(price as u128)
    }
}

impl Default for GasPrice {
    fn default() -> Self {
        Self {
            numerator: GAS_NUMERATOR_DEFAULT,
            denominator: GAS_DENOMINATOR,
            gas_adjustment_numerator: GAS_ADJUSTMENT_NUMERATOR_DEFAULT,
        }
    }
}

#[cw_serde]
pub struct Config {
    // Runtime
    pub paused: bool,
    pub owner_id: Addr,

    // Agent management
    // The minimum number of tasks per agent
    // Example: 10
    // Explanation: For every 1 agent, 10 tasks per slot are available.
    // NOTE: Caveat, when there are odd number of tasks or agents, the overflow will be available to first-come, first-serve. This doesn't negate the possibility of a failed txn from race case choosing winner inside a block.
    // NOTE: The overflow will be adjusted to be handled by sweeper in next implementation.
    pub min_tasks_per_agent: u64,
    // How many slots an agent can miss before being removed from the active queue
    pub agents_eject_threshold: u64,
    // The duration a prospective agent has to nominate themselves.
    // When a task is created such that a new agent can join,
    // The agent at the zeroth index of the pending agent queue has this time to nominate
    // The agent at the first index has twice this time to nominate (which would remove the former agent from the pending queue)
    // Value is in seconds
    pub agent_nomination_duration: u16,
    pub cw_rules_addr: Addr,
    pub croncat_tasks_addr: Addr,
    pub croncat_agents_addr: Addr,

    // Economics
    pub agent_fee: u64,
    pub gas_price: GasPrice,
    pub gas_base_fee: u64,
    pub gas_action_fee: u64,
    pub gas_query_fee: u64,
    pub gas_wasm_query_fee: u64,
    pub slot_granularity_time: u64,

    // Treasury
    // pub treasury_id: Option<Addr>,
    pub cw20_whitelist: Vec<Addr>, // TODO: Consider fee structure for whitelisted CW20s
    pub native_denom: String,

    pub balancer: RoundRobinBalancer,

    // The default amount of tasks to query
    pub limit: u64,
}

#[cw_serde]
pub struct UpdateConfig {
    pub owner_id: Option<String>,
    pub slot_granularity_time: Option<u64>,
    pub paused: Option<bool>,
    pub agent_fee: Option<u64>,
    pub gas_base_fee: Option<u64>,
    pub gas_action_fee: Option<u64>,
    pub gas_query_fee: Option<u64>,
    pub gas_wasm_query_fee: Option<u64>,
    pub gas_price: Option<GasPrice>,
    pub min_tasks_per_agent: Option<u64>,
    pub agents_eject_threshold: Option<u64>,
    pub balancer: Option<RoundRobinBalancer>,
    // pub treasury_id: Option<String>,
}

#[cw_serde]
pub struct BalancesResponse {
    pub native_denom: String,
    pub available_native_balance: Vec<Coin>,
    pub available_cw20_balance: Vec<Cw20CoinVerified>,
}

#[cfg(test)]
mod test {
    use crate::CoreError;

    use super::GasPrice;

    #[test]
    fn gas_price_validation() {
        assert!(!GasPrice {
            numerator: 0,
            denominator: 1,
            gas_adjustment_numerator: 1
        }
        .is_valid());

        assert!(!GasPrice {
            numerator: 1,
            denominator: 0,
            gas_adjustment_numerator: 1
        }
        .is_valid());

        assert!(!GasPrice {
            numerator: 1,
            denominator: 1,
            gas_adjustment_numerator: 0
        }
        .is_valid());

        assert!(GasPrice {
            numerator: 1,
            denominator: 1,
            gas_adjustment_numerator: 1
        }
        .is_valid());
    }

    #[test]
    fn gas_price_calculate_test() {
        // Test with default values
        let gas_price_wrapper = GasPrice::default();
        let gas_price = 0.04;
        let gas_adjustments = 1.5;

        let gas = 200_000;
        let expected = gas as f64 * gas_adjustments * gas_price;
        assert_eq!(expected as u128, gas_price_wrapper.calculate(gas).unwrap());

        let gas = 160_000;
        let expected = gas as f64 * gas_adjustments * gas_price;
        assert_eq!(expected as u128, gas_price_wrapper.calculate(gas).unwrap());

        let gas = 1_234_000;
        let expected = gas as f64 * gas_adjustments * gas_price;
        assert_eq!(expected as u128, gas_price_wrapper.calculate(gas).unwrap());

        // Check custom works
        let gas_price_wrapper = GasPrice {
            numerator: 25,
            denominator: 100,
            gas_adjustment_numerator: 120,
        };
        let gas_price = 0.25;
        let gas_adjustments = 1.2;

        let gas = 200_000;
        let expected = gas as f64 * gas_adjustments * gas_price;
        assert_eq!(expected as u128, gas_price_wrapper.calculate(gas).unwrap());

        let gas = 160_000;
        let expected = gas as f64 * gas_adjustments * gas_price;
        assert_eq!(expected as u128, gas_price_wrapper.calculate(gas).unwrap());

        let gas = 1_234_000;
        let expected = gas as f64 * gas_adjustments * gas_price;
        assert_eq!(expected as u128, gas_price_wrapper.calculate(gas).unwrap());
    }

    #[test]
    fn failed_gas_calculations() {
        let gas_price_wrapper = GasPrice::default();

        let err = gas_price_wrapper.calculate(u64::MAX).unwrap_err();
        assert!(matches!(err, CoreError::InvalidGas {}));
    }
}
