use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub commodity: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const BIDS: Map<Addr, Coin> = Map::new("bids");
