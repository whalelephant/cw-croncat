use cosmwasm_std::{coin, Addr};
use cw20::Cw20CoinVerified;

use crate::types::AmountForOneTask;

#[test]
fn amount_for_one_task_add_gas() {
    let mut amount = AmountForOneTask {
        gas: 0,
        cw20: None,
        coin: [None, None],
    };

    // amount.gas < limit
    assert!(amount.add_gas(10, 11));
    assert_eq!(amount.gas, 10);

    // amount.gas == limit
    assert!(amount.add_gas(5, 15));
    assert_eq!(amount.gas, 15);

    // amount.gas > limit
    assert!(!amount.add_gas(3, 17));
    assert_eq!(amount.gas, 18);
}

#[test]
fn amount_for_one_task_add_coin() {
    let mut amount = AmountForOneTask {
        gas: 0,
        cw20: None,
        coin: [None, None],
    };

    // Add the first coin
    let mut coin1 = coin(10, "denom1".to_string());
    assert!(amount.add_coin(coin1.clone()).unwrap());
    assert_eq!(amount.coin, [Some(coin1.clone()), None]);

    // Add the second coin
    let mut coin2 = coin(100, "denom2".to_string());
    assert!(amount.add_coin(coin2.clone()).unwrap());
    assert_eq!(amount.coin, [Some(coin1), Some(coin2.clone())]);

    // Add coin with the first denom
    coin1 = coin(20, "denom1".to_string());
    assert!(amount.add_coin(coin1).unwrap());
    assert_eq!(
        amount.coin,
        [Some(coin(30, "denom1".to_string())), Some(coin2.clone())]
    );

    // Add coin with the second denom
    coin2 = coin(200, "denom2".to_string());
    assert!(amount.add_coin(coin2).unwrap());
    assert_eq!(
        amount.coin,
        [
            Some(coin(30, "denom1".to_string())),
            Some(coin(300, "denom2".to_string()))
        ]
    );

    // Add coin with a new denom, return false
    let another_coin = coin(1, "denom3".to_string());
    assert!(!amount.add_coin(another_coin).unwrap());
    assert_eq!(
        amount.coin,
        [
            Some(coin(30, "denom1".to_string())),
            Some(coin(300, "denom2".to_string()))
        ]
    );
}

#[test]
fn amount_for_one_task_add_cw20() {
    let mut amount = AmountForOneTask {
        gas: 0,
        cw20: None,
        coin: [None, None],
    };

    // Add cw20 coin
    let mut cw20 = Cw20CoinVerified {
        address: Addr::unchecked("addr"),
        amount: 1u64.into(),
    };
    assert!(amount.add_cw20(cw20.clone()));
    assert_eq!(amount.cw20, Some(cw20));

    // Add cw20 coin with the same address
    cw20 = Cw20CoinVerified {
        address: Addr::unchecked("addr"),
        amount: 10u64.into(),
    };
    assert!(amount.add_cw20(cw20.clone()));
    assert_eq!(
        amount.cw20,
        Some(Cw20CoinVerified {
            address: Addr::unchecked("addr"),
            amount: 11u64.into(),
        })
    );

    // Add cw20 coin with a wrong address
    cw20 = Cw20CoinVerified {
        address: Addr::unchecked("addr2"),
        amount: 10u64.into(),
    };
    assert!(!amount.add_cw20(cw20));
    assert_eq!(
        amount.cw20,
        Some(Cw20CoinVerified {
            address: Addr::unchecked("addr"),
            amount: 11u64.into(),
        })
    );
}
