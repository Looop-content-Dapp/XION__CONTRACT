#[cfg(test)]
mod tests {
    use cosmwasm_std::{Coin, Empty, Addr};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};
    use crate::contract::{execute, instantiate, query};
    use crate::msg::{InstantiateMsg, ExecuteMsg, QueryMsg, PassMsg, ValidityResponse, ConfigResponse};
    use crate::msg::PassQuery;

    #[test]
    fn test_pass_flow() {
        // Setup test accounts
        let mut app = App::default();
        let owner = Addr::unchecked("owner");
        let user1 = Addr::unchecked("user1");
        let user2 = Addr::unchecked("user2");
        let payment_addr = Addr::unchecked("payment_addr");

       

        // Initialize balances
        app.init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &user1,
                    vec![Coin::new(1000u128, "uxion")]
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &user2,
                    vec![Coin::new(1000u128, "uxion")]
                )
                .unwrap();
        });

        println!("\n=== Starting Pass Flow Test ===");
        println!("Setting up users:");
        println!("Owner: {}", owner);
        println!("Users: {}, {}", user1, user2);
        println!("Payment Address: {}", payment_addr);

        // Upload and instantiate contract
        let contract_id = app.store_code(contract_pass());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    name: "Test Pass".to_string(),
                    symbol: "PASS".to_string(),
                    minter: owner.to_string(),
                    pass_price: 10u128,
                    pass_duration: 20 * 60, // 20 minutes
                    grace_period: 5 * 60,   // 5 minutes
                    payment_address: payment_addr.clone(),
                },
                &[],
                "music-pass",
                None,
            )
            .unwrap();

        // Query initial config
        let config: ConfigResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::Extension { msg: PassQuery::GetConfig {} })
            .unwrap();
        
        println!("\n=== Initial Contract Config ===");
        println!("Pass Price: {}", config.pass_price);
        println!("Pass Duration: {}", config.pass_duration);
        println!("Grace Period: {}", config.grace_period);

        // Print initial balances
        println!("\n=== Initial Balances ===");
        for user in [&user1, &user2] {
            let balance = app
                .wrap()
                .query_balance(user.clone(), "uxion")
                .unwrap();
            println!("Balance of {}: {}", user, balance.amount);
        }

        // Mint passes
        println!("\n=== Minting Passes ===");
        let mut first_token_id = String::new();
        let mut second_token_id = String::new();
    
        for user in [&user1, &user2] {
            let msg = ExecuteMsg::Extension { 
                msg: PassMsg::MintPass {} 
            };
            
            let res = app.execute_contract(
                user.clone(),
                contract_addr.clone(),
                &msg,
                &[Coin::new(10u128, "uxion")],
            ).unwrap();
        
            // Get the token ID from the response
            let token_id = res.events
                .iter()
                .find(|e| e.ty == "wasm")
                .and_then(|e| e.attributes
                    .iter()
                    .find(|attr| attr.key == "token_id")
                    .map(|attr| attr.value.clone())
                )
                .unwrap();
            
            println!("{} minted pass {}", user, token_id);
    
            // Query pass validity after minting
            let validity: ValidityResponse = app
                .wrap()
                .query_wasm_smart(
                    contract_addr.clone(),
                    &QueryMsg::Extension { 
                        msg: PassQuery::CheckValidity { 
                            token_id: token_id.clone() 
                        }
                    },
                )
                .unwrap();
            println!("Pass {} validity: {:?}", token_id, validity);
    
            // Store token IDs for later use
            if user == &user1 {
                first_token_id = token_id;
            } else {
                second_token_id = token_id;
            }
        }
    
        // Test pass renewal with first token
        println!("\n=== Testing Pass Renewal ===");
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::RenewPass { 
                token_id: first_token_id.clone() 
            }
        };
        
        app.execute_contract(
            user1.clone(),
            contract_addr.clone(),
            &msg,
            &[Coin::new(10u128, "uxion")],
        )
        .unwrap();
        println!("User1 renewed {}", first_token_id);
    
        // Test burning expired pass
        println!("\n=== Testing Pass Burning ===");
        app.update_block(|block| {
            block.time = block.time.plus_seconds(30 * 60);
        });
    
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::BurnExpiredPass { 
                token_id: second_token_id.clone() 
            }
        };
        
        app.execute_contract(
            user2.clone(),
            contract_addr.clone(),
            &msg,
            &[],
        )
        .unwrap();
        println!("User2 burned {}", second_token_id);
    
        // Check final balances
        println!("\n=== Final Balances ===");
        for user in [&user1, &user2] {
            let balance = app
                .wrap()
                .query_balance(user.clone(), "uxion")
                .unwrap();
            println!("Final balance of {}: {}", user, balance.amount);
        }
    }

    #[test]
    fn test_auto_increment_token_ids() {
        let mut app = App::default();
        let owner = Addr::unchecked("owner");
        let user1 = Addr::unchecked("user1");
        let user2 = Addr::unchecked("user2");
        let payment_addr = Addr::unchecked("payment_addr");
    
        // Initialize balances
        app.init_modules(|router, _api, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &user1,
                    vec![Coin::new(1000u128, "uxion")]
                )
                .unwrap();
            router
                .bank
                .init_balance(
                    storage,
                    &user2,
                    vec![Coin::new(1000u128, "uxion")]
                )
                .unwrap();
        });
    
        // Upload and instantiate contract
        let contract_id = app.store_code(contract_pass());
        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    name: "Test Pass".to_string(),
                    symbol: "PASS".to_string(),
                    minter: owner.to_string(),
                    pass_price: 10u128,
                    pass_duration: 20 * 60,
                    grace_period: 5 * 60,
                    payment_address: payment_addr.clone(),
                },
                &[],
                "music-pass",
                None,
            )
            .unwrap();
    
        // Mint first pass
        let msg = ExecuteMsg::Extension { 
            msg: PassMsg::MintPass {} 
        };
        
        let res = app.execute_contract(
            user1.clone(),
            contract_addr.clone(),
            &msg,
            &[Coin::new(10u128, "uxion")],
        ).unwrap();
    
        // Check if first token ID is "PASS1"
        assert!(res.events.iter().any(|e| 
            e.attributes.iter().any(|attr| 
                attr.key == "token_id" && attr.value == "PASS1"
            )
        ));
    
        // Mint second pass
        let res = app.execute_contract(
            user2.clone(),
            contract_addr.clone(),
            &msg,
            &[Coin::new(10u128, "uxion")],
        ).unwrap();
    
        // Check if second token ID is "PASS2"
        assert!(res.events.iter().any(|e| 
            e.attributes.iter().any(|attr| 
                attr.key == "token_id" && attr.value == "PASS2"
            )
        ));
    
        // Verify both passes exist and are valid
        let query_msg = QueryMsg::Extension { 
            msg: PassQuery::CheckValidity { 
                token_id: "PASS1".to_string() 
            } 
        };
        let res: ValidityResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &query_msg)
            .unwrap();
        assert!(res.is_valid);
    
        let query_msg = QueryMsg::Extension { 
            msg: PassQuery::CheckValidity { 
                token_id: "PASS2".to_string() 
            } 
        };
        let res: ValidityResponse = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &query_msg)
            .unwrap();
        assert!(res.is_valid);
    }

    fn contract_pass() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new_with_empty(execute, instantiate, query);
        Box::new(contract)
    }
}