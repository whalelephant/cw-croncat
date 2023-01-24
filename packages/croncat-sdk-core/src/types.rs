use cosmwasm_schema::cw_serde;
use cosmwasm_std::Coin;
use cw20::Cw20CoinVerified;

#[cw_serde]
#[derive(Default)]
pub struct AmountForOneTask {
    pub gas: u64,
    pub cw20: Option<Cw20CoinVerified>,
    pub coin: Option<Coin>,
}

impl AmountForOneTask {
    #[must_use]
    pub fn add_gas(&mut self, gas: u64, limit: u64) -> bool {
        self.gas = self.gas.saturating_add(gas);

        self.gas <= limit
    }

    #[must_use]
    pub fn add_coin(&mut self, coin: Coin) -> bool {
        if let Some(coin_inner) = &mut self.coin {
            if coin_inner.denom != coin.denom {
                return false;
            }
            coin_inner.amount += coin.amount;
        } else {
            self.coin = Some(coin);
        }
        true
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
