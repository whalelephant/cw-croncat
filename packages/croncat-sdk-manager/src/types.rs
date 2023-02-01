use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw20::Cw20CoinVerified;

use crate::error::SdkError;

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
pub struct TaskBalance {
    pub native_balance: Uint128,
    pub cw20_balance: Option<Cw20CoinVerified>,
    pub ibc_balance: Option<Coin>,
}

impl TaskBalance {
    pub fn verify_enough_attached(
        &self,
        native_required: Uint128,
        cw20_required: Option<Cw20CoinVerified>,
        ibc_required: Option<Coin>,
        recurring: bool,
        native_denom: &str,
    ) -> Result<(), SdkError> {
        let multiplier = if recurring {
            Uint128::new(2)
        } else {
            Uint128::new(1)
        };
        if self.native_balance < native_required * multiplier {
            return Err(SdkError::NotEnoughNative {
                denom: native_denom.to_owned(),
                lack: native_required * multiplier - self.native_balance,
            });
        }
        match (cw20_required, &self.cw20_balance) {
            (Some(req), Some(attached)) => {
                if req.address != attached.address {
                    return Err(SdkError::NotEnoughCw20 {
                        addr: req.address.into_string(),
                        lack: req.amount * multiplier,
                    });
                }
                if attached.amount < req.amount * multiplier {
                    return Err(SdkError::NotEnoughCw20 {
                        addr: req.address.into_string(),
                        lack: req.amount * multiplier - attached.amount,
                    });
                }
            }
            (Some(req), None) => {
                return Err(SdkError::NotEnoughCw20 {
                    addr: req.address.into_string(),
                    lack: req.amount * multiplier,
                })
            }
            // Note: we are Ok if user decided to attach "needless" cw20
            (None, Some(_)) | (None, None) => (),
        }
        match (ibc_required, &self.ibc_balance) {
            (Some(req), Some(attached)) => {
                if req.denom != attached.denom {
                    return Err(SdkError::NotEnoughNative {
                        denom: req.denom,
                        lack: req.amount * multiplier,
                    });
                }
                if attached.amount < req.amount * multiplier {
                    return Err(SdkError::NotEnoughNative {
                        denom: req.denom,
                        lack: req.amount * multiplier - attached.amount,
                    });
                }
            }
            (Some(req), None) => {
                return Err(SdkError::NotEnoughNative {
                    denom: req.denom,
                    lack: req.amount * multiplier,
                })
            }
            // Note: we are Ok if user decided to attach "needless" cw20
            (None, Some(_)) | (None, None) => (),
        }
        Ok(())
    }
}

#[cw_serde]
pub struct Config {
    // Runtime
    pub paused: bool,
    pub owner_addr: Addr,

    /// Address of the croncat_factory
    pub croncat_factory_addr: Addr,

    /// Key to query address of the tasks
    pub croncat_tasks_key: (String, [u8; 2]),
    /// Key to query address of the agents
    pub croncat_agents_key: (String, [u8; 2]),

    // Economics
    pub agent_fee: u64,
    pub treasury_fee: u64,
    pub gas_price: GasPrice,

    // Treasury
    pub treasury_addr: Option<Addr>,
    pub cw20_whitelist: Vec<Addr>, // TODO: Consider fee structure for whitelisted CW20s
    pub native_denom: String,

    // The default query limit
    pub limit: u64,
}

#[cw_serde]
pub struct UpdateConfig {
    pub owner_addr: Option<String>,
    pub paused: Option<bool>,
    pub agent_fee: Option<u64>,
    pub treasury_fee: Option<u64>,
    pub gas_price: Option<GasPrice>,
    pub croncat_tasks_key: Option<(String, [u8; 2])>,
    pub croncat_agents_key: Option<(String, [u8; 2])>,
    pub treasury_addr: Option<String>,
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{coin, Addr, Uint128};
    use cw20::Cw20CoinVerified;

    use crate::SdkError;

