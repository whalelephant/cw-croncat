use cosmwasm_std::{to_binary, Addr, Binary, Empty, Uint128};
use cw20::Cw20Coin;
use cw20_staked_balance_voting::msg::ActiveThreshold;
use cw_core::state::ProposalModule;
use cw_croncat_core::types::CheckProposalStatus;
use cw_multi_test::{next_block, App, Contract, ContractWrapper, Executor};
use cw_proposal_multiple::{
    state::{MultipleChoiceOption, MultipleChoiceOptions},
    voting_strategy::VotingStrategy,
};
use voting::{
    status::Status,
    threshold::{PercentageThreshold, Threshold},
    voting::{MultipleChoiceVote, Vote},
};

use crate::msg::{InstantiateMsg, QueryMsg, RuleResponse};

const CREATOR_ADDR: &str = "creator";

fn cw_rules_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    );
    Box::new(contract)
}

fn cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    );
    Box::new(contract)
}

fn cw20_stake_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_stake::contract::execute,
        cw20_stake::contract::instantiate,
        cw20_stake::contract::query,
    );
    Box::new(contract)
}

fn single_proposal_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw_proposal_single::contract::execute,
        cw_proposal_single::contract::instantiate,
        cw_proposal_single::contract::query,
    )
    .with_reply(cw_proposal_single::contract::reply)
    .with_migrate(cw_proposal_single::contract::migrate);
    Box::new(contract)
}

fn multiple_proposal_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw_proposal_multiple::contract::execute,
        cw_proposal_multiple::contract::instantiate,
        cw_proposal_multiple::contract::query,
    )
    .with_reply(cw_proposal_multiple::contract::reply)
    .with_migrate(cw_proposal_multiple::contract::migrate);
    Box::new(contract)
}

fn cw_gov_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw_core::contract::execute,
        cw_core::contract::instantiate,
        cw_core::contract::query,
    )
    .with_reply(cw_core::contract::reply);
    Box::new(contract)
}

fn cw20_staked_balances_voting() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw20_staked_balance_voting::contract::execute,
        cw20_staked_balance_voting::contract::instantiate,
        cw20_staked_balance_voting::contract::query,
    )
    .with_reply(cw20_staked_balance_voting::contract::reply);
    Box::new(contract)
}

