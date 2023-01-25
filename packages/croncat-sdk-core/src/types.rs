use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, StdResult};
use cw20::Cw20CoinVerified;

#[cw_serde]
pub struct AmountForOneTask {
    pub gas: u64,
    pub cw20: Option<Cw20CoinVerified>,
    pub coin: [Option<Coin>; 2],
}

impl AmountForOneTask {
    #[must_use]
    pub fn add_gas(&mut self, gas: u64, limit: u64) -> bool {
        self.gas = self.gas.saturating_add(gas);

        self.gas <= limit
    }

    #[must_use]
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
}