    use super::{GasPrice, TaskBalance};

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
        assert!(matches!(err, SdkError::InvalidGas {}));
    }

    #[test]
    fn verify_enough_attached_ok_test() {
        let native_balance = Uint128::from(100u64);
        let cw20 = Cw20CoinVerified {
            address: Addr::unchecked("addr"),
            amount: Uint128::from(100u64),
        };
        let ibc_coin = coin(100, "ibc");

        // Test when cw20_balance and ibc_balance are None
        let task_balance = TaskBalance {
            native_balance,
            cw20_balance: None,
            ibc_balance: None,
        };
        assert!(task_balance
            .verify_enough_attached(Uint128::from(100u64), None, None, false, "denom")
            .is_ok());
        assert!(task_balance
            .verify_enough_attached(Uint128::from(50u64), None, None, true, "denom")
            .is_ok());

        // Test with cw20_balance and ibc_balance
        let task_balance = TaskBalance {
            native_balance,
            cw20_balance: Some(cw20.clone()),
            ibc_balance: Some(ibc_coin.clone()),
        };
        assert!(task_balance
            .verify_enough_attached(Uint128::from(100u64), None, None, false, "denom")
            .is_ok());
        assert!(task_balance
            .verify_enough_attached(Uint128::from(50u64), None, None, true, "denom")
            .is_ok());
        assert!(task_balance
            .verify_enough_attached(
                Uint128::from(100u64),
                Some(cw20),
                Some(ibc_coin),
                false,
                "denom"
            )
            .is_ok());
        assert!(task_balance
            .verify_enough_attached(
                Uint128::from(50u64),
                Some(Cw20CoinVerified {
                    address: Addr::unchecked("addr"),
                    amount: Uint128::from(50u64),
                }),
                Some(coin(50, "ibc")),
                true,
                "denom"
            )
            .is_ok());
    }

    #[test]
    fn verify_enough_attached_err_test() {
        let native_balance = Uint128::from(100u64);
        let cw20 = Cw20CoinVerified {
            address: Addr::unchecked("addr"),
            amount: Uint128::from(100u64),
        };
        let ibc_coin = coin(100, "ibc");

        // Test when cw20_balance and ibc_balance are None, native_balance is not sufficient
        let task_balance = TaskBalance {
            native_balance,
            cw20_balance: None,
            ibc_balance: None,
        };
        assert_eq!(
            task_balance
                .verify_enough_attached(Uint128::from(101u64), None, None, false, "denom")
                .unwrap_err(),
            SdkError::NotEnoughNative {
                denom: "denom".to_owned(),
                lack: 1u64.into(),
            }
        );
        assert_eq!(
            task_balance
                .verify_enough_attached(native_balance, Some(cw20.clone()), None, false, "denom")
                .unwrap_err(),
            SdkError::NotEnoughCw20 {
                addr: "addr".to_owned(),
                lack: 100u64.into(),
            }
        );
        assert_eq!(
            task_balance
                .verify_enough_attached(
                    native_balance,
                    None,
                    Some(ibc_coin.clone()),
                    false,
                    "denom"
                )
                .unwrap_err(),
            SdkError::NotEnoughNative {
                denom: "ibc".to_owned(),
                lack: 100u64.into(),
            }
        );

        // Test when cw20_balance or ibc_balance are not sufficient
        let task_balance = TaskBalance {
            native_balance,
            cw20_balance: Some(cw20.clone()),
            ibc_balance: Some(ibc_coin.clone()),
        };
        // cw20_balance is not sufficient
        assert_eq!(
            task_balance
                .verify_enough_attached(
                    Uint128::from(100u64),
                    Some(Cw20CoinVerified {
                        address: Addr::unchecked("addr"),
                        amount: Uint128::from(101u64),
                    }),
                    Some(ibc_coin.clone()),
                    false,
                    "denom"
                )
                .unwrap_err(),
            SdkError::NotEnoughCw20 {
                addr: "addr".to_owned(),
                lack: 1u64.into(),
            }
        );
        // cw20_balance has another address
        assert_eq!(
            task_balance
                .verify_enough_attached(
                    Uint128::from(100u64),
                    Some(Cw20CoinVerified {
                        address: Addr::unchecked("addr2"),
                        amount: Uint128::from(100u64),
                    }),
                    Some(ibc_coin),
                    false,
                    "denom"
                )
                .unwrap_err(),
            SdkError::NotEnoughCw20 {
                addr: "addr2".to_owned(),
                lack: 100u64.into(),
            }
        );
        // ibc_balance is not sufficient
        assert_eq!(
            task_balance
                .verify_enough_attached(
                    Uint128::from(100u64),
                    Some(cw20.clone()),
                    Some(coin(101, "ibc")),
                    false,
                    "denom"
                )
                .unwrap_err(),
            SdkError::NotEnoughNative {
                denom: "ibc".to_owned(),
                lack: 1u64.into(),
            }
        );
        // ibc_balance has another denom
        assert_eq!(
            task_balance
                .verify_enough_attached(
                    Uint128::from(100u64),
                    Some(cw20),
                    Some(coin(100, "ibc2")),
                    false,
                    "denom"
                )
                .unwrap_err(),
            SdkError::NotEnoughNative {
                denom: "ibc2".to_owned(),
                lack: 100u64.into(),
            }
        );
    }
}
