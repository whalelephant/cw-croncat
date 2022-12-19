use cosmwasm_std::{DepsMut, MessageInfo, Response};
use cw20::{Cw20CoinVerified, Cw20ReceiveMsg};
use cw_croncat_core::traits::BalancesOperations;

use crate::{ContractError, CwCroncat};

impl<'a> CwCroncat<'a> {
    /// Add cw20 coin to user balance, that sent this coins
    pub fn receive_cw20(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        msg: Cw20ReceiveMsg,
    ) -> Result<Response, ContractError> {
        let sender = deps.api.addr_validate(&msg.sender)?;
        let coin_address = info.sender;

        // Updating user balance
        let new_balances = self.balances.update(
            deps.storage,
            &sender,
            |balances| -> Result<_, ContractError> {
                let mut balances = balances.unwrap_or_default();
                balances.checked_add_coins(&[Cw20CoinVerified {
                    address: coin_address.clone(),
                    amount: msg.amount,
                }])?;
                Ok(balances)
            },
        )?;

        // Updating contract balance
        self.config
            .update(deps.storage, |mut c| -> Result<_, ContractError> {
                c.available_balance.checked_add_cw20(&[Cw20CoinVerified {
                    address: coin_address,
                    amount: msg.amount,
                }])?;
                Ok(c)
            })?;

        let total_cw20_string: Vec<String> = new_balances.iter().map(ToString::to_string).collect();
        Ok(Response::new()
            .add_attribute("method", "receive_cw20")
            .add_attribute("total_cw20_balances", format!("{total_cw20_string:?}")))
    }
}
