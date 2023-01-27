use cosmwasm_std::{to_binary, Addr, Uint128};
use cw_multi_test::{next_block, App, Executor};
use cw_utils::Duration;
use dao_core::state::ProposalModule;
use dao_voting::{
    multiple_choice::{MultipleChoiceOption, MultipleChoiceOptions, VotingStrategy},
    proposal::SingleChoiceProposeMsg,
    threshold::{PercentageThreshold, Threshold},
};
use mod_sdk::types::QueryResponse;

use crate::{
    msg::{InstantiateMsg, QueryMsg},
    tests::helpers::{
        contract_template, instantiate_with_staking_active_threshold, multiple_proposal_contract,
        single_proposal_contract, CREATOR_ADDR,
    },
};

#[test]
fn test_dao_single_has_proposals() {
    let mut app = App::default();
    let code_id = app.store_code(contract_template());

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
    let max_voting_period = Duration::Height(6);
    let instantiate_govmod = dao_proposal_single::msg::InstantiateMsg {
        threshold,
        max_voting_period,
        min_voting_period: None,
        only_members_execute: false,
        allow_revoting: false,
        close_proposal_on_execution_failure: true,
        pre_propose_info: dao_voting::pre_propose::PreProposeInfo::AnyoneMayPropose {},
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
            &dao_core::msg::QueryMsg::ProposalModules {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(governance_modules.len(), 1);
    let govmod_single = governance_modules.into_iter().next().unwrap().address;

    let govmod_config: dao_proposal_single::state::Config = app
        .wrap()
        .query_wasm_smart(
            govmod_single.clone(),
            &dao_proposal_single::msg::QueryMsg::Config {},
        )
        .unwrap();
    let dao = govmod_config.dao;
    let voting_module: Addr = app
        .wrap()
        .query_wasm_smart(dao, &dao_core::msg::QueryMsg::VotingModule {})
        .unwrap();
    let staking_contract: Addr = app
        .wrap()
        .query_wasm_smart(
            voting_module.clone(),
            &dao_voting_cw20_staked::msg::QueryMsg::StakingContract {},
        )
        .unwrap();
    let token_contract: Addr = app
        .wrap()
        .query_wasm_smart(
            voting_module,
            &dao_interface::voting::Query::TokenContract {},
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
        token_contract,
        &msg,
        &[],
    )
    .unwrap();
    app.update_block(next_block);

    // Check HasNew if there aren't any proposals
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::HasProposalsGtId {
                dao_address: govmod_single.to_string(),
                value: 0,
            },
        )
        .unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: false,
            data: to_binary(&0).unwrap()
        }
    );

    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::HasProposalsGtId {
                dao_address: govmod_single.to_string(),
                value: 1,
            },
        )
        .unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: false,
            data: to_binary(&0).unwrap()
        }
    );

    // Create three proposals
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &dao_proposal_single::msg::ExecuteMsg::Propose(SingleChoiceProposeMsg {
            title: "Cron".to_string(),
            description: "Cat".to_string(),
            msgs: vec![],
            proposer: None,
        }),
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &dao_proposal_single::msg::ExecuteMsg::Propose(SingleChoiceProposeMsg {
            title: "Cron2".to_string(),
            description: "Cat2".to_string(),
            msgs: vec![],
            proposer: None,
        }),
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &dao_proposal_single::msg::ExecuteMsg::Propose(SingleChoiceProposeMsg {
            title: "Cron3".to_string(),
            description: "Cat3".to_string(),
            msgs: vec![],
            proposer: None,
        }),
        &[],
    )
    .unwrap();

    // Compare the last proposal id (3) with specific value
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::HasProposalsGtId {
                dao_address: govmod_single.to_string(),
                value: 4,
            },
        )
        .unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: false,
            data: to_binary(&3).unwrap()
        }
    );

    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::HasProposalsGtId {
                dao_address: govmod_single.to_string(),
                value: 3,
            },
        )
        .unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: false,
            data: to_binary(&3).unwrap()
        }
    );

    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::HasProposalsGtId {
                dao_address: govmod_single.to_string(),
                value: 2,
            },
        )
        .unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: true,
            data: to_binary(&3).unwrap()
        }
    );
}

