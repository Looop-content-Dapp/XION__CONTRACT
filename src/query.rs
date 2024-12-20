use cosmwasm_std::{Deps, Env, StdResult};
use crate::msg::{ ValidityResponse, ConfigResponse};
use crate::state::{Contract, CONFIG, PassStatus};



//check if token is still valid, not in grace period 
pub fn query_validity(deps: Deps, env: Env, token_id: String) -> StdResult<ValidityResponse> {
    let contract = Contract::default();
    let token = contract.tokens.load(deps.storage, &token_id)?;
    
    let status = token.extension.status(env.block.time);
    let is_valid = matches!(status, PassStatus::Active | PassStatus::InGracePeriod);
    
    Ok(ValidityResponse {
        token_id,
        is_valid,
        expires_at: token.extension.expires_at,
        in_grace_period: matches!(status, PassStatus::InGracePeriod),
        grace_period_end: if is_valid { Some(token.extension.grace_period_end) } else { None },
    })
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    
    Ok(ConfigResponse {
        pass_price: config.pass_price,
        pass_duration: config.pass_duration,
        grace_period: config.grace_period,
        payment_address: config.payment_address.to_string(),
    })
}

