use cosmwasm_std::{Addr, coin, coins, Uint128};
use cw_multi_test::App;

use crate::error::ContractError;
use crate::msg::{Bid, BidsResponse, ConfigResponse};
use crate::state::Config;

use super::contract::BiddingContract;

const ATOM: &str = "atom";

#[test]
fn instantiate() {
    let mut app = App::default();

    let contract_id = BiddingContract::store_code(&mut app);

    let owner = Addr::unchecked("owner");
    let commodity = "gold".to_string();

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &Addr::unchecked("sender"),
        &owner,
        None,
        commodity.as_str(),
        None,
    ).unwrap();

    let resp: ConfigResponse = contract.query_config(&app).unwrap();

    assert_eq!(resp, ConfigResponse { config: Config { owner: owner.clone(), commodity } });

    let resp: BidsResponse = contract.query_bids(&app).unwrap();

    assert_eq!(resp, BidsResponse { bids: vec![Bid { address: owner, coin: coin(0, ATOM) }] });
}

#[test]
fn instantiate_with_bid() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, "atom"))
            .unwrap();
    });

    let contract_id = BiddingContract::store_code(&mut app);

    let commodity = "gold".to_string();

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        &owner,
        None,
        commodity.as_str(),
        10,
    ).unwrap();

    let resp: ConfigResponse = contract.query_config(&app).unwrap();

    assert_eq!(resp, ConfigResponse { config: Config { owner: owner.clone(), commodity } });
    assert_eq!(app.wrap().query_all_balances(owner.clone()).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), coins(10, ATOM));

    let resp: BidsResponse = contract.query_bids(&app).unwrap();

    assert_eq!(resp, BidsResponse { bids: vec![Bid { address: owner, coin: coin(10, ATOM) }] });
}

#[test]
fn bid_owner_error() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &owner, coins(10, "atom"))
            .unwrap();
    });

    let contract_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        &owner,
        None,
        None,
        None,
    ).unwrap();

    let err = contract
        .make_bid(&mut app, &owner, &coins(2, ATOM))
        .unwrap_err();

    assert_eq!(err,  ContractError::OwnerCannotBid {});
}

#[test]
fn bid_empty_bid() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::default();

    let contract_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender,
        &owner,
        None,
        None,
        None,
    ).unwrap();

    let err = contract
        .make_bid(&mut app, &sender, &[])
        .unwrap_err();

    assert_eq!(err,  ContractError::EmptyBid {});
}

