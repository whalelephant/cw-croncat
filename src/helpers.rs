use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, BankMsg, Coin, CosmosMsg, StdResult, SubMsg, WasmMsg};
use cw20::{Balance, Cw20CoinVerified, Cw20ExecuteMsg};

use crate::msg::ExecuteMsg;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinVerified>,
}

impl GenericBalance {
    pub fn add_tokens(&mut self, add: Balance) {
        match add {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    match index {
                        Some(idx) => self.native[idx].amount += token.amount,
                        None => self.native.push(token),
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                match index {
                    Some(idx) => self.cw20[idx].amount += token.amount,
                    None => self.cw20.push(token),
                }
            }
        };
    }
    pub fn minus_tokens(&mut self, minus: Balance) {
        match minus {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    if let Some(idx) = index {
                        self.native[idx].amount -= token.amount
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                if let Some(idx) = index {
                    self.cw20[idx].amount -= token.amount
                }
            }
        };
    }
}

// Helper to distribute funds/tokens
pub(crate) fn send_tokens(
    to: &Addr,
    balance: &GenericBalance,
) -> StdResult<(Vec<SubMsg>, GenericBalance)> {
    let native_balance = &balance.native;
    let mut coins: GenericBalance = GenericBalance::default();
    let mut msgs: Vec<SubMsg> = if native_balance.is_empty() {
        vec![]
    } else {
        coins.native = balance.native.clone();
        vec![SubMsg::new(BankMsg::Send {
            to_address: to.into(),
            amount: native_balance.to_vec(),
        })]
    };

    let cw20_balance = &balance.cw20;
    let cw20_msgs: StdResult<Vec<_>> = cw20_balance
        .iter()
        .map(|c| {
            let msg = Cw20ExecuteMsg::Transfer {
                recipient: to.into(),
                amount: c.amount,
            };
            let exec = SubMsg::new(WasmMsg::Execute {
                contract_addr: c.address.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            });
            Ok(exec)
        })
        .collect();
    coins.cw20 = balance.cw20.clone();
    msgs.append(&mut cw20_msgs?);
    Ok((msgs, coins))
}

/// has_cw_coins returns true if the list of CW20 coins has at least the required amount
pub(crate) fn has_cw_coins(coins: &[Cw20CoinVerified], required: &Cw20CoinVerified) -> bool {
    coins
        .iter()
        .find(|c| c.address == required.address)
        .map(|m| m.amount >= required.amount)
        .unwrap_or(false)
}

/// CwTemplateContract is a wrapper around Addr that provides a lot of helpers
/// for working with this.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

impl CwTemplateContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    // /// Get Count
    // pub fn count<Q, T, CQ>(&self, querier: &Q) -> StdResult<CountResponse>
    // where
    //     Q: Querier,
    //     T: Into<String>,
    //     CQ: CustomQuery,
    // {
    //     let msg = QueryMsg::GetCount {};
    //     let query = WasmQuery::Smart {
    //         contract_addr: self.addr().into(),
    //         msg: to_binary(&msg)?,
    //     }
    //     .into();
    //     let res: CountResponse = QuerierWrapper::<CQ>::new(querier).query(&query)?;
    //     Ok(res)
    // }
}
