use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, StdResult, Uint128};
use cw20::Cw20CoinVerified;

#[cw_serde]
pub struct AmountForOneTask {
    pub gas: u64,
    pub cw20: Option<Cw20CoinVerified>,
    pub coin: [Option<Coin>; 2],
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
                    c1.amount = c1.amount.checked_add(coin.amount)?;
                } else {
                    self.coin[1] = Some(coin);
                }
                Ok(true)
            }
            [Some(c1), Some(c2)] => {
                if c1.denom == coin.denom {
                    c1.amount = c1.amount.checked_add(coin.amount)?;

                    Ok(true)
                } else if c2.denom == coin.denom {
                    c2.amount = c2.amount.checked_add(coin.amount)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            [None, Some(_)] => unreachable!(),
        }
    }

    #[must_use]
    pub fn add_cw20(&mut self, cw20: Cw20CoinVerified) -> bool {
        if let Some(cw20_inner) = &mut self.cw20 {
            if cw20_inner.address != cw20.address {
                return false;
            }
            cw20_inner.amount += cw20.amount;
        } else {
            self.cw20 = Some(cw20);
        }
        true
    }

    pub fn sub_coin(&mut self, coin: &Coin) -> StdResult<()> {
        match &mut self.coin {
            [Some(c1), Some(c2)] => {
                if c1.denom == coin.denom {
                    c1.amount = c1.amount.checked_sub(coin.amount)?;
                } else if c2.denom == coin.denom {
                    c2.amount = c2.amount.checked_sub(coin.amount)?;
                } else {
                    Uint128::zero().checked_sub(coin.amount)?;
                }
            }
            [Some(c1), None] => {
                if c1.denom == coin.denom {
                    c1.amount = c1.amount.checked_sub(coin.amount)?;
                } else {
                    Uint128::zero().checked_sub(coin.amount)?;
                }
            }
            [None, None] => {
                Uint128::zero().checked_sub(coin.amount)?;
            }
            [None, Some(_)] => unreachable!(),
        }
        Ok(())
    }

    pub fn sub_cw20(&mut self, cw20: &Cw20CoinVerified) -> StdResult<()> {
        match &mut self.cw20 {
            Some(task_cw20) if task_cw20.address == cw20.address => {
                task_cw20.amount = task_cw20.amount.checked_sub(cw20.amount)?;
            }
            _ => {
                // If addresses doesn't match it means we have zero coins
                Uint128::zero().checked_sub(cw20.amount)?;
            }
        }
        Ok(())
    }
}
