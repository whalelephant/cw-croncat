use cosmwasm_std::{coins, Addr};
use cw_multi_test::{App, AppBuilder};

use super::{ADMIN, ANYONE, DENOM};

pub(crate) fn default_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        let accounts: Vec<(u128, String)> = vec![
            (6_000_000, ADMIN.to_string()),
            (500_000, ANYONE.to_string()),
        ];
        for (amt, address) in accounts {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(address), coins(amt, DENOM))
                .unwrap();
        }
    })
}
