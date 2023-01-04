use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Owner cannot bid")]
    OwnerCannotBid {},

    #[error("Empty bid")]
    EmptyBid {},

    #[error("Bid is too low: amount {amount}; required {required}")]
    BidTooLow {amount: Uint128, required: Uint128},

    #[error("No retractable bid")]
    NoRectractableBid {},
}
