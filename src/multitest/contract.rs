use cosmwasm_std::{Addr, Coin, coin, coins, StdResult};
use cw_multi_test::{App, BasicApp, ContractWrapper, Executor};

use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::msg::{BidsResponse, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

pub struct BiddingContract(Addr);

impl BiddingContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn app_with_funds(sender: impl Into<Option<Addr>>, amount: impl Into<Option<u128>>) -> BasicApp {
        App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &sender.into().unwrap_or_else(|| Addr::unchecked("owner")),
                    coins(amount.into().unwrap_or_else(|| 0), "atom"))
                .unwrap();
        })
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate<'a>(
        app: &mut App,
        code_id: u64,
        sender: impl Into<Option<&'a Addr>>,
        owner: impl Into<Option<&'a Addr>>,
        admin: impl Into<Option<&'a Addr>>,
        commodity: impl Into<Option<&'a str>>,
        bid: impl Into<Option<u128>>,
    ) -> StdResult<Self> {
        let sender = sender.into().cloned().unwrap_or_else(|| Addr::unchecked("sender"));
        let admin = admin.into().map(Addr::to_string);
        let owner = owner.into().map(Addr::to_string);
        let commodity = commodity.into().unwrap_or_else(|| "gold").to_string();
        let bid = bid.into().map(|b| vec![coin(b, "atom")]).unwrap_or_else(|| vec![]);

        app.instantiate_contract(
            code_id,
            sender,
            &InstantiateMsg {
                commodity,
                owner,
            },
            bid.as_slice(),
            "Bidding contract",
            admin,
        )
            .map(BiddingContract)
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn make_bid(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecuteMsg::MakeBid {}, funds)
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn close(
        &self,
        app: &mut App,
        sender: &Addr
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecuteMsg::Close {}, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn retract<'a>(
        &self,
        app: &mut App,
        sender: &Addr,
        receiver: impl Into<Option<&'a Addr>>,
    ) -> Result<(), ContractError> {
        let receiver = receiver.into().map(Addr::to_string);
        app.execute_contract(sender.clone(), self.0.clone(), &ExecuteMsg::Retract {receiver}, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn query_config(&self, app: &App) -> StdResult<ConfigResponse> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Config {})
    }

    #[track_caller]
    pub fn query_bids(&self, app: &App) -> StdResult<BidsResponse> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Bids {})
    }
}

impl From<BiddingContract> for Addr {
    fn from(contract: BiddingContract) -> Self {
        contract.0
    }
}
