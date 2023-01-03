use cosmwasm_std::{Binary, coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{BIDS, Config, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DENOM: &str = "atom";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = msg.owner.unwrap_or_else(|| info.sender.to_string());
    let validated_owner = deps.api.addr_validate(&owner)?;
    let config = Config {
        owner: validated_owner.clone(),
        commodity: msg.commodity,
    };
    CONFIG.save(deps.storage, &config)?;

    let empty_bid = coin(0, DENOM);
    let bid = info.funds.iter().find(|coin| {
        coin.denom == DENOM
    }).unwrap_or_else(|| {
        &empty_bid
    });

    BIDS.save(deps.storage, validated_owner, bid)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    unimplemented!()
}

pub mod exec {}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {

    match msg {
        QueryMsg::Config {} => to_binary(&query::query_config(deps)?),
        QueryMsg::Bids { } => to_binary(&query::query_bids(deps)?),
    }
}


pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::msg::{Bid, BidsResponse, ConfigResponse};
    use crate::state::{BIDS, CONFIG};

    pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
        let config = CONFIG.load(deps.storage)?;
        Ok(ConfigResponse { config })
    }

    pub fn query_bids(deps: Deps) -> StdResult<BidsResponse> {
        let bids = BIDS
            .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
            .map(|item| {
                let (address, coin) = item?;
                Ok(Bid {
                    address,
                    coin,
                })
            })
            .collect::<StdResult<_>>()?;
        Ok(BidsResponse { bids })
    }
}
