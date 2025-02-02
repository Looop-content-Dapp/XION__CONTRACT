#[cfg(not(feature = "library"))]

use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw721_base_soulbound::state::TokenInfo;
use crate::error::ContractError;
use crate::state::{Contract, PassExtension, CONFIG, TOKEN_ID_COUNTER};
use crate::state::PassStatus;
use crate::helpers::validate_payment;
// use crate::msg::{ExecuteMsg, PassMsg};

pub fn mint_pass(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage).unwrap();

    // Validate payment
    validate_payment(&info, config.pass_price)?;

    // Get and increment token ID counter
    let counter = TOKEN_ID_COUNTER.load(deps.storage)?;
    let token_id = format!("PASS{}", counter + 1);
    TOKEN_ID_COUNTER.save(deps.storage, &(counter + 1))?;

    // Create new pass extension with timestamps
    let extension = PassExtension::new(
        env.block.time,
        config.pass_duration,
        config.grace_period,
    );

    // Create token using base contract's functionality
    let contract = Contract::default();

    contract.tokens.update(deps.storage, &token_id, |old| match old {
        Some(_) => Err(ContractError::Custom("Token ID already exists".to_string())),
        None => Ok(TokenInfo {
            owner: info.sender.clone(),
            approvals: vec![],
            token_uri: None,
            extension,
        }),
    })?;

    // Increment token count
    contract.increment_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "mint_pass")
        .add_attribute("minter", info.sender)
        .add_attribute("token_id", token_id))
}

pub fn renew_pass(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    
    // Validate payment
    validate_payment(&info, config.pass_price)?;

    let contract = Contract::default();
    let mut token = contract.tokens.load(deps.storage, &token_id)?;

    // Check ownership
    if token.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Renew the pass
    token.extension.renew(
        env.block.time,
        config.pass_duration,
        config.grace_period,
    );

    // Save updated token
    contract.tokens.save(deps.storage, &token_id, &token)?;

    Ok(Response::new()
        .add_attribute("action", "renew_pass")
        .add_attribute("token_id", token_id)
        .add_attribute("owner", info.sender)
        .add_attribute("new_expiry", token.extension.expires_at.to_string()))
}

pub fn burn_expired_pass(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let contract = Contract::default();
    let token = contract.tokens.load(deps.storage, &token_id)?;

    // Check if pass is expired
    if token.extension.status(env.block.time) != PassStatus::Expired {
        return Err(ContractError::Custom("Pass is not expired".to_string()));
    }

    // Check if caller is owner
    if token.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // Remove token
    contract.tokens.remove(deps.storage, &token_id)?;
    contract.decrement_tokens(deps.storage)?;

    Ok(Response::new()
        .add_attribute("action", "burn_expired_pass")
        .add_attribute("token_id", token_id)
        .add_attribute("owner", info.sender))
}




