use cosmwasm_std::{Binary, coin, Coin, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{Bid, ExecuteMsg, InstantiateMsg, QueryMsg};
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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MakeBid {} => exec::make_bid(deps, env, info),
        ExecuteMsg::Close {} => exec::close(deps, env, info),
        ExecuteMsg::Retract { receiver } => exec::retract(deps, env, info, receiver),
    }
}

pub mod exec {
    use cosmwasm_std::{BankMsg, coin, DepsMut, Env, MessageInfo, Response, Uint128};

    use crate::{ContractError};
    use crate::contract::{Commission, DENOM, query};
    use crate::msg::Bid;
    use crate::state::{BIDS, CONFIG};

    pub fn make_bid(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if info.sender == config.owner {
            return Err(ContractError::OwnerCannotBid {});
        }

        let fund = info.funds.iter().find(|coin| {
            coin.denom == DENOM
        });
        if fund.is_none() {
            return Err(ContractError::EmptyBid {});
        }

        let resp = query::query_bids(deps.as_ref())?;
        let new_bid = Bid { address: info.sender.clone(), coin: fund.unwrap().clone() };
        let summarized_bid: Bid = resp.bids.iter().find(|bid| {
            bid.address == info.sender
        }).map(|bid| {
            let amount = bid.coin.amount + new_bid.coin.amount;
            Bid { address: info.sender.clone(), coin: coin(amount.u128(), DENOM) }
        }).unwrap_or_else(|| {
            let amount = new_bid.coin.amount;
            Bid { address: info.sender.clone(), coin: coin(amount.u128(), DENOM) }
        });

        let old_bid = resp.bids.first();
        if old_bid.is_some() && old_bid.unwrap().address != info.sender.clone() {
            if old_bid.unwrap().coin.amount >= summarized_bid.coin.amount {
                return Err(ContractError::BidTooLow {
                    amount: summarized_bid.coin.amount,
                    required: old_bid.unwrap().coin.amount,
                });
            }
        }

        BIDS.save(deps.storage, info.sender.clone(), &summarized_bid.coin)?;

        let mut resp = Response::new()
            .add_attribute("action", "bid")
            .add_attribute("sender", info.sender.as_str());

        if new_bid.commission_as_coin().amount > Uint128::zero() {
            let bank_msg = BankMsg::Send {
                to_address: config.owner.to_string(),
                amount: vec![new_bid.commission_as_coin()],
            };
            resp = resp.add_message(bank_msg);
        }

        Ok(resp)
    }

    pub fn close(deps: DepsMut, _env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
        if info.sender != config.owner {
            return Err(ContractError::Unauthorized {});
        }

        let mut resp = Response::new()
            .add_attribute("action", "close")
            .add_attribute("sender", info.sender.as_str());

        let result = query::query_bids(deps.as_ref())?;
        let highest_bid = result.bids.first();
        if highest_bid.is_some() && highest_bid.unwrap().coin.amount != Uint128::zero() {
            let bank_msg = BankMsg::Send {
                to_address: info.sender.to_string(),
                amount: vec![highest_bid.unwrap().amount_as_coin()],
            };

            resp = resp.add_message(bank_msg);
        }

        Ok(resp)
    }

    pub fn retract(deps: DepsMut, _env: Env, info: MessageInfo, receiver: Option<String>) -> Result<Response, ContractError> {
        let receiver = receiver.unwrap_or_else(|| info.sender.to_string());
        let validated_receiver = deps.api.addr_validate(&receiver)?;

        let result = query::query_bids(deps.as_ref())?;
        let split: Option<(&Bid, &[Bid])> = result.bids.split_first();
        if split.is_none() {
            return Err(ContractError::NoRectractableBid {});
        }

        let bid: &Bid = split.unwrap().1.iter().find(|bid| {
            bid.address == info.sender
        }).ok_or(ContractError::NoRectractableBid {})?;

        if bid.coin.amount == Uint128::zero() {
            return Err(ContractError::NoRectractableBid {});
        }

        let mut resp = Response::new()
            .add_attribute("action", "retract")
            .add_attribute("sender", info.sender.as_str());

        let bank_msg = BankMsg::Send {
            to_address: validated_receiver.to_string(),
            amount: vec![coin(bid.amount_as_coin().amount.u128(), DENOM)],
        };
        resp = resp.add_message(bank_msg);

        Ok(resp)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::query_config(deps)?),
        QueryMsg::Bids {} => to_binary(&query::query_bids(deps)?),
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
        let mut bids: Vec<Bid> = BIDS
            .range(deps.storage, None, None, cosmwasm_std::Order::Descending)
            .map(|item| {
                let (address, coin) = item?;
                Ok(Bid {
                    address,
                    coin,
                })
            })
            .collect::<StdResult<_>>()?;
        bids.sort_by(|a, b| {
            b.coin.amount.cmp(&a.coin.amount)
        });
        Ok(BidsResponse { bids })
    }
}


const DEFAULT_COMMISSION: u64 = 10;

pub trait Commission {
    fn commission(&self) -> Decimal;
    fn commission_as_coin(&self) -> Coin;
    fn amount(&self) -> Decimal;
    fn amount_as_coin(&self) -> Coin;
}

impl Commission for Bid {
    fn commission(&self) -> Decimal {
        Decimal::new(self.coin.amount) * Decimal::percent(DEFAULT_COMMISSION)
    }

    fn commission_as_coin(&self) -> Coin {
        coin(self.commission().atomics().u128(), DENOM)
    }

    fn amount(&self) -> Decimal {
        Decimal::new(self.coin.amount) - self.commission()
    }

    fn amount_as_coin(&self) -> Coin {
        coin( self.amount().atomics().u128(), DENOM)
    }
}