fn instantiate_with_staking_active_threshold(
    app: &mut App,
    proposal_module_code_id: u64,
    proposal_module_instantiate: Binary,
    initial_balances: Option<Vec<Cw20Coin>>,
    active_threshold: Option<ActiveThreshold>,
) -> Addr {
    let cw20_id = app.store_code(cw20_contract());
    let cw20_staking_id = app.store_code(cw20_stake_contract());
    let governance_id = app.store_code(cw_gov_contract());
    let votemod_id = app.store_code(cw20_staked_balances_voting());

    let initial_balances = initial_balances.unwrap_or_else(|| {
        vec![Cw20Coin {
            address: CREATOR_ADDR.to_string(),
            amount: Uint128::new(100_000_000),
        }]
    });

    let governance_instantiate = cw_core::msg::InstantiateMsg {
        admin: None,
        name: "DAO DAO".to_string(),
        description: "A DAO that builds DAOs".to_string(),
        image_url: None,
        automatically_add_cw20s: true,
        automatically_add_cw721s: true,
        voting_module_instantiate_info: cw_core::msg::ModuleInstantiateInfo {
            code_id: votemod_id,
            msg: to_binary(&cw20_staked_balance_voting::msg::InstantiateMsg {
                token_info: cw20_staked_balance_voting::msg::TokenInfo::New {
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
            admin: cw_core::msg::Admin::CoreContract {},
            label: "DAO DAO voting module".to_string(),
        },
        proposal_modules_instantiate_info: vec![cw_core::msg::ModuleInstantiateInfo {
            code_id: proposal_module_code_id,
            msg: proposal_module_instantiate,
            admin: cw_core::msg::Admin::CoreContract {},
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

#[test]
fn test_dao_single_proposal_ready() {
    let mut app = App::default();
    let code_id = app.store_code(cw_rules_contract());

    let instantiate = InstantiateMsg {};
    let contract_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate,
            &[],
            "cw-rules",
            None,
        )
        .unwrap();

    let proposal_module_code_id = app.store_code(single_proposal_contract());
    let threshold = Threshold::AbsolutePercentage {
        percentage: PercentageThreshold::Majority {},
    };
    let max_voting_period = cw_utils::Duration::Height(6);
    let instantiate_govmod = cw_proposal_single::msg::InstantiateMsg {
        threshold,
        max_voting_period,
        min_voting_period: None,
        only_members_execute: false,
        allow_revoting: false,
        deposit_info: None,
        close_proposal_on_execution_failure: true,
    };
    let governance_addr = instantiate_with_staking_active_threshold(
        &mut app,
        proposal_module_code_id,
        to_binary(&instantiate_govmod).unwrap(),
        None,
        None,
    );
    let governance_modules: Vec<ProposalModule> = app
        .wrap()
        .query_wasm_smart(
            governance_addr,
            &cw_core::msg::QueryMsg::ProposalModules {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(governance_modules.len(), 1);
    let govmod_single = governance_modules.into_iter().next().unwrap().address;

    let govmod_config: cw_proposal_single::state::Config = app
        .wrap()
        .query_wasm_smart(
            govmod_single.clone(),
            &cw_proposal_single::msg::QueryMsg::Config {},
        )
        .unwrap();
    let dao = govmod_config.dao;
    let voting_module: Addr = app
        .wrap()
        .query_wasm_smart(dao, &cw_core::msg::QueryMsg::VotingModule {})
        .unwrap();
    let staking_contract: Addr = app
        .wrap()
        .query_wasm_smart(
            voting_module.clone(),
            &cw20_staked_balance_voting::msg::QueryMsg::StakingContract {},
        )
        .unwrap();
    let token_contract: Addr = app
        .wrap()
        .query_wasm_smart(
            voting_module,
            &cw_core_interface::voting::Query::TokenContract {},
        )
        .unwrap();

    // Stake some tokens so we can propose
    let msg = cw20::Cw20ExecuteMsg::Send {
        contract: staking_contract.to_string(),
        amount: Uint128::new(2000),
        msg: to_binary(&cw20_stake::msg::ReceiveMsg::Stake {}).unwrap(),
    };
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        token_contract.clone(),
        &msg,
        &[],
    )
    .unwrap();
    app.update_block(next_block);

    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &cw_proposal_single::msg::ExecuteMsg::Propose {
            title: "Cron".to_string(),
            description: "Cat".to_string(),
            msgs: vec![],
        },
        &[],
    )
    .unwrap();

    // It is not ready to execute yet, so false
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::CheckProposalStatus(CheckProposalStatus {
                dao_address: govmod_single.to_string(),
                proposal_id: 1,
                status: Status::Passed,
            }),
        )
        .unwrap();
    assert_eq!(res, (false, None));

    // Approve proposal
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &cw_proposal_single::msg::ExecuteMsg::Vote {
            proposal_id: 1,
            vote: Vote::Yes,
        },
        &[],
    )
    .unwrap();

    // It's now ready to be executed
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::CheckProposalStatus(CheckProposalStatus {
                dao_address: govmod_single.to_string(),
                proposal_id: 1,
                status: Status::Passed,
            }),
        )
        .unwrap();
    assert_eq!(res, (true, None));

    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &cw_proposal_single::msg::ExecuteMsg::Execute { proposal_id: 1 },
        &[],
    )
    .unwrap();

    // It's executed now
    // Test if other types of status works
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::CheckProposalStatus(CheckProposalStatus {
                dao_address: govmod_single.to_string(),
                proposal_id: 1,
                status: Status::Executed,
            }),
        )
        .unwrap();
    assert_eq!(res, (true, None));
}

#[test]
fn test_dao_multiple_proposal_ready() {
    let mut app = App::default();
    let code_id = app.store_code(cw_rules_contract());
    let instantiate = InstantiateMsg {};
    let contract_addr = app
        .instantiate_contract(
            code_id,
            Addr::unchecked(CREATOR_ADDR),
            &instantiate,
            &[],
            "cw-rules",
            None,
        )
        .unwrap();

    let proposal_module_code_id = app.store_code(multiple_proposal_contract());
    let voting_strategy = VotingStrategy::SingleChoice {
        quorum: PercentageThreshold::Majority {},
    };
    let max_voting_period = cw_utils::Duration::Height(6);
    let instantiate_govmod = cw_proposal_multiple::msg::InstantiateMsg {
        voting_strategy,
        max_voting_period,
        min_voting_period: None,
        only_members_execute: false,
        allow_revoting: false,
        deposit_info: None,
        close_proposal_on_execution_failure: true,
    };
    let governance_addr = instantiate_with_staking_active_threshold(
        &mut app,
        proposal_module_code_id,
        to_binary(&instantiate_govmod).unwrap(),
        None,
        None,
    );
    let governance_modules: Vec<ProposalModule> = app
        .wrap()
        .query_wasm_smart(
            governance_addr,
            &cw_core::msg::QueryMsg::ProposalModules {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(governance_modules.len(), 1);
    let govmod_single = governance_modules.into_iter().next().unwrap().address;

    let govmod_config: cw_proposal_multiple::state::Config = app
        .wrap()
        .query_wasm_smart(
            govmod_single.clone(),
            &cw_proposal_multiple::msg::QueryMsg::Config {},
        )
        .unwrap();
    let dao = govmod_config.dao;
    let voting_module: Addr = app
        .wrap()
        .query_wasm_smart(dao, &cw_core::msg::QueryMsg::VotingModule {})
        .unwrap();
    let staking_contract: Addr = app
        .wrap()
        .query_wasm_smart(
            voting_module.clone(),
            &cw20_staked_balance_voting::msg::QueryMsg::StakingContract {},
        )
        .unwrap();
    let token_contract: Addr = app
        .wrap()
        .query_wasm_smart(
            voting_module,
            &cw_core_interface::voting::Query::TokenContract {},
        )
        .unwrap();

    // Stake some tokens so we can propose
    let msg = cw20::Cw20ExecuteMsg::Send {
        contract: staking_contract.to_string(),
        amount: Uint128::new(2000),
        msg: to_binary(&cw20_stake::msg::ReceiveMsg::Stake {}).unwrap(),
    };
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        token_contract.clone(),
        &msg,
        &[],
    )
    .unwrap();
    app.update_block(next_block);

    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &cw_proposal_multiple::msg::ExecuteMsg::Propose {
            title: "Cron".to_string(),
            description: "Cat".to_string(),
            choices: MultipleChoiceOptions {
                options: vec![
                    MultipleChoiceOption {
                        description: "a".to_string(),
                        msgs: None,
                    },
                    MultipleChoiceOption {
                        description: "b".to_string(),
                        msgs: None,
                    },
                ],
            },
        },
        &[],
    )
    .unwrap();

    // It is not ready to execute yet, so false
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::CheckProposalStatus(CheckProposalStatus {
                dao_address: govmod_single.to_string(),
                proposal_id: 1,
                status: Status::Passed,
            }),
        )
        .unwrap();
    assert_eq!(res, (false, None));

    // Approve proposal
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &cw_proposal_multiple::msg::ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    // It's now ready to be executed
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::CheckProposalStatus(CheckProposalStatus {
                dao_address: govmod_single.to_string(),
                proposal_id: 1,
                status: Status::Passed,
            }),
        )
        .unwrap();
    assert_eq!(res, (true, None));

    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &cw_proposal_multiple::msg::ExecuteMsg::Execute { proposal_id: 1 },
        &[],
    )
    .unwrap();

    // It's executed now
    // Test if other types of status works
    let res: RuleResponse<Option<Binary>> = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::CheckProposalStatus(CheckProposalStatus {
                dao_address: govmod_single.to_string(),
                proposal_id: 1,
                status: Status::Executed,
            }),
        )
        .unwrap();
    assert_eq!(res, (true, None));
}
