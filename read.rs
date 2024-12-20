// // state.rs starts from here

// use schemars::JsonSchema;
// use serde::de::DeserializeOwned;
// use serde::{Deserialize, Serialize};
// use std::marker::PhantomData;

// use cosmwasm_std::{Addr, BlockInfo, StdResult, Storage};

// use cw721_soulbound::{ContractInfoResponse, CustomMsg, Cw721, Expiration};
// use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

// pub struct Cw721Contract<'a, T, C, E, Q>
// where
//     T: Serialize + DeserializeOwned + Clone,
//     Q: CustomMsg,
//     E: CustomMsg,
// {
//     pub contract_info: Item<'a, ContractInfoResponse>,
//     pub minter: Item<'a, Addr>,
//     pub token_count: Item<'a, u64>,
//     /// Stored as (granter, operator) giving operator full control over granter's account
//     pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
//     pub tokens: IndexedMap<'a, &'a str, TokenInfo<T>, TokenIndexes<'a, T>>,

//     pub(crate) _custom_response: PhantomData<C>,
//     pub(crate) _custom_query: PhantomData<Q>,
//     pub(crate) _custom_execute: PhantomData<E>,
// }

// // This is a signal, the implementations are in other files
// impl<'a, T, C, E, Q> Cw721<T, C> for Cw721Contract<'a, T, C, E, Q>
// where
//     T: Serialize + DeserializeOwned + Clone,
//     C: CustomMsg,
//     E: CustomMsg,
//     Q: CustomMsg,
// {
// }

// impl<T, C, E, Q> Default for Cw721Contract<'static, T, C, E, Q>
// where
//     T: Serialize + DeserializeOwned + Clone,
//     E: CustomMsg,
//     Q: CustomMsg,
// {
//     fn default() -> Self {
//         Self::new(
//             "nft_info",
//             "minter",
//             "num_tokens",
//             "operators",
//             "tokens",
//             "tokens__owner",
//         )
//     }
// }

// impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
// where
//     T: Serialize + DeserializeOwned + Clone,
//     E: CustomMsg,
//     Q: CustomMsg,
// {
//     fn new(
//         contract_key: &'a str,
//         minter_key: &'a str,
//         token_count_key: &'a str,
//         operator_key: &'a str,
//         tokens_key: &'a str,
//         tokens_owner_key: &'a str,
//     ) -> Self {
//         let indexes = TokenIndexes {
//             owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
//         };
//         Self {
//             contract_info: Item::new(contract_key),
//             minter: Item::new(minter_key),
//             token_count: Item::new(token_count_key),
//             operators: Map::new(operator_key),
//             tokens: IndexedMap::new(tokens_key, indexes),
//             _custom_response: PhantomData,
//             _custom_execute: PhantomData,
//             _custom_query: PhantomData,
//         }
//     }

//     pub fn token_count(&self, storage: &dyn Storage) -> StdResult<u64> {
//         Ok(self.token_count.may_load(storage)?.unwrap_or_default())
//     }

//     pub fn increment_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
//         let val = self.token_count(storage)? + 1;
//         self.token_count.save(storage, &val)?;
//         Ok(val)
//     }

