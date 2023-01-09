use cosmwasm_std::{to_binary, Addr, Uint128};
use cw_multi_test::{next_block, App, Executor};
use cw_utils::{Duration, Expiration};
use dao_core::state::ProposalModule;
use dao_proposal_multiple::proposal::MultipleChoiceProposal;
use dao_proposal_single::proposal::SingleChoiceProposal;
use dao_voting::{
    multiple_choice::{
        CheckedMultipleChoiceOption, MultipleChoiceOption, MultipleChoiceOptionType,
        MultipleChoiceOptions, MultipleChoiceVote, MultipleChoiceVotes, VotingStrategy,
    },
    proposal::SingleChoiceProposeMsg,
    threshold::{PercentageThreshold, Threshold},
    voting::{Vote, Votes},
};
use mod_sdk::types::QueryResponse;

use crate::{
    msg::{InstantiateMsg, QueryMsg},
    tests::helpers::{
        contract_template, instantiate_with_staking_active_threshold, multiple_proposal_contract,
        single_proposal_contract, CREATOR_ADDR,
    },
    types::{dao::Status, CheckProposalStatus},
};

#[test]
fn test_dao_single_proposal_ready() {
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
        threshold: threshold.clone(),
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
        token_contract.clone(),
        &msg,
        &[],
    )
    .unwrap();
    app.update_block(next_block);

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

    // It is not ready to execute yet, so false
    let res: QueryResponse = app
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
    assert_eq!(
        res,
        QueryResponse {
            result: false,
            data: to_binary(&dao_proposal_single::query::ProposalResponse {
                id: 1,
                proposal: SingleChoiceProposal {
                    title: "Cron".to_string(),
                    description: "Cat".to_string(),
                    proposer: Addr::unchecked(CREATOR_ADDR),
                    start_height: 12346,
                    min_voting_period: None,
                    expiration: Expiration::AtHeight(12352),
                    threshold: threshold.clone(),
                    total_power: Uint128::new(2000),
                    msgs: vec![],
                    status: dao_voting::status::Status::Open,
                    votes: Votes {
                        yes: Uint128::zero(),
                        no: Uint128::zero(),
                        abstain: Uint128::zero(),
                    },
                    allow_revoting: false
                },
            })
            .unwrap()
        }
    );

    // Approve proposal
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &dao_proposal_single::msg::ExecuteMsg::Vote {
            proposal_id: 1,
            vote: Vote::Yes,
            rationale: None,
        },
        &[],
    )
    .unwrap();

    // It's now ready to be executed
    let res: QueryResponse = app
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
    assert_eq!(
        res,
        QueryResponse {
            result: true,
            data: to_binary(&dao_proposal_single::query::ProposalResponse {
                id: 1,
                proposal: SingleChoiceProposal {
                    title: "Cron".to_string(),
                    description: "Cat".to_string(),
                    proposer: Addr::unchecked(CREATOR_ADDR),
                    start_height: 12346,
                    min_voting_period: None,
                    expiration: Expiration::AtHeight(12352),
                    threshold: threshold.clone(),
                    total_power: Uint128::new(2000),
                    msgs: vec![],
                    status: dao_voting::status::Status::Passed,
                    votes: Votes {
                        yes: Uint128::new(2000),
                        no: Uint128::zero(),
                        abstain: Uint128::zero(),
                    },
                    allow_revoting: false
                },
            })
            .unwrap()
        }
    );
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &dao_proposal_single::msg::ExecuteMsg::Execute { proposal_id: 1 },
        &[],
    )
    .unwrap();

    // It's executed now
    // Test if other types of status works
    let res: QueryResponse = app
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
    assert_eq!(
        res,
        QueryResponse {
            result: true,
            data: to_binary(&dao_proposal_single::query::ProposalResponse {
                id: 1,
                proposal: SingleChoiceProposal {
                    title: "Cron".to_string(),
                    description: "Cat".to_string(),
                    proposer: Addr::unchecked(CREATOR_ADDR),
                    start_height: 12346,
                    min_voting_period: None,
                    expiration: Expiration::AtHeight(12352),
                    threshold: threshold.clone(),
                    total_power: Uint128::new(2000),
                    msgs: vec![],
                    status: dao_voting::status::Status::Executed,
                    votes: Votes {
                        yes: Uint128::new(2000),
                        no: Uint128::zero(),
                        abstain: Uint128::zero(),
                    },
                    allow_revoting: false
                },
            })
            .unwrap()
        }
    );
}

