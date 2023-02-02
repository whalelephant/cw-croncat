use super::{
    ADMIN, AGENT0, AGENT1, AGENT2, AGENT3, AGENT4, AGENT_BENEFICIARY, ANYONE, DENOM, PARTICIPANT0,
    PARTICIPANT1, PARTICIPANT2, PARTICIPANT3, PARTICIPANT4, PARTICIPANT5, PARTICIPANT6, VERY_RICH,
};

use cosmwasm_std::{coins, Addr};
use cw_multi_test::{App, AppBuilder};

pub(crate) fn default_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        let accounts: Vec<(u128, String)> = vec![
            (6_000_000, ADMIN.to_string()),
            (500_000, ANYONE.to_string()),
            (2_000_000, AGENT0.to_string()),
            (2_000_000, AGENT1.to_string()),
            (2_000_000, AGENT2.to_string()),
            (2_000_000, AGENT3.to_string()),
            (2_000_000, AGENT4.to_string()),
            (500_0000, PARTICIPANT0.to_string()),
            (500_0000, PARTICIPANT1.to_string()),
            (500_0000, PARTICIPANT2.to_string()),
            (500_0000, PARTICIPANT3.to_string()),
            (500_0000, PARTICIPANT4.to_string()),
            (500_0000, PARTICIPANT5.to_string()),
            (500_0000, PARTICIPANT6.to_string()),
            (2_000_000, AGENT_BENEFICIARY.to_string()),
            (u128::max_value(), VERY_RICH.to_string()),
        ];
        for (amt, address) in accounts {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(address), coins(amt, DENOM))
                .unwrap();
        }
    })
}
