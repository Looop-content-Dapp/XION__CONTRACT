#[cfg(test)]
mod tests {
    use crate::contract::{instantiate, execute, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg, PassMsg, PassQuery, ConfigResponse, ValidityResponse};
    // use crate::state::{Config, PassStatus};
    use cosmwasm_std::{from_json, Addr, coins, OwnedDeps, testing::{MockApi, MockQuerier, MockStorage}};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    // Constants for testing
    const PASS_PRICE: u128 = 1;
    const PASS_DURATION: u64 = 20 * 60; // 20 mins in seconds for testing
    const GRACE_PERIOD: u64 = 5 * 60;   // 5 mins in seconds for testing
    const USER: &str = "user";
    const MINTER: &str = "minter";
    const PAYMENT_ADDR: &str = "payment_addr";

    // Helper function to setup contract
    fn setup_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let minter = Addr::unchecked(MINTER);
        let payment_address = Addr::unchecked(PAYMENT_ADDR);

        let msg = InstantiateMsg {
            name: "Test Pass".to_string(),
            symbol: "PASS".to_string(),
            minter: minter.to_string(),
            pass_price: PASS_PRICE,
            pass_duration: PASS_DURATION,
            grace_period: GRACE_PERIOD,
            payment_address: payment_address.clone(),
        };

        let info = mock_info(MINTER, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        deps
    }

    #[test]
    fn test_initialization() {
        println!("\n=== Testing Contract Instantiation ===");
        let mut deps = mock_dependencies();
        let minter = Addr::unchecked(MINTER);
        let payment_address = Addr::unchecked(PAYMENT_ADDR);

        let info = mock_info(MINTER, &[]);

        let msg = InstantiateMsg {
            name: "Test Pass".to_string(),
            symbol: "PASS".to_string(),
            minter: minter.to_string(),
            pass_price: PASS_PRICE,
            pass_duration: PASS_DURATION,
            grace_period: GRACE_PERIOD,
            payment_address: payment_address.clone(),
        };

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        
        // Query config
        let msg = QueryMsg::Extension { 
            msg: PassQuery::GetConfig {} 
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let config: ConfigResponse = from_json(&res).unwrap();
        
        assert_eq!(config.pass_price, PASS_PRICE);
        assert_eq!(config.pass_duration, PASS_DURATION);
        assert_eq!(config.grace_period, GRACE_PERIOD);
        assert_eq!(config.payment_address, payment_address);
    }

    #[test]
    fn test_mint_pass() {
        let mut deps = setup_contract();
        let token_id = "pass1".to_string();

        // Try minting without payment
        let info = mock_info(USER, &[]);
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                token_id: token_id.clone() 
            } 
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
        assert_eq!(err.to_string(), "No uxion payment found");

        // Mint with correct payment
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert!(!res.attributes.is_empty());

        // Query pass validity
        let msg = QueryMsg::Extension { 
            msg: PassQuery::CheckValidity { 
                token_id: token_id.clone() 
            } 
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let validity: ValidityResponse = from_json(&res).unwrap();
        
        assert!(validity.is_valid);
        assert!(!validity.in_grace_period);
    }

    #[test]
    fn test_renew_pass() {
        let mut deps = setup_contract();
        let token_id = "pass1".to_string();

        // First mint a pass
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let mint_msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                token_id: token_id.clone() 
            } 
        };
        execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // Try renewing without payment
        let info = mock_info(USER, &[]);
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::RenewPass { 
                token_id: token_id.clone() 
            } 
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
        assert_eq!(err.to_string(), "No uxion payment found");

        // Renew with correct payment
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert!(res.attributes.iter().any(|attr| attr.key == "action" && attr.value == "renew_pass"));
    }

    #[test]
    fn test_burn_expired_pass() {
        let mut deps = setup_contract();
        let token_id = "pass1".to_string();

        // Mint a pass
        let info = mock_info(USER, &coins(PASS_PRICE, "uxion"));
        let mint_msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass { 
                token_id: token_id.clone() 
            } 
        };
        execute(deps.as_mut(), mock_env(), info.clone(), mint_msg).unwrap();

        // Try burning an active pass
        let burn_msg = ExecuteMsg::Extension { 
            msg: PassMsg::BurnExpiredPass { 
                token_id: token_id.clone() 
            } 
        };
        let err = execute(deps.as_mut(), mock_env(), info.clone(), burn_msg.clone()).unwrap_err();
        assert!(err.to_string().contains("Pass must be expired to burn"));

        // Move time past expiration and grace period
        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(PASS_DURATION + GRACE_PERIOD + 1);

        // Now burn should succeed
        let res = execute(deps.as_mut(), env, info, burn_msg).unwrap();
        assert!(res.attributes.iter().any(|attr| attr.key == "action" && attr.value == "burn_expired_pass"));
    }
}