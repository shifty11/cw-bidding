use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin};

use crate::state::Config;

#[cw_serde]
pub struct InstantiateMsg {
    pub commodity: String,
    pub owner: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    MakeBid {},
    Close {},
    Retract {
        receiver: Option<String>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(BidsResponse)]
    Bids {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}

#[cw_serde]
pub struct Bid {
    pub address: Addr,
    pub coin: Coin,
}

#[cw_serde]
pub struct BidsResponse {
    pub bids: Vec<Bid>,
}
