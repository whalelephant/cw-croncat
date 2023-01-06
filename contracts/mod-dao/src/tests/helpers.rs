use cosmwasm_std::{to_binary, Addr, Binary, Empty, Uint128};
use cw20::Cw20Coin;
use cw_multi_test::{App, Contract, ContractWrapper, Executor};
use dao_voting_cw20_staked::msg::ActiveThreshold;
//use cw_rules_core::msg::InstantiateMsg;

pub const CREATOR_ADDR: &str = "creator";
pub const ADMIN: &str = "cosmos1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u0tvx7u";
pub const ANYONE: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const ADMIN_CW20: &str = "cosmos1a7uhnpqthunr2rzj0ww0hwurpn42wyun6c5puz";
pub const ANOTHER: &str = "cosmos1wze8mn5nsgl9qrgazq6a92fvh7m5e6psjcx2du";
pub const NATIVE_DENOM: &str = "atom";

pub fn contract_template() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

// pub fn cw_rules_contract() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new(
//         crate::contract::execute,
//         crate::contract::instantiate,
//         crate::contract::query,
//     );
//     Box::new(contract)
// }

// pub fn cw4_contract() -> Box<dyn Contract<Empty>> {
//     let contract = ContractWrapper::new(
//         cw4_group::contract::execute,
//         cw4_group::contract::instantiate,
//         cw4_group::contract::query,
//     );
//     Box::new(contract)
// }

pub fn cw20_template() -> Box<dyn Contract<Empty>> {
    let cw20 = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(cw20)
}

pub fn cw20_stake_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_stake::contract::execute,
        cw20_stake::contract::instantiate,
        cw20_stake::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn single_proposal_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        dao_proposal_single::contract::execute,
        dao_proposal_single::contract::instantiate,
        dao_proposal_single::contract::query,
    )
    .with_reply(dao_proposal_single::contract::reply)
    .with_migrate(dao_proposal_single::contract::migrate);
    Box::new(contract)
}

pub(crate) fn multiple_proposal_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        dao_proposal_multiple::contract::execute,
        dao_proposal_multiple::contract::instantiate,
        dao_proposal_multiple::contract::query,
    )
    .with_reply(dao_proposal_multiple::contract::reply)
    .with_migrate(dao_proposal_multiple::contract::migrate);
    Box::new(contract)
}

pub fn cw_gov_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        dao_core::contract::execute,
        dao_core::contract::instantiate,
        dao_core::contract::query,
    )
    .with_reply(dao_core::contract::reply);
    Box::new(contract)
}

pub fn cw20_staked_balances_voting() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        dao_voting_cw20_staked::contract::execute,
        dao_voting_cw20_staked::contract::instantiate,
        dao_voting_cw20_staked::contract::query,
    )
    .with_reply(dao_voting_cw20_staked::contract::reply);
    Box::new(contract)
}

// fn mock_app() -> App {
//     AppBuilder::new().build(|router, _, storage| {
//         let accounts: Vec<(u128, String)> = vec![
//             (6_000_000, ADMIN.to_string()),
//             (6_000_000, ADMIN_CW20.to_string()),
//             (1_000_000, ANYONE.to_string()),
//         ];
//         for (amt, address) in accounts.iter() {
//             router
//                 .bank
//                 .init_balance(
//                     storage,
//                     &Addr::unchecked(address),
//                     vec![coin(amt.clone(), NATIVE_DENOM.to_string())],
//                 )
//                 .unwrap();
//         }
//     })
// }

// pub fn proper_instantiate() -> (App, Addr, Addr) {
//     let mut app = mock_app();
//     let cw_template_id = app.store_code(contract_template());
//     let owner_addr = Addr::unchecked(ADMIN);
//     let nft_owner_addr = Addr::unchecked(ADMIN_CW20);

//     let msg = InstantiateMsg {};
//     let cw_template_contract_addr = app
//         .instantiate_contract(
//             cw_template_id,
//             owner_addr,
//             &msg,
//             &coins(2_000_000, NATIVE_DENOM),
//             "CW-RULES",
//             None,
//         )
//         .unwrap();

//     let cw20_id = app.store_code(cw20_template());
//     let msg = cw20_base::msg::InstantiateMsg {
//         name: "Test".to_string(),
//         symbol: "Test".to_string(),
//         decimals: 6,
//         initial_balances: vec![Cw20Coin {
//             address: ANYONE.to_string(),
//             amount: 15u128.into(),
//         }],
//         mint: None,
//         marketing: None,
//     };
//     let cw20_addr = app
//         .instantiate_contract(cw20_id, nft_owner_addr, &msg, &[], "Fungible-tokens", None)
//         .unwrap();

//     (app, cw_template_contract_addr, cw20_addr)
// }

pub(crate) fn instantiate_with_staking_active_threshold(
    app: &mut App,
    proposal_module_code_id: u64,
    proposal_module_instantiate: Binary,
    initial_balances: Option<Vec<Cw20Coin>>,
    active_threshold: Option<ActiveThreshold>,
) -> Addr {
    let cw20_id = app.store_code(cw20_template());
    let cw20_staking_id = app.store_code(cw20_stake_contract());
    let governance_id = app.store_code(cw_gov_contract());
    let votemod_id = app.store_code(cw20_staked_balances_voting());

    let initial_balances = initial_balances.unwrap_or_else(|| {
        vec![Cw20Coin {
            address: CREATOR_ADDR.to_string(),
            amount: Uint128::new(100_000_000),
        }]
    });

    let governance_instantiate = dao_core::msg::InstantiateMsg {
        admin: None,
        name: "DAO DAO".to_string(),
        description: "A DAO that builds DAOs".to_string(),
        image_url: None,
        automatically_add_cw20s: true,
        automatically_add_cw721s: true,
        voting_module_instantiate_info: dao_interface::ModuleInstantiateInfo {
            code_id: votemod_id,
            msg: to_binary(&dao_voting_cw20_staked::msg::InstantiateMsg {
                token_info: dao_voting_cw20_staked::msg::TokenInfo::New {
                    code_id: cw20_id,
                    label: "DAO DAO governance token".to_string(),
                    name: "DAO".to_string(),
                    symbol: "DAO".to_string(),
                    decimals: 6,
                    initial_balances,
                    marketing: None,
                    staking_code_id: cw20_staking_id,
                    unstaking_duration: None,
                    initial_dao_balance: None,
                },
                active_threshold,
            })
            .unwrap(),
            admin: Some(dao_interface::Admin::CoreModule {}),
            label: "DAO DAO voting module".to_string(),
        },
        proposal_modules_instantiate_info: vec![dao_interface::ModuleInstantiateInfo {
            code_id: proposal_module_code_id,
            msg: proposal_module_instantiate,
            admin: Some(dao_interface::Admin::CoreModule {}),
            label: "DAO DAO governance module".to_string(),
        }],
        initial_items: None,
        dao_uri: None,
    };

    app.instantiate_contract(
        governance_id,
        Addr::unchecked(CREATOR_ADDR),
        &governance_instantiate,
        &[],
        "DAO DAO",
        None,
    )
    .unwrap()
}
