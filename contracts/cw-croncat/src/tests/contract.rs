use crate::contract::GAS_ADJUSTMENT_NUMERATOR_DEFAULT;
use crate::contract::GAS_DENOMINATOR;
use crate::contract::GAS_NUMERATOR_DEFAULT;
use crate::state::QueueItem;
use crate::tests::helpers::mock_init;
use crate::tests::helpers::AGENT0;
use crate::tests::helpers::NATIVE_DENOM;
use crate::ContractError;
use crate::CwCroncat;
use crate::InstantiateMsg;
use cosmwasm_std::testing::{
    mock_dependencies_with_balance, mock_env, mock_info, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{coins, from_binary, Addr, Binary, Event, Reply, SubMsgResponse, SubMsgResult};
use cw_croncat_core::msg::{GetConfigResponse, QueryMsg};
use cw_croncat_core::types::GasPrice;
use cw_croncat_core::types::SlotType;

#[test]
fn configure() {
    let mut deps = mock_dependencies_with_balance(&coins(200, ""));
    let mut store = CwCroncat::default();

    let msg = InstantiateMsg {
        denom: NATIVE_DENOM.to_string(),
        owner_id: None,
        chain_name: "atom".to_string(),
        gas_action_fee: None,
        gas_query_fee: None,
        gas_wasm_query_fee: None,
        gas_price: None,
        agent_nomination_duration: Some(360),
        cw_rules_addr: "todo".to_string(),
        gas_base_fee: None,
    };
    let info = mock_info("creator", &coins(1000, "meow"));

    // we can just call .unwrap() to assert this was a success
    let res = store
        .instantiate(deps.as_mut(), mock_env(), info.clone(), msg)
        .unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = store
        .query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {})
        .unwrap();
    let value: GetConfigResponse = from_binary(&res).unwrap();
    assert_eq!(false, value.paused);
    assert_eq!(info.sender, value.owner_id);
    // assert_eq!(None, value.treasury_id);
    assert_eq!(3, value.min_tasks_per_agent);
    assert_eq!(
        vec![(SlotType::Block, 0, 0), (SlotType::Cron, 0, 0)],
        value.agent_active_indices
    );
    assert_eq!(600, value.agents_eject_threshold);
    assert_eq!("atom", value.native_denom);
    assert_eq!(5, value.agent_fee);
    assert_eq!(
        GasPrice {
            numerator: GAS_NUMERATOR_DEFAULT,
            denominator: GAS_DENOMINATOR,
            gas_adjustment_numerator: GAS_ADJUSTMENT_NUMERATOR_DEFAULT,
        },
        value.gas_price
    );
    assert_eq!(3, value.proxy_callback_gas);
    assert_eq!(10_000_000_000, value.slot_granularity_time);
}

#[test]
fn replies() {
    let mut deps = mock_dependencies_with_balance(&coins(200, ""));
    let store = CwCroncat::default();
    mock_init(&store, deps.as_mut()).unwrap();
    let task_hash = "ad15b0f15010d57a51ff889d3400fe8d083a0dab2acfc752c5eb55e9e6281705"
        .as_bytes()
        .to_vec();
    let response = SubMsgResponse {
        data: Some(Binary::from_base64("MTMzNw==").unwrap()),
        events: vec![Event::new("wasm").add_attribute("cat", "meow")],
    };

    let mut msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(response),
    };

    // Check there wasn't any known reply
    let res_err1 = store
        .reply(deps.as_mut(), mock_env(), msg.clone())
        .unwrap_err();
    assert_eq!(ContractError::UnknownReplyID {}, res_err1);

    // Create fake Queue item, check that it gets removed, returns default reply_id
    // TODO: Dont think it's possible to create fake queue items
    // store
    //     .rq_push(
    //         deps.as_mut().storage,
    //         QueueItem {
    //             action_idx: 0,
    //             task_hash: Some(task_hash.clone()),
    //             contract_addr: None,
    //             task_is_extra: Some(false),
    //             agent_id: Some(Addr::unchecked(AGENT0)),
    //             failed: false,
    //         },
    //     )
    //     .unwrap();
    // let queue_item1 = store
    //     .reply_queue
    //     .may_load(deps.as_mut().storage, msg.id)
    //     .unwrap();
    // assert!(queue_item1.is_some());

    // let res1 = store.reply(deps.as_mut(), mock_env(), msg.clone()).unwrap();
    // let mut has_reply_id: bool = false;
    // for a in res1.attributes {
    //     if a.key == "reply_id" && a.value == "1" {
    //         has_reply_id = true;
    //     }
    // }
    // assert!(has_reply_id);
    // let queue_item2 = store
    //     .reply_queue
    //     .may_load(deps.as_mut().storage, msg.id)
    //     .unwrap();
    // assert!(queue_item2.is_none());

    // Create fake Queue item with known contract address,
    // check that it gets removed, the rest is covered in proxy_callback tests
    store
        .rq_push(
            deps.as_mut().storage,
            QueueItem {
                action_idx: 0,
                task_hash: Some(task_hash),
                contract_addr: Some(Addr::unchecked(MOCK_CONTRACT_ADDR)),
                task_is_extra: Some(false),
                agent_id: Some(Addr::unchecked(AGENT0)),
                failure: None,
            },
        )
        .unwrap();
    msg.id = 1;
    let queue_item3 = store
        .reply_queue
        .may_load(deps.as_mut().storage, msg.id)
        .unwrap();
    assert!(queue_item3.is_some());

    let res_err2 = store
        .reply(deps.as_mut(), mock_env(), msg.clone())
        .unwrap_err();
    assert_eq!(ContractError::NoTaskFound {}, res_err2);
    // It can't get removed, because contract will rollback to original state at failure
    // TODO: retest it with integration tests
    // let queue_item4 = store
    //     .reply_queue
    //     .may_load(deps.as_mut().storage, msg.id)
    //     .unwrap();
    // assert!(queue_item4.is_some());
}

// TODO: make it for every item in cw_croncat
#[test]
pub fn tasks_with_queries_total_initialized() {
    let mut deps = mock_dependencies_with_balance(&coins(200, ""));
    let store = CwCroncat::default();
    mock_init(&store, deps.as_mut()).unwrap();

    let total = store
        .tasks_with_queries_total
        .may_load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(total, Some(0));
}
