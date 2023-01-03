use cosmwasm_std::{Addr, coin, coins, Decimal};
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
    assert_eq!(app.wrap().query_all_balances(owner).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(contract.addr()).unwrap(), coins(10, ATOM));

    let resp: BidsResponse = contract.query_bids(&app).unwrap();

    assert_eq!(resp, BidsResponse { bids: vec![Bid { address: sender, coin: coin(10, ATOM) }] });
}