#[test]
fn test_dao_multiple_has_proposals() {
    let mut app = App::default();
    let code_id = app.store_code(contract_template());
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
    let instantiate_govmod = dao_proposal_multiple::msg::InstantiateMsg {
        voting_strategy,
        max_voting_period,
        min_voting_period: None,
        only_members_execute: false,
        allow_revoting: false,
        close_proposal_on_execution_failure: true,
        pre_propose_info: dao_voting::pre_propose::PreProposeInfo::AnyoneMayPropose {},
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
            &dao_core::msg::QueryMsg::ProposalModules {
                start_after: None,
                limit: None,
            },
        )
        .unwrap();

    assert_eq!(governance_modules.len(), 1);
    let govmod_single = governance_modules.into_iter().next().unwrap().address;

    let govmod_config: dao_proposal_multiple::state::Config = app
        .wrap()
        .query_wasm_smart(
            govmod_single.clone(),
            &dao_proposal_multiple::msg::QueryMsg::Config {},
        )
        .unwrap();
    let dao = govmod_config.dao;
    let voting_module: Addr = app
        .wrap()
        .query_wasm_smart(dao, &dao_core::msg::QueryMsg::VotingModule {})
        .unwrap();
    let staking_contract: Addr = app
        .wrap()
        .query_wasm_smart(
            voting_module.clone(),
            &dao_voting_cw20_staked::msg::QueryMsg::StakingContract {},
        )
        .unwrap();
    let token_contract: Addr = app
        .wrap()
        .query_wasm_smart(
            voting_module,
            &dao_interface::voting::Query::TokenContract {},
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
        token_contract,
        &msg,
        &[],
    )
    .unwrap();
    app.update_block(next_block);

    // Check HasNew if there aren't any proposals
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::HasProposalsGtId {
                dao_address: govmod_single.to_string(),
                value: 0,
            },
        )
        .unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: false,
            data: to_binary(&0).unwrap()
        }
    );

    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::HasProposalsGtId {
                dao_address: govmod_single.to_string(),
                value: 1,
            },
        )
        .unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: false,
            data: to_binary(&0).unwrap()
        }
    );

    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &dao_proposal_multiple::msg::ExecuteMsg::Propose {
            title: "Cron".to_string(),
            description: "Cat".to_string(),
            choices: MultipleChoiceOptions {
                options: vec![
                    MultipleChoiceOption {
                        title: "A".to_string(),
                        description: "a".to_string(),
                        msgs: vec![],
                    },
                    MultipleChoiceOption {
                        title: "B".to_string(),
                        description: "b".to_string(),
                        msgs: vec![],
                    },
                ],
            },
            proposer: None,
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &dao_proposal_multiple::msg::ExecuteMsg::Propose {
            title: "Cron2".to_string(),
            description: "Cat2".to_string(),
            choices: MultipleChoiceOptions {
                options: vec![
                    MultipleChoiceOption {
                        title: "A".to_string(),
                        description: "a".to_string(),
                        msgs: vec![],
                    },
                    MultipleChoiceOption {
                        title: "B".to_string(),
                        description: "b".to_string(),
                        msgs: vec![],
                    },
                ],
            },
            proposer: None,
        },
        &[],
    )
    .unwrap();

    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &dao_proposal_multiple::msg::ExecuteMsg::Propose {
            title: "Cron3".to_string(),
            description: "Cat3".to_string(),
            choices: MultipleChoiceOptions {
                options: vec![
                    MultipleChoiceOption {
                        title: "A".to_string(),
                        description: "a".to_string(),
                        msgs: vec![],
                    },
                    MultipleChoiceOption {
                        title: "B".to_string(),
                        description: "b".to_string(),
                        msgs: vec![],
                    },
                ],
            },
            proposer: None,
        },
        &[],
    )
    .unwrap();

    // Compare the last proposal id (3) with specific value
    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::HasProposalsGtId {
                dao_address: govmod_single.to_string(),
                value: 4,
            },
        )
        .unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: false,
            data: to_binary(&3).unwrap()
        }
    );

    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr.clone(),
            &QueryMsg::HasProposalsGtId {
                dao_address: govmod_single.to_string(),
                value: 3,
            },
        )
        .unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: false,
            data: to_binary(&3).unwrap()
        }
    );

    let res: QueryResponse = app
        .wrap()
        .query_wasm_smart(
            contract_addr,
            &QueryMsg::HasProposalsGtId {
                dao_address: govmod_single.to_string(),
                value: 2,
            },
        )
        .unwrap();
    assert_eq!(
        res,
        QueryResponse {
            result: true,
            data: to_binary(&3).unwrap()
        }
    );
}