#[test]
fn test_dao_multiple_proposal_ready() {
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
        voting_strategy: voting_strategy.clone(),
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
        token_contract.clone(),
        &msg,
        &[],
    )
    .unwrap();
    app.update_block(next_block);

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

    // It is not ready to execute yet, so false
    let res: QueryResponse = app
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
    assert_eq!(
        res,
        QueryResponse {
            result: false,
            data: to_binary(&dao_proposal_multiple::query::ProposalResponse {
                id: 1,
                proposal: MultipleChoiceProposal {
                    title: "Cron".to_string(),
                    description: "Cat".to_string(),
                    proposer: Addr::unchecked(CREATOR_ADDR),
                    start_height: 12346,
                    min_voting_period: None,
                    expiration: Expiration::AtHeight(12352),
                    total_power: Uint128::new(2000),
                    status: dao_voting::status::Status::Open,
                    votes: MultipleChoiceVotes {
                        vote_weights: vec![Uint128::zero(), Uint128::zero(), Uint128::zero()]
                    },
                    allow_revoting: false,
                    choices: vec![
                        CheckedMultipleChoiceOption {
                            index: 0,
                            option_type: MultipleChoiceOptionType::Standard,
                            title: "A".to_string(),
                            description: "a".to_owned(),
                            msgs: vec![],
                            vote_count: Uint128::zero()
                        },
                        CheckedMultipleChoiceOption {
                            index: 1,
                            option_type: MultipleChoiceOptionType::Standard,
                            title: "B".to_string(),
                            description: "b".to_owned(),
                            msgs: vec![],
                            vote_count: Uint128::zero()
                        },
                        CheckedMultipleChoiceOption {
                            index: 2,
                            option_type: MultipleChoiceOptionType::None,
                            title: "C".to_string(),
                            description: "None of the above".to_owned(),
                            msgs: vec![],
                            vote_count: Uint128::zero()
                        }
                    ],
                    voting_strategy: voting_strategy.clone()
                },
            })
            .unwrap()
        }
    );

    // Approve proposal
    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &dao_proposal_multiple::msg::ExecuteMsg::Vote {
            proposal_id: 1,
            vote: MultipleChoiceVote { option_id: 0 },
        },
        &[],
    )
    .unwrap();

    // It's now ready to be executed
    let res: QueryResponse = app
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
    assert_eq!(
        res,
        QueryResponse {
            result: true,
            data: to_binary(&dao_proposal_multiple::query::ProposalResponse {
                id: 1,
                proposal: MultipleChoiceProposal {
                    title: "Cron".to_string(),
                    description: "Cat".to_string(),
                    proposer: Addr::unchecked(CREATOR_ADDR),
                    start_height: 12346,
                    min_voting_period: None,
                    expiration: cw_utils::Expiration::AtHeight(12352),
                    total_power: Uint128::new(2000),
                    status: dao_voting::status::Status::Passed,
                    votes: MultipleChoiceVotes {
                        vote_weights: vec![Uint128::new(2000), Uint128::zero(), Uint128::zero()]
                    },
                    allow_revoting: false,
                    choices: vec![
                        CheckedMultipleChoiceOption {
                            index: 0,
                            option_type: MultipleChoiceOptionType::Standard,
                            title: "A".to_owned(),
                            description: "a".to_owned(),
                            msgs: vec![],
                            vote_count: Uint128::zero()
                        },
                        CheckedMultipleChoiceOption {
                            index: 1,
                            option_type: MultipleChoiceOptionType::Standard,
                            title: "B".to_owned(),
                            description: "b".to_owned(),
                            msgs: vec![],
                            vote_count: Uint128::zero()
                        },
                        CheckedMultipleChoiceOption {
                            index: 2,
                            option_type: MultipleChoiceOptionType::None,
                            title: "C".to_owned(),
                            description: "None of the above".to_owned(),
                            msgs: vec![],
                            vote_count: Uint128::zero()
                        }
                    ],
                    voting_strategy: voting_strategy.clone()
                },
            })
            .unwrap()
        }
    );

    app.execute_contract(
        Addr::unchecked(CREATOR_ADDR),
        govmod_single.clone(),
        &dao_proposal_multiple::msg::ExecuteMsg::Execute { proposal_id: 1 },
        &[],
    )
    .unwrap();

    // It's executed now
    // Test if other types of status works
    let res: QueryResponse = app
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
    assert_eq!(
        res,
        QueryResponse {
            result: true,
            data: to_binary(&dao_proposal_multiple::query::ProposalResponse {
                id: 1,
                proposal: MultipleChoiceProposal {
                    title: "Cron".to_string(),
                    description: "Cat".to_string(),
                    proposer: Addr::unchecked(CREATOR_ADDR),
                    start_height: 12346,
                    min_voting_period: None,
                    expiration: cw_utils::Expiration::AtHeight(12352),
                    total_power: Uint128::new(2000),
                    status: dao_voting::status::Status::Executed,
                    votes: MultipleChoiceVotes {
                        vote_weights: vec![Uint128::new(2000), Uint128::zero(), Uint128::zero()]
                    },
                    allow_revoting: false,
                    choices: vec![
                        CheckedMultipleChoiceOption {
                            index: 0,
                            option_type: MultipleChoiceOptionType::Standard,
                            title: "A".to_owned(),
                            description: "a".to_owned(),
                            msgs: vec![],
                            vote_count: Uint128::zero()
                        },
                        CheckedMultipleChoiceOption {
                            index: 1,
                            option_type: MultipleChoiceOptionType::Standard,
                            title: "B".to_owned(),
                            description: "b".to_owned(),
                            msgs: vec![],
                            vote_count: Uint128::zero()
                        },
                        CheckedMultipleChoiceOption {
                            index: 2,
                            option_type: MultipleChoiceOptionType::None,
                            title: "C".to_owned(),
                            description: "None of the above".to_owned(),
                            msgs: vec![],
                            vote_count: Uint128::zero()
                        }
                    ],
                    voting_strategy: voting_strategy.clone()
                },
            })
            .unwrap()
        }
    );
}
