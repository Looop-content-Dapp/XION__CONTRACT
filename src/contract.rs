#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError, to_json_binary};
use cw2::set_contract_version;



use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, PassMsg};
use crate::state::{CONFIG, Config};
use crate::execute::{mint_pass, renew_pass, burn_expired_pass};
use crate::query::{query_config, query_validity};
use crate::msg::PassQuery;
use crate::state::Contract;
use crate::helpers::convert_query_msg;

// Version info for migration info
const CONTRACT_NAME: &str = "crates.io:loop_music";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        pass_price: msg.pass_price,
        pass_duration: msg.pass_duration,
        grace_period: msg.grace_period,
        payment_address: deps.api.addr_validate(&msg.payment_address.to_string())?,
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Extension { msg } => match msg {
            PassMsg::MintPass { token_id } => mint_pass(deps, env, info, token_id),
            PassMsg::RenewPass { token_id } => renew_pass(deps, env, info, token_id),
            PassMsg::BurnExpiredPass { token_id } => burn_expired_pass(deps, env, info, token_id),
        },
        _ => Err(ContractError::Custom("Unsupported operation".to_string())),
    }
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    msg: cw721_base_soulbound::QueryMsg<PassQuery>,
) -> StdResult<Binary> {
    let contract = Contract::default();
    
    match msg {
        QueryMsg::Extension { msg } => match msg {
            PassQuery::CheckValidity { token_id } => to_json_binary(&query_validity(deps, env, token_id)?),
            PassQuery::GetConfig {} => to_json_binary(&query_config(deps)?),
        },
        base_query => {
            let converted_msg = convert_query_msg(base_query)
                .map_err(|e| StdError::generic_err(format!("{}", e)))?;
            contract.query(deps, env, converted_msg)
        }
    }
}

