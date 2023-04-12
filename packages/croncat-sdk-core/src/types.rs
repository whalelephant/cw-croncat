use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, StdError, StdResult};
use cw20::Cw20CoinVerified;

use crate::error::SdkError;

/// from_index: Start at the 0 index for retrieving data, unless specified for pagination
pub const DEFAULT_PAGINATION_FROM_INDEX: u64 = 0;
/// limit: will grab a total set of records or the maximum allowed.
pub const DEFAULT_PAGINATION_LIMIT: u64 = 100;

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

    pub fn calculate(&self, gas_amount: u64) -> Result<u128, SdkError> {
        let gas_adjusted = gas_amount
            .checked_mul(self.gas_adjustment_numerator)
            .and_then(|g| g.checked_div(self.denominator))
            .ok_or(SdkError::InvalidGas {})?;

        let price = gas_adjusted
            .checked_mul(self.numerator)
            .and_then(|g| g.checked_div(self.denominator))
            .ok_or(SdkError::InvalidGas {})?;

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
#[derive(Default)]
pub struct AmountForOneTask {
    // Attached balances, used for forwarding during actions
    pub cw20: Option<Cw20CoinVerified>,
    pub coin: [Option<Coin>; 2],

    // to stabilize deposited fees against point-in-time configured fees
    pub gas: u64,
    pub agent_fee: u16,
    pub treasury_fee: u16,
    pub gas_price: GasPrice,
}

impl AmountForOneTask {
    pub fn add_gas(&mut self, gas: u64) {
        self.gas = self.gas.saturating_add(gas);
    }

    pub fn add_coin(&mut self, coin: Coin) -> StdResult<bool> {
        match &mut self.coin {
            [None, None] => {
                self.coin[0] = Some(coin);
                Ok(true)
            }
            [Some(c1), None] => {
                if c1.denom == coin.denom {
                    c1.amount = c1
                        .amount
                        .checked_add(coin.amount)
                        .map_err(|_| StdError::generic_err("Overflow when adding coin 1"))?;
                } else {
                    self.coin[1] = Some(coin);
                }
                Ok(true)
            }
            [Some(c1), Some(c2)] => {
                if c1.denom == coin.denom {
                    c1.amount = c1
                        .amount
                        .checked_add(coin.amount)
                        .map_err(|_| StdError::generic_err("Overflow when adding coin 1"))?;
                    Ok(true)
                } else if c2.denom == coin.denom {
                    c2.amount = c2
                        .amount
                        .checked_add(coin.amount)
                        .map_err(|_| StdError::generic_err("Overflow when adding coin 2"))?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            [None, Some(_)] => unreachable!(),
        }
    }

    pub fn add_cw20(&mut self, cw20: Cw20CoinVerified) -> StdResult<bool> {
        if let Some(cw20_inner) = &mut self.cw20 {
            if cw20_inner.address != cw20.address {
                return Ok(false);
            }
            cw20_inner.amount = cw20_inner
                .amount
                .checked_add(cw20.amount)
                .map_err(|_| StdError::generic_err("Overflow when adding cw20"))?;
        } else {
            self.cw20 = Some(cw20);
        }
        Ok(true)
    }

    pub fn sub_coin(&mut self, coin: &Coin) -> StdResult<()> {
        match &mut self.coin {
            [Some(c1), Some(c2)] => {
                if c1.denom == coin.denom {
                    c1.amount = c1
                        .amount
                        .checked_sub(coin.amount)
                        .map_err(|_| StdError::generic_err("Underflow when subtracting coin 1"))?;
                } else if c2.denom == coin.denom {
                    c2.amount = c2
                        .amount
                        .checked_sub(coin.amount)
                        .map_err(|_| StdError::generic_err("Underflow when subtracting coin 2"))?;
                } else {
                    return Err(StdError::generic_err(
                        "No matching coin found for subtraction",
                    ));
                }
            }
            [Some(c1), None] => {
                if c1.denom == coin.denom {
                    c1.amount = c1
                        .amount
                        .checked_sub(coin.amount)
                        .map_err(|_| StdError::generic_err("Underflow when subtracting coin 1"))?;
                } else {
                    return Err(StdError::generic_err(
                        "No matching coin found for subtraction",
                    ));
                }
            }
            [None, None] => {
                return Err(StdError::generic_err(
                    "No matching coin found for subtraction",
                ));
            }
            [None, Some(_)] => unreachable!(),
        }
        Ok(())
    }

    pub fn sub_cw20(&mut self, cw20: &Cw20CoinVerified) -> StdResult<()> {
        match &mut self.cw20 {
            Some(task_cw20) if task_cw20.address == cw20.address => {
                task_cw20.amount = task_cw20
                    .amount
                    .checked_sub(cw20.amount)
                    .map_err(|_| StdError::generic_err("Underflow when subtracting cw20"))?;
            }
            _ => {
                // If addresses don't match, it means we have zero coins
                return Err(StdError::generic_err(
                    "Not enough cw20 balance for operation",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{Addr, Uint128};

    fn create_test_coin(denom: &str, amount: u128) -> Coin {
        Coin {
            denom: denom.to_string(),
            amount: Uint128::from(amount),
        }
    }

    fn create_test_cw20(address: &str, amount: u128) -> Cw20CoinVerified {
        Cw20CoinVerified {
            address: Addr::unchecked(address),
            amount: Uint128::from(amount),
        }
    }

    #[test]
    fn test_add_gas() {
        let mut amount = AmountForOneTask::default();
        amount.add_gas(100);
        assert_eq!(amount.gas, 100);

        amount.add_gas(100);
        assert_eq!(amount.gas, 200);
    }

    #[test]
    fn test_add_coin() {
        let mut amount = AmountForOneTask::default();
        let coin1 = create_test_coin("test1", 50);
        let coin2 = create_test_coin("test2", 100);
        let coin3 = create_test_coin("test3", 200);

        assert!(amount.add_coin(coin1.clone()).unwrap());
        assert_eq!(amount.coin[0].as_ref().unwrap(), &coin1);

        assert!(amount.add_coin(coin2.clone()).unwrap());
        assert_eq!(amount.coin[1].as_ref().unwrap(), &coin2);

        assert!(!amount.add_coin(coin3).unwrap());
    }

    #[test]
    fn test_add_cw20() {
        let mut amount = AmountForOneTask::default();
        let cw20_1 = create_test_cw20("addr1", 50);
        let cw20_2 = create_test_cw20("addr1", 100);
        let cw20_3 = create_test_cw20("addr2", 200);

        assert!(amount.add_cw20(cw20_1.clone()).unwrap());
        assert_eq!(amount.cw20.as_ref().unwrap(), &cw20_1);

        assert!(amount.add_cw20(cw20_2).unwrap());
        assert_eq!(amount.cw20.as_ref().unwrap().amount, Uint128::from(150u128));

        assert!(!amount.add_cw20(cw20_3).unwrap());
    }

    #[test]
    fn test_sub_coin() {
        let mut amount = AmountForOneTask::default();
        let coin1 = create_test_coin("test1", 100);
        let coin2 = create_test_coin("test1", 50);
        let coin3 = create_test_coin("test2", 100);
        let coin4 = create_test_coin("test2", 50);
        let coin5 = create_test_coin("test3", 100);

        amount.add_coin(coin1).unwrap();
        amount.add_coin(coin3).unwrap();

        amount.sub_coin(&coin2).unwrap();
        assert_eq!(
            amount.coin[0].as_ref().unwrap().amount,
            Uint128::from(50u128)
        );

        amount.sub_coin(&coin4).unwrap();
        assert_eq!(
            amount.coin[1].as_ref().unwrap().amount,
            Uint128::from(50u128)
        );

        assert_eq!(
            amount.sub_coin(&coin5).unwrap_err(),
            StdError::generic_err("No matching coin found for subtraction")
        );
    }

    #[test]
    fn test_sub_cw20() {
        let mut amount = AmountForOneTask::default();
        let cw20_1 = create_test_cw20("addr1", 100);
        let cw20_2 = create_test_cw20("addr1", 50);
        let cw20_3 = create_test_cw20("addr2", 100);

        amount.add_cw20(cw20_1).unwrap();

        amount.sub_cw20(&cw20_2).unwrap();
        assert_eq!(amount.cw20.as_ref().unwrap().amount, Uint128::from(50u128));

        assert_eq!(
            amount.sub_cw20(&cw20_3).unwrap_err(),
            StdError::generic_err("Not enough cw20 balance for operation")
        );
    }

    #[test]
    fn test_add_coin_overflow() {
        let mut amount = AmountForOneTask::default();
        let coin1 = create_test_coin("test1", u128::MAX);
        let coin2 = create_test_coin("test1", 1);

        amount.add_coin(coin1).unwrap();
        assert_eq!(
            amount.add_coin(coin2).unwrap_err(),
            StdError::generic_err("Overflow when adding coin 1")
        );
    }

    #[test]
    fn test_add_cw20_overflow() {
        let mut amount = AmountForOneTask::default();
        let cw20_1 = create_test_cw20("addr1", u128::MAX);
        let cw20_2 = create_test_cw20("addr1", 1);

        amount.add_cw20(cw20_1).unwrap();
        assert_eq!(
            amount.add_cw20(cw20_2).unwrap_err(),
            StdError::generic_err("Overflow when adding cw20")
        );
    }

    #[test]
    fn test_sub_coin_underflow() {
        let mut amount = AmountForOneTask::default();
        let coin1 = create_test_coin("test1", 50);
        let coin2 = create_test_coin("test1", 100);

        amount.add_coin(coin1).unwrap();
        assert_eq!(
            amount.sub_coin(&coin2).unwrap_err(),
            StdError::generic_err("Underflow when subtracting coin 1")
        );
    }

    #[test]
    fn test_add_coin_overflow_2() {
        let mut amount = AmountForOneTask::default();
        let coin1 = create_test_coin("test1", 100);
        let coin2 = create_test_coin("test2", u128::MAX);
        let coin3 = create_test_coin("test2", 1);

        amount.add_coin(coin1).unwrap();
        amount.add_coin(coin2).unwrap();
        assert_eq!(
            amount.add_coin(coin3).unwrap_err(),
            StdError::generic_err("Overflow when adding coin 2")
        );
    }

    #[test]
    fn test_sub_cw20_underflow() {
        let mut amount = AmountForOneTask::default();
        let cw20_1 = create_test_cw20("addr1", 50);
        let cw20_2 = create_test_cw20("addr1", 100);

        amount.add_cw20(cw20_1).unwrap();
        assert_eq!(
            amount.sub_cw20(&cw20_2).unwrap_err(),
            StdError::generic_err("Underflow when subtracting cw20")
        );
    }
}