#[test]
fn bid_success() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(20, "atom"))
            .unwrap();
        router
            .bank
            .init_balance(storage, &sender2, coins(20, "atom"))
            .unwrap();
    });

    let contract_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender1,
        &owner,
        None,
        None,
        None,
    ).unwrap();

    contract
        .make_bid(&mut app, &sender1, &coins(10, ATOM))
        .unwrap();

    assert_eq!(app.wrap().query_all_balances(sender1.clone()).unwrap(), coins(10, ATOM));
    assert_eq!(app.wrap().query_all_balances(sender2.clone()).unwrap(), coins(20, ATOM));
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), coins(9, ATOM));
    assert_eq!(app.wrap().query_all_balances(owner.clone()).unwrap(), coins(1, ATOM));

    let resp: BidsResponse = contract.query_bids(&app).unwrap();

    assert_eq!(resp, BidsResponse { bids: vec![
        Bid { address: sender1.clone(), coin: coin(10, ATOM) },
        Bid { address: owner.clone(), coin: coin(0, ATOM) },
    ] });

    contract
        .make_bid(&mut app, &sender2, &coins(12, ATOM))
        .unwrap();

    assert_eq!(app.wrap().query_all_balances(sender1.clone()).unwrap(), coins(10, ATOM));
    assert_eq!(app.wrap().query_all_balances(sender2.clone()).unwrap(), coins(8, ATOM));
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), coins(20, ATOM));
    assert_eq!(app.wrap().query_all_balances(owner.clone()).unwrap(), coins(2, ATOM));

    let resp: BidsResponse = contract.query_bids(&app).unwrap();

    assert_eq!(resp, BidsResponse { bids: vec![
        Bid { address: sender2.clone(), coin: coin(12, ATOM) },
        Bid { address: sender1.clone(), coin: coin(10, ATOM) },
        Bid { address: owner.clone(), coin: coin(0, ATOM) },
    ] });

    contract
        .make_bid(&mut app, &sender1, &coins(10, ATOM))
        .unwrap();

    assert_eq!(app.wrap().query_all_balances(sender1.clone()).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(sender2.clone()).unwrap(), coins(8, ATOM));
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), coins(29, ATOM));
    assert_eq!(app.wrap().query_all_balances(owner.clone()).unwrap(), coins(3, ATOM));

    let resp: BidsResponse = contract.query_bids(&app).unwrap();

    assert_eq!(resp, BidsResponse { bids: vec![
        Bid { address: sender1.clone(), coin: coin(20, ATOM) },
        Bid { address: sender2.clone(), coin: coin(12, ATOM) },
        Bid { address: owner.clone(), coin: coin(0, ATOM) },
    ] });

    let err = contract
        .make_bid(&mut app, &sender2, &coins(5, ATOM))
        .unwrap_err();

    assert_eq!(err,  ContractError::BidTooLow { amount: Uint128::new(17), required: Uint128::new(20) });

    let err = contract
        .close(&mut app, &sender2)
        .unwrap_err();

    assert_eq!(err,  ContractError::Unauthorized { });

    contract
        .close(&mut app, &owner)
        .unwrap();

    assert_eq!(app.wrap().query_all_balances(sender1.clone()).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(sender2.clone()).unwrap(), coins(8, ATOM));
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), coins(11, ATOM));
    assert_eq!(app.wrap().query_all_balances(owner.clone()).unwrap(), coins(21, ATOM));

    let err = contract
        .retract(&mut app, &sender1, None)
        .unwrap_err();

    assert_eq!(err,  ContractError::NoRectractableBid { });

    let err = contract
        .retract(&mut app, &owner, None)
        .unwrap_err();

    assert_eq!(err,  ContractError::NoRectractableBid { });

    contract
        .retract(&mut app, &sender2, None)
        .unwrap();

    assert_eq!(app.wrap().query_all_balances(sender1.clone()).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(sender2.clone()).unwrap(), coins(19, ATOM));
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(owner.clone()).unwrap(), coins(21, ATOM));
}

#[test]
fn retract_to_friend() {
    let owner = Addr::unchecked("owner");
    let sender1 = Addr::unchecked("sender1");
    let sender2 = Addr::unchecked("sender2");
    let recipient = Addr::unchecked("recipient");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender1, coins(20, "atom"))
            .unwrap();
        router
            .bank
            .init_balance(storage, &sender2, coins(20, "atom"))
            .unwrap();
    });

    let contract_id = BiddingContract::store_code(&mut app);

    let contract = BiddingContract::instantiate(
        &mut app,
        contract_id,
        &sender1,
        &owner,
        None,
        None,
        None,
    ).unwrap();

    contract
        .make_bid(&mut app, &sender1, &coins(10, ATOM))
        .unwrap();

    contract
        .make_bid(&mut app, &sender2, &coins(15, ATOM))
        .unwrap();

    assert_eq!(app.wrap().query_all_balances(sender1.clone()).unwrap(), coins(10, ATOM));
    assert_eq!(app.wrap().query_all_balances(sender2.clone()).unwrap(), coins(5, ATOM));
    assert_eq!(app.wrap().query_all_balances(recipient.clone()).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), coins(23, ATOM));
    assert_eq!(app.wrap().query_all_balances(owner.clone()).unwrap(), coins(2, ATOM));

    contract
        .close(&mut app, &owner)
        .unwrap();

    contract
        .retract(&mut app, &sender1, &recipient)
        .unwrap();

    assert_eq!(app.wrap().query_all_balances(sender1.clone()).unwrap(), coins(10, ATOM));
    assert_eq!(app.wrap().query_all_balances(sender2.clone()).unwrap(), coins(5, ATOM));
    assert_eq!(app.wrap().query_all_balances(recipient.clone()).unwrap(), coins(9, ATOM));
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(owner.clone()).unwrap(), coins(16, ATOM));
}