//     pub fn decrement_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
//         let val = self.token_count(storage)? - 1;
//         self.token_count.save(storage, &val)?;
//         Ok(val)
//     }
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct TokenInfo<T> {
//     /// The owner of the newly minted NFT
//     pub owner: Addr,
//     /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
//     pub approvals: Vec<Approval>,

//     /// Universal resource identifier for this NFT
//     /// Should point to a JSON file that conforms to the ERC721
//     /// Metadata JSON Schema
//     pub token_uri: Option<String>,

//     /// You can add any custom metadata here when you extend cw721-base
//     pub extension: T,
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// pub struct Approval {
//     /// Account that can transfer/send the token
//     pub spender: Addr,
//     /// When the Approval expires (maybe Expiration::never)
//     pub expires: Expiration,
// }

// impl Approval {
//     pub fn is_expired(&self, block: &BlockInfo) -> bool {
//         self.expires.is_expired(block)
//     }
// }

// pub struct TokenIndexes<'a, T>
// where
//     T: Serialize + DeserializeOwned + Clone,
// {
//     pub owner: MultiIndex<'a, Addr, TokenInfo<T>, String>,
// }

// impl<'a, T> IndexList<TokenInfo<T>> for TokenIndexes<'a, T>
// where
//     T: Serialize + DeserializeOwned + Clone,
// {
//     fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo<T>>> + '_> {
//         let v: Vec<&dyn Index<TokenInfo<T>>> = vec![&self.owner];
//         Box::new(v.into_iter())
//     }
// }

// pub fn token_owner_idx<T>(d: &TokenInfo<T>) -> Addr {
//     d.owner.clone()
// }


// // msg.rs  starts from here

// use schemars::JsonSchema;
// use serde::{Deserialize, Serialize};

// use cw721_soulbound::Expiration;

// #[allow(clippy::derive_partial_eq_without_eq)]
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct InstantiateMsg {
//     /// Name of the NFT contract
//     pub name: String,
//     /// Symbol of the NFT contract
//     pub symbol: String,

//     /// The minter is the only one who can create new NFTs.
//     /// This is designed for a base NFT that is controlled by an external program
//     /// or contract. You will likely replace this with custom logic in custom NFTs
//     pub minter: String,
// }

// /// This is like Cw721ExecuteMsg but we add a Mint command for an owner
// /// to make this stand-alone. You will likely want to remove mint and
// /// use other control logic in any contract that inherits this.
// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// #[serde(rename_all = "snake_case")]
// pub enum ExecuteMsg<T, E> {
//     /// Allows operator to transfer / send the token from the owner's account.
//     /// If expiration is set, then this allowance has a time/height limit
//     Approve {
//         spender: String,
//         token_id: String,
//         expires: Option<Expiration>,
//     },
    
//     /// Remove previously granted Approval
//     Revoke { 
//         spender: String, 
//         token_id: String 
//     },
    
//     /// Allows operator to transfer / send any token from the owner's account.
//     /// If expiration is set, then this allowance has a time/height limit
//     ApproveAll {
//         operator: String,
//         expires: Option<Expiration>,
//     },
//     /// Remove previously granted ApproveAll permission
//     RevokeAll { operator: String },

//     /// Mint a new NFT, can only be called by the contract minter
//     Mint(MintMsg<T>),

//     /// Burn an NFT the sender has access to
//     Burn { token_id: String },

//     /// Extension msg
//     Extension { msg: E },
// }

// #[allow(clippy::derive_partial_eq_without_eq)]
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct MintMsg<T> {
//     /// Unique ID of the NFT
//     pub token_id: String,
//     /// The owner of the newly minter NFT
//     pub owner: String,
//     /// Universal resource identifier for this NFT
//     /// Should point to a JSON file that conforms to the ERC721
//     /// Metadata JSON Schema
//     pub token_uri: Option<String>,
//     /// Any custom extension used by this contract
//     pub extension: T,
// }

// #[allow(clippy::derive_partial_eq_without_eq)]
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum QueryMsg<Q> {
//     /// Return the owner of the given token, error if token does not exist
//     /// Return type: OwnerOfResponse
//     OwnerOf {
//         token_id: String,
//         /// unset or false will filter out expired approvals, you must set to true to see them
//         include_expired: Option<bool>,
//     },
//     /// Return operator that can access all of the owner's tokens.
//     /// Return type: `ApprovalResponse`
//     Approval {
//         token_id: String,
//         spender: String,
//         include_expired: Option<bool>,
//     },
//     /// Return approvals that a token has
//     /// Return type: `ApprovalsResponse`
//     Approvals {
//         token_id: String,
//         include_expired: Option<bool>,
//     },
//     /// List all operators that can access all of the owner's tokens
//     /// Return type: `OperatorsResponse`
//     AllOperators {
//         owner: String,
//         /// unset or false will filter out expired items, you must set to true to see them
//         include_expired: Option<bool>,
//         start_after: Option<String>,
//         limit: Option<u32>,
//     },
//     /// Total number of tokens issued
//     NumTokens {},

//     /// With MetaData Extension.
//     /// Returns top-level metadata about the contract: `ContractInfoResponse`
//     ContractInfo {},
//     /// With MetaData Extension.
//     /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
//     /// but directly from the contract: `NftInfoResponse`
//     NftInfo {
//         token_id: String,
//     },
//     /// With MetaData Extension.
//     /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
//     /// for clients: `AllNftInfo`
//     AllNftInfo {
//         token_id: String,
//         /// unset or false will filter out expired approvals, you must set to true to see them
//         include_expired: Option<bool>,
//     },

//     /// With Enumerable extension.
//     /// Returns all tokens owned by the given address, [] if unset.
//     /// Return type: TokensResponse.
//     Tokens {
//         owner: String,
//         start_after: Option<String>,
//         limit: Option<u32>,
//     },
//     /// With Enumerable extension.
//     /// Requires pagination. Lists all token_ids controlled by the contract.
//     /// Return type: TokensResponse.
//     AllTokens {
//         start_after: Option<String>,
//         limit: Option<u32>,
//     },

//     // Return the minter
//     Minter {},

//     /// Extension query
//     Extension {
//         msg: Q,
//     },
// }

// /// Shows who can mint these tokens
// #[allow(clippy::derive_partial_eq_without_eq)]
// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// pub struct MinterResponse {
//     pub minter: String,
// }


// // execute.rs starts from here 

// use serde::de::DeserializeOwned;
// use serde::Serialize;

// use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult};

// use cw2::set_contract_version;
// use cw721_soulbound::{ContractInfoResponse, CustomMsg, Cw721Execute, Expiration};

// use crate::error::ContractError;
// use crate::msg::{ExecuteMsg, InstantiateMsg, MintMsg};
// use crate::state::{Approval, Cw721Contract, TokenInfo};

// // Version info for migration
// const CONTRACT_NAME: &str = "crates.io:cw721-soulbound";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
// where
//     T: Serialize + DeserializeOwned + Clone,
//     C: CustomMsg,
//     E: CustomMsg,
//     Q: CustomMsg,
// {
//     pub fn instantiate(
//         &self,
//         deps: DepsMut,
//         _env: Env,
//         _info: MessageInfo,
//         msg: InstantiateMsg,
//     ) -> StdResult<Response<C>> {
//         set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

//         let info = ContractInfoResponse {
//             name: msg.name,
//             symbol: msg.symbol,
//         };
//         self.contract_info.save(deps.storage, &info)?;
//         let minter = deps.api.addr_validate(&msg.minter)?;
//         self.minter.save(deps.storage, &minter)?;
//         Ok(Response::default())
//     }

//     pub fn execute(
//         &self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         msg: ExecuteMsg<T, E>,
//     ) -> Result<Response<C>, ContractError> {
//         match msg {
//             ExecuteMsg::Mint(msg) => self.mint(deps, env, info, msg),
//             ExecuteMsg::Approve {
//                 spender,
//                 token_id,
//                 expires,
//             } => self.approve(deps, env, info, spender, token_id, expires),
//             ExecuteMsg::Revoke { spender, token_id } => {
//                 self.revoke(deps, env, info, spender, token_id)
//             }
//             ExecuteMsg::ApproveAll { operator, expires } => {
//                 self.approve_all(deps, env, info, operator, expires)
//             }
//             ExecuteMsg::RevokeAll { operator } => self.revoke_all(deps, env, info, operator),
//             ExecuteMsg::Burn { token_id } => self.burn(deps, env, info, token_id),
//             ExecuteMsg::Extension { msg: _ } => Ok(Response::default()),
//         }
//     }
// }

// // TODO pull this into some sort of trait extension??
// impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
// where
//     T: Serialize + DeserializeOwned + Clone,
//     C: CustomMsg,
//     E: CustomMsg,
//     Q: CustomMsg,
// {
//     pub fn mint(
//         &self,
//         deps: DepsMut,
//         _env: Env,
//         info: MessageInfo,
//         msg: MintMsg<T>,
//     ) -> Result<Response<C>, ContractError> {
//         let minter = self.minter.load(deps.storage)?;

//         if info.sender != minter {
//             return Err(ContractError::Unauthorized {});
//         }

//         // create the token
//         let token = TokenInfo {
//             owner: deps.api.addr_validate(&msg.owner)?,
//             approvals: vec![],
//             token_uri: msg.token_uri,
//             extension: msg.extension,
//         };
//         self.tokens
//             .update(deps.storage, &msg.token_id, |old| match old {
//                 Some(_) => Err(ContractError::Claimed {}),
//                 None => Ok(token),
//             })?;

//         self.increment_tokens(deps.storage)?;

//         Ok(Response::new()
//             .add_attribute("action", "mint")
//             .add_attribute("minter", info.sender)
//             .add_attribute("owner", msg.owner)
//             .add_attribute("token_id", msg.token_id))
//     }
// }

// impl<'a, T, C, E, Q> Cw721Execute<T, C> for Cw721Contract<'a, T, C, E, Q>
// where
//     T: Serialize + DeserializeOwned + Clone,
//     C: CustomMsg,
//     E: CustomMsg,
//     Q: CustomMsg,
// {
//     type Err = ContractError;

//     fn approve(
//         &self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         spender: String,
//         token_id: String,
//         expires: Option<Expiration>,
//     ) -> Result<Response<C>, ContractError> {
//         self._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

//         Ok(Response::new()
//             .add_attribute("action", "approve")
//             .add_attribute("sender", info.sender)
//             .add_attribute("spender", spender)
//             .add_attribute("token_id", token_id))
//     }

//     fn revoke(
//         &self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         spender: String,
//         token_id: String,
//     ) -> Result<Response<C>, ContractError> {
//         self._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

//         Ok(Response::new()
//             .add_attribute("action", "revoke")
//             .add_attribute("sender", info.sender)
//             .add_attribute("spender", spender)
//             .add_attribute("token_id", token_id))
//     }

//     fn approve_all(
//         &self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         operator: String,
//         expires: Option<Expiration>,
//     ) -> Result<Response<C>, ContractError> {
//         // reject expired data as invalid
//         let expires = expires.unwrap_or_default();
//         if expires.is_expired(&env.block) {
//             return Err(ContractError::Expired {});
//         }

//         // set the operator for us
//         let operator_addr = deps.api.addr_validate(&operator)?;
//         self.operators
//             .save(deps.storage, (&info.sender, &operator_addr), &expires)?;

//         Ok(Response::new()
//             .add_attribute("action", "approve_all")
//             .add_attribute("sender", info.sender)
//             .add_attribute("operator", operator))
//     }

//     fn revoke_all(
//         &self,
//         deps: DepsMut,
//         _env: Env,
//         info: MessageInfo,
//         operator: String,
//     ) -> Result<Response<C>, ContractError> {
//         let operator_addr = deps.api.addr_validate(&operator)?;
//         self.operators
//             .remove(deps.storage, (&info.sender, &operator_addr));

//         Ok(Response::new()
//             .add_attribute("action", "revoke_all")
//             .add_attribute("sender", info.sender)
//             .add_attribute("operator", operator))
//     }

//     fn burn(
//         &self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         token_id: String,
//     ) -> Result<Response<C>, ContractError> {
//         let token = self.tokens.load(deps.storage, &token_id)?;
//         self.check_can_send(deps.as_ref(), &env, &info, &token)?;

//         self.tokens.remove(deps.storage, &token_id)?;
//         self.decrement_tokens(deps.storage)?;

//         Ok(Response::new()
//             .add_attribute("action", "burn")
//             .add_attribute("sender", info.sender)
//             .add_attribute("token_id", token_id))
//     }
// }

// // helpers
// impl<'a, T, C, E, Q> Cw721Contract<'a, T, C, E, Q>
// where
//     T: Serialize + DeserializeOwned + Clone,
//     C: CustomMsg,
//     E: CustomMsg,
//     Q: CustomMsg,
// {
//     #[allow(clippy::too_many_arguments)]
//     pub fn _update_approvals(
//         &self,
//         deps: DepsMut,
//         env: &Env,
//         info: &MessageInfo,
//         spender: &str,
//         token_id: &str,
//         // if add == false, remove. if add == true, remove then set with this expiration
//         add: bool,
//         expires: Option<Expiration>,
//     ) -> Result<TokenInfo<T>, ContractError> {
//         let mut token = self.tokens.load(deps.storage, token_id)?;
//         // ensure we have permissions
//         self.check_can_approve(deps.as_ref(), env, info, &token)?;

//         // update the approval list (remove any for the same spender before adding)
//         let spender_addr = deps.api.addr_validate(spender)?;
//         token.approvals.retain(|apr| apr.spender != spender_addr);

//         // only difference between approve and revoke
//         if add {
//             // reject expired data as invalid
//             let expires = expires.unwrap_or_default();
//             if expires.is_expired(&env.block) {
//                 return Err(ContractError::Expired {});
//             }
//             let approval = Approval {
//                 spender: spender_addr,
//                 expires,
//             };
//             token.approvals.push(approval);
//         }

//         self.tokens.save(deps.storage, token_id, &token)?;

//         Ok(token)
//     }

//     /// returns true if the sender can execute approve or reject on the contract
//     pub fn check_can_approve(
//         &self,
//         deps: Deps,
//         env: &Env,
//         info: &MessageInfo,
//         token: &TokenInfo<T>,
//     ) -> Result<(), ContractError> {
//         // owner can approve
//         if token.owner == info.sender {
//             return Ok(());
//         }
//         // operator can approve
//         let op = self
//             .operators
//             .may_load(deps.storage, (&token.owner, &info.sender))?;
//         match op {
//             Some(ex) => {
//                 if ex.is_expired(&env.block) {
//                     Err(ContractError::Unauthorized {})
//                 } else {
//                     Ok(())
//                 }
//             }
//             None => Err(ContractError::Unauthorized {}),
//         }
//     }

//     /// returns true if the sender can transfer ownership of the token
//     pub fn check_can_send(
//         &self,
//         deps: Deps,
//         env: &Env,
//         info: &MessageInfo,
//         token: &TokenInfo<T>,
//     ) -> Result<(), ContractError> {
//         // owner can send
//         if token.owner == info.sender {
//             return Ok(());
//         }

//         // any non-expired token approval can send
//         if token
//             .approvals
//             .iter()
//             .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
//         {
//             return Ok(());
//         }

//         // operator can send
//         let op = self
//             .operators
//             .may_load(deps.storage, (&token.owner, &info.sender))?;
//         match op {
//             Some(ex) => {
//                 if ex.is_expired(&env.block) {
//                     Err(ContractError::Unauthorized {})
//                 } else {
//                     Ok(())
//                 }
//             }
//             None => Err(ContractError::Unauthorized {}),
//         }
//     }
// }


// // helpers.rs starts from here 

// use crate::{ExecuteMsg, QueryMsg};
// use cosmwasm_std::{
//     to_json_binary, Addr, CosmosMsg, CustomMsg, QuerierWrapper, StdResult, WasmMsg, WasmQuery,
// };
// use cw721_soulbound::{
//     AllNftInfoResponse, Approval, ApprovalResponse, ApprovalsResponse, ContractInfoResponse,
//     NftInfoResponse, NumTokensResponse, OperatorsResponse, OwnerOfResponse, TokensResponse,
// };
// use serde::__private::PhantomData;
// use serde::de::DeserializeOwned;
// use serde::{Deserialize, Serialize};

// #[allow(clippy::derive_partial_eq_without_eq)]
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// pub struct Cw721Contract<Q: CustomMsg, E: CustomMsg>(
//     pub Addr,
//     pub PhantomData<Q>,
//     pub PhantomData<E>,
// );

// #[allow(dead_code)]
// impl<Q: CustomMsg, E: CustomMsg> Cw721Contract<Q, E> {
//     pub fn addr(&self) -> Addr {
//         self.0.clone()
//     }

//     pub fn call<T: Serialize>(&self, msg: ExecuteMsg<T, E>) -> StdResult<CosmosMsg> {
//         let msg = to_json_binary(&msg)?;
//         Ok(WasmMsg::Execute {
//             contract_addr: self.addr().into(),
//             msg,
//             funds: vec![],
//         }
//         .into())
//     }

//     pub fn query<T: DeserializeOwned>(
//         &self,
//         querier: &QuerierWrapper,
//         req: QueryMsg<Q>,
//     ) -> StdResult<T> {
//         let query = WasmQuery::Smart {
//             contract_addr: self.addr().into(),
//             msg: to_json_binary(&req)?,
//         }
//         .into();
//         querier.query(&query)
//     }

//     /*** queries ***/

//     pub fn owner_of<T: Into<String>>(
//         &self,
//         querier: &QuerierWrapper,
//         token_id: T,
//         include_expired: bool,
//     ) -> StdResult<OwnerOfResponse> {
//         let req = QueryMsg::OwnerOf {
//             token_id: token_id.into(),
//             include_expired: Some(include_expired),
//         };
//         self.query(querier, req)
//     }

//     pub fn approval<T: Into<String>>(
//         &self,
//         querier: &QuerierWrapper,
//         token_id: T,
//         spender: T,
//         include_expired: Option<bool>,
//     ) -> StdResult<ApprovalResponse> {
//         let req = QueryMsg::Approval {
//             token_id: token_id.into(),
//             spender: spender.into(),
//             include_expired,
//         };
//         let res: ApprovalResponse = self.query(querier, req)?;
//         Ok(res)
//     }

//     pub fn approvals<T: Into<String>>(
//         &self,
//         querier: &QuerierWrapper,
//         token_id: T,
//         include_expired: Option<bool>,
//     ) -> StdResult<ApprovalsResponse> {
//         let req = QueryMsg::Approvals {
//             token_id: token_id.into(),
//             include_expired,
//         };
//         let res: ApprovalsResponse = self.query(querier, req)?;
//         Ok(res)
//     }

//     pub fn all_operators<T: Into<String>>(
//         &self,
//         querier: &QuerierWrapper,
//         owner: T,
//         include_expired: bool,
//         start_after: Option<String>,
//         limit: Option<u32>,
//     ) -> StdResult<Vec<Approval>> {
//         let req = QueryMsg::AllOperators {
//             owner: owner.into(),
//             include_expired: Some(include_expired),
//             start_after,
//             limit,
//         };
//         let res: OperatorsResponse = self.query(querier, req)?;
//         Ok(res.operators)
//     }

//     pub fn num_tokens(&self, querier: &QuerierWrapper) -> StdResult<u64> {
//         let req = QueryMsg::NumTokens {};
//         let res: NumTokensResponse = self.query(querier, req)?;
//         Ok(res.count)
//     }

//     /// With metadata extension
//     pub fn contract_info(&self, querier: &QuerierWrapper) -> StdResult<ContractInfoResponse> {
//         let req = QueryMsg::ContractInfo {};
//         self.query(querier, req)
//     }

//     /// With metadata extension
//     pub fn nft_info<T: Into<String>, U: DeserializeOwned>(
//         &self,
//         querier: &QuerierWrapper,
//         token_id: T,
//     ) -> StdResult<NftInfoResponse<U>> {
//         let req = QueryMsg::NftInfo {
//             token_id: token_id.into(),
//         };
//         self.query(querier, req)
//     }

//     /// With metadata extension
//     pub fn all_nft_info<T: Into<String>, U: DeserializeOwned>(
//         &self,
//         querier: &QuerierWrapper,
//         token_id: T,
//         include_expired: bool,
//     ) -> StdResult<AllNftInfoResponse<U>> {
//         let req = QueryMsg::AllNftInfo {
//             token_id: token_id.into(),
//             include_expired: Some(include_expired),
//         };
//         self.query(querier, req)
//     }

//     /// With enumerable extension
//     pub fn tokens<T: Into<String>>(
//         &self,
//         querier: &QuerierWrapper,
//         owner: T,
//         start_after: Option<String>,
//         limit: Option<u32>,
//     ) -> StdResult<TokensResponse> {
//         let req = QueryMsg::Tokens {
//             owner: owner.into(),
//             start_after,
//             limit,
//         };
//         self.query(querier, req)
//     }

//     /// With enumerable extension
//     pub fn all_tokens(
//         &self,
//         querier: &QuerierWrapper,
//         start_after: Option<String>,
//         limit: Option<u32>,
//     ) -> StdResult<TokensResponse> {
//         let req = QueryMsg::AllTokens { start_after, limit };
//         self.query(querier, req)
//     }

//     /// returns true if the contract supports the metadata extension
//     pub fn has_metadata(&self, querier: &QuerierWrapper) -> bool {
//         self.contract_info(querier).is_ok()
//     }

//     /// returns true if the contract supports the enumerable extension
//     pub fn has_enumerable(&self, querier: &QuerierWrapper) -> bool {
//         self.tokens(querier, self.addr(), None, Some(1)).is_ok()
//     }
// }
// // lib.rs starts from here 
// mod contract_tests;
// mod error;
// mod execute;
// pub mod helpers;
// pub mod msg;
// mod query;
// pub mod state;

// pub use crate::error::ContractError;
// pub use crate::msg::{ExecuteMsg, InstantiateMsg, MintMsg, MinterResponse, QueryMsg};
// pub use crate::state::Cw721Contract;
// use cosmwasm_std::Empty;


// //custom

// pub use cw721_soulbound::CustomMsg;
// pub use cw721_soulbound::Expiration;

// // This is a simple type to let us handle empty extensions
// pub type Extension = Option<Empty>;

// pub mod entry {
//     use super::*;

//     #[cfg(not(feature = "library"))]
//     use cosmwasm_std::entry_point;
//     use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

//     // This makes a conscious choice on the various generics used by the contract
//     #[cfg_attr(not(feature = "library"), entry_point)]
//     pub fn instantiate(
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         msg: InstantiateMsg,
//     ) -> StdResult<Response> {
//         let tract = Cw721Contract::<Extension, Empty, Empty, Empty>::default();
//         tract.instantiate(deps, env, info, msg)
//     }

//     #[cfg_attr(not(feature = "library"), entry_point)]
//     pub fn execute(
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         msg: ExecuteMsg<Extension, Empty>,
//     ) -> Result<Response, ContractError> {
//         let tract = Cw721Contract::<Extension, Empty, Empty, Empty>::default();
//         tract.execute(deps, env, info, msg)
//     }

//     #[cfg_attr(not(feature = "library"), entry_point)]
//     pub fn query(deps: Deps, env: Env, msg: QueryMsg<Empty>) -> StdResult<Binary> {
//         let tract = Cw721Contract::<Extension, Empty, Empty, Empty>::default();
//         tract.query(deps, env, msg)
//     }
// }














// // my code starts here 


// // state.rs starts from here: 

// use cosmwasm_schema::cw_serde;
// use cosmwasm_std::{Addr, Timestamp};
// use cw_storage_plus::{Item, Map};

// // Import Custom Message (if applicable)
// use crate::msg::PassMsg;


// // Define the main configuration item
// pub const CONFIG: Item<Config> = Item::new("config");

// // Define metadata for tokens
// #[cw_serde]
// pub struct PassExtension {
//     pub expires_at: Timestamp,
//     pub is_active: bool,
//     pub grace_period_end: Timestamp,
//     pub times_renewed: u32,
// }

// pub type Contract<'a> = cw721_base_soulbound::Cw721Contract<'a, PassExtension, PassMsg, PassMsg, PassMsg>;


// /// Contract configuration
// #[cw_serde]
// pub struct Config {
//     pub pass_price: u128, //Cost to mint/renew a pass in uxion
//     pub pass_duration: u64, //Duration of the pass in seconds
//     pub grace_period: u64, //Grace period after expiry
//     pub payment_address: Addr, //Address receiving payments
// }

// /// Additional helpers for managing PassExtension logic
// impl PassExtension {
//     /// Create a new pass with proper timestamps
//     pub fn new(current_time: Timestamp, pass_duration: u64, grace_period: u64) -> Self {
//         let expires_at = current_time.plus_seconds(pass_duration);
//         let grace_period_end = expires_at.plus_seconds(grace_period);

//         Self {
//             expires_at,
//             is_active: true,
//             grace_period_end,
//             times_renewed: 0,
//         }
//     }

//     /// Check pass status
//     pub fn status(&self, current_time: Timestamp) -> PassStatus {
//         if current_time < self.expires_at {
//             PassStatus::Active
//         } else if current_time <= self.grace_period_end {
//             PassStatus::InGracePeriod
//         } else {
//             PassStatus::Expired
//         }
//     }

//     /// Handle renewal of a pass
//     pub fn renew(&mut self, current_time: Timestamp, pass_duration: u64, grace_period: u64) {
//         self.expires_at = current_time.plus_seconds(pass_duration);
//         self.grace_period_end = self.expires_at.plus_seconds(grace_period);
//         self.is_active = true;
//         self.times_renewed += 1;
//     }
// }

// /// Enum to represent pass status
// #[cw_serde]
// pub enum PassStatus {
//     Active,
//     InGracePeriod,
//     Expired,
// }


// pub const TOKENS_BY_OWNER: Map<&Addr, Vec<String>> = Map::new("tokens_by_owner");


// // msg.rs 

// use cosmwasm_schema::cw_serde;
// use cosmwasm_std::Timestamp;
// use crate::state::PassExtension;
// use cw721_base_soulbound::CustomMsg;

// // Custom Instantiate message for your contract
// #[cw_serde]
// pub struct InstantiateMsg {
//     pub name: String,
//     pub symbol: String,
//     pub minter: String,
//     pub pass_price: u128,
//     pub pass_duration: u64,
//     pub grace_period: u64,
//     pub payment_address: String,
// }

// // Custom Pass messages extending the base contract
// #[cw_serde]
// pub enum PassMsg {
//     MintPass { token_id: String },
//     RenewPass { token_id: String },
//     BurnExpiredPass { token_id: String },
// }

// impl CustomMsg for PassMsg {}

// pub type ExecuteMsg = cw721_base_soulbound::ExecuteMsg<PassExtension, PassMsg>;

// // Custom Pass Queries
// #[cw_serde]
// pub enum PassQuery {
//     CheckValidity { token_id: String },
//     GetConfig {},
// }

// pub type QueryMsg = cw721_base_soulbound::QueryMsg<PassQuery>;

// // Custom query responses
// #[cw_serde]
// pub struct ValidityResponse {
//     pub token_id: String,
//     pub is_valid: bool,
//     pub expires_at: Timestamp,
//     pub in_grace_period: bool,
//     pub grace_period_end: Option<Timestamp>,
// }

// #[cw_serde]
// pub struct ConfigResponse {
//     pub pass_price: u128,
//     pub pass_duration: u64,
//     pub grace_period: u64,
//     pub payment_address: String,
// }


// // execute.rs 

// use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
// use cw721_base_soulbound::ContractError;
// use crate::state::{Contract, PassExtension, CONFIG};
// use crate::msg::{ExecuteMsg, PassMsg};

// // Execute function handles both the custom extension messages and standard contract functions
// pub fn execute(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     msg: ExecuteMsg,
// ) -> Result<Response, ContractError> {
//     // Handle extension messages first
//     if let ExecuteMsg::Extension { msg } = msg {
//         match msg {
//             PassMsg::MintPass { token_id } => mint_pass(deps, env, info, token_id),
//             PassMsg::RenewPass { token_id } => renew_pass(deps, env, info, token_id),
//             PassMsg::BurnExpiredPass { token_id } => burn_expired_pass(deps, env, info, token_id),
//         }
//     } else {
//         let contract = Contract::default().execute(deps, env, info, msg);
//         // Delegate other standard operations to the base contract
        
//     }
// }

// // Mint pass function
// fn mint_pass(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     token_id: String,
// ) -> Result<Response, ContractError> {
//     let config = CONFIG.load(deps.storage)?;

//     // Validate the payment
//     validate_payment(&info, config.pass_price)?;

//     // Create the extension with time-based properties
//     let extension = PassExtension::new(
//         env.block.time,
//         config.pass_duration,
//         config.grace_period,
//     );

//     // Use base contract's mint functionality
//     Contract::default().tokens.update(deps.storage, &token_id, |old| match old {
//         Some(_) => Err(ContractError::Custom("Token ID already exists".to_string())),
//         None => Ok(cw721_base_soulbound::state::TokenInfo {
//             owner: info.sender.clone(),
//             approvals: vec![],
//             token_uri: None,
//             extension,
//         }),
//     })?;

//     Ok(Response::new()
//         .add_attribute("action", "mint_pass")
//         .add_attribute("token_id", token_id))
// }

// // Renew pass function
// fn renew_pass(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     token_id: String,
// ) -> Result<Response, ContractError> {
//     let config = CONFIG.load(deps.storage)?;
//     validate_payment(&info, config.pass_price)?;

//     let contract = Contract::default();
//     let mut token = contract.tokens.load(deps.storage, &token_id)?;

//     // Ensure the sender is the token owner
//     if token.owner != info.sender {
//         return Err(ContractError::Unauthorized {});
//     }

//     // Renew the pass with updated extension values
//     token.extension.renew(env.block.time, config.pass_duration, config.grace_period);

//     contract.tokens.save(deps.storage, &token_id, &token)?;

//     Ok(Response::new()
//         .add_attribute("action", "renew_pass")
//         .add_attribute("token_id", token_id))
// }

// // Burn expired pass function
// fn burn_expired_pass(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     token_id: String,
// ) -> Result<Response, ContractError> {
//     let contract = Contract::default();
//     let token = contract.tokens.load(deps.storage, &token_id)?;

//     // Check if the pass is expired
//     if token.extension.status(env.block.time) != PassStatus::Expired {
//         return Err(ContractError::Custom("Pass must be expired to burn".to_string()));
//     }

//     // Remove the expired token from storage
//     contract.tokens.remove(deps.storage, &token_id)?;

//     Ok(Response::new()
//         .add_attribute("action", "burn_expired_pass")
//         .add_attribute("token_id", token_id))
// }

// // Payment validation helper
// fn validate_payment(info: &MessageInfo, required_price: u128) -> Result<(), ContractError> {
//     let payment = info
//         .funds
//         .iter()
//         .find(|c| c.denom == "uxion")
//         .ok_or_else(|| ContractError::Custom("No payment provided".to_string()))?;

//     if payment.amount.u128() < required_price {
//         return Err(ContractError::Custom("Insufficient payment".to_string()));
//     }

//     Ok(())
// }
