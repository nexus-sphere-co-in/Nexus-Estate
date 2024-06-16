use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Addr,
};
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

// Define the state structure
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RealEstate {
    pub avg_block_time: u8,
    pub decimals: u8,
    pub tax: u8,
    pub rental_limit_months: u8,
    pub rental_limit_blocks: u64,
    pub total_supply: u64,
    pub total_supply2: u64,
    pub rent_per_30_day: Uint128,
    pub accumulated: Uint128,
    pub blocks_per_30_day: u64,
    pub rental_begin: u64,
    pub occupied_until: u64,
    pub tax_deduct: Uint128,
    pub name: String,
    pub symbol: String,
    pub gov: Addr,
    pub main_property_owner: Addr,
    pub tenant: Addr,
    pub stakeholders: Vec<Addr>,
}

pub const REAL_ESTATE: Item<RealEstate> = Item::new("real_estate");
pub const REVENUES: Map<&Addr, Uint128> = Map::new("revenues");
pub const SHARES: Map<&Addr, u64> = Map::new("shares");
pub const ALLOWED: Map<(&Addr, &Addr), u64> = Map::new("allowed");
pub const RENT_PAID_UNTIL: Map<&Addr, u64> = Map::new("rent_paid_until");
pub const SHARES_OFFERED: Map<&Addr, u64> = Map::new("shares_offered");
pub const SHARE_SELL_PRICE: Map<&Addr, Uint128> = Map::new("share_sell_price");

// Define instantiate message
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InstantiateMsg {
    pub property_id: String,
    pub property_symbol: String,
    pub main_property_owner: String,
    pub tax: u8,
    pub avg_block_time: u8,
}

// Define execute messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ExecuteMsg {
    AddStakeholder { stakeholder: String },
    BanStakeholder { stakeholder: String },
    SetTax { tax: u8 },
    SetAvgBlockTime { seconds_per_block: u8 },
    Distribute {},
    SeizureFrom { from: String, to: String, value: u64 },
    SetRentPer30Day { rent: Uint128 },
    PayRent {},
    OfferSharesForSale { amount_shares: u64, price_per_share: Uint128 },
    BuyShares { seller: String, amount_shares: u64 },
    WithdrawRevenue {},
}

// Define query messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum QueryMsg {
    ShowSharesOf { owner: String },
    IsStakeholder { address: String },
    CurrentTenantCheck { tenant_check: String },
}

// Define instantiate function
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let main_property_owner = deps.api.addr_validate(&msg.main_property_owner)?;
    let gov = info.sender.clone();
    let total_supply = 100;
    let total_supply2 = total_supply * total_supply;
    let blocks_per_30_day = 60 * 60 * 24 * 30 / msg.avg_block_time as u64;

    let real_estate = RealEstate {
        avg_block_time: msg.avg_block_time,
        decimals: 0,
        tax: msg.tax,
        rental_limit_months: 12,
        rental_limit_blocks: blocks_per_30_day * 12,
        total_supply,
        total_supply2,
        rent_per_30_day: Uint128::zero(),
        accumulated: Uint128::zero(),
        blocks_per_30_day,
        rental_begin: 0,
        occupied_until: 0,
        tax_deduct: Uint128::zero(),
        name: msg.property_id,
        symbol: msg.property_symbol,
        gov: gov.clone(),
        main_property_owner: main_property_owner.clone(),
        tenant: Addr::unchecked(""),
        stakeholders: vec![gov.clone(), main_property_owner.clone()],
    };

    REAL_ESTATE.save(deps.storage, &real_estate)?;
    SHARES.save(deps.storage, &main_property_owner, &total_supply)?;
    ALLOWED.save(deps.storage, (&main_property_owner, &gov), &u64::MAX)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

// Define execute function
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::AddStakeholder { stakeholder } => add_stakeholder(deps, info, stakeholder),
        ExecuteMsg::BanStakeholder { stakeholder } => ban_stakeholder(deps, info, stakeholder),
        ExecuteMsg::SetTax { tax } => set_tax(deps, info, tax),
        ExecuteMsg::SetAvgBlockTime { seconds_per_block } => set_avg_block_time(deps, info, seconds_per_block),
        ExecuteMsg::Distribute {} => distribute(deps, env, info),
        ExecuteMsg::SeizureFrom { from, to, value } => seizure_from(deps, info, from, to, value),
        ExecuteMsg::SetRentPer30Day { rent } => set_rent_per_30_day(deps, info, rent),
        ExecuteMsg::PayRent {} => pay_rent(deps, env, info),
        ExecuteMsg::OfferSharesForSale { amount_shares, price_per_share } => offer_shares_for_sale(deps, info, amount_shares, price_per_share),
        ExecuteMsg::BuyShares { seller, amount_shares } => buy_shares(deps, info, seller, amount_shares),
        ExecuteMsg::WithdrawRevenue {} => withdraw_revenue(deps, info),
    }
}

// Define query function
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ShowSharesOf { owner } => to_binary(&show_shares_of(deps, owner)?),
        QueryMsg::IsStakeholder { address } => to_binary(&is_stakeholder(deps, address)?),
        QueryMsg::CurrentTenantCheck { tenant_check } => to_binary(&current_tenant_check(deps, tenant_check)?),
    }
}

// Define individual handler functions
fn add_stakeholder(deps: DepsMut, info: MessageInfo, stakeholder: String) -> StdResult<Response> {
    let real_estate = REAL_ESTATE.load(deps.storage)?;
    if info.sender != real_estate.gov {
        return Err(cosmwasm_std::StdError::generic_err("Only gov can add stakeholders"));
    }
    let stakeholder_addr = deps.api.addr_validate(&stakeholder)?;
    if !real_estate.stakeholders.contains(&stakeholder_addr) {
        let mut new_real_estate = real_estate.clone();
        new_real_estate.stakeholders.push(stakeholder_addr.clone());
        REAL_ESTATE.save(deps.storage, &new_real_estate)?;
        ALLOWED.save(deps.storage, (&stakeholder_addr, &real_estate.gov), &u64::MAX)?;
        Ok(Response::new().add_attribute("method", "add_stakeholder"))
    } else {
        Err(cosmwasm_std::StdError::generic_err("Stakeholder already exists"))
    }
}

fn ban_stakeholder(deps: DepsMut, info: MessageInfo, stakeholder: String) -> StdResult<Response> {
    let real_estate = REAL_ESTATE.load(deps.storage)?;
    if info.sender != real_estate.gov {
        return Err(cosmwasm_std::StdError::generic_err("Only gov can ban stakeholders"));
    }
    let stakeholder_addr = deps.api.addr_validate(&stakeholder)?;
    if let Some(index) = real_estate.stakeholders.iter().position(|x| *x == stakeholder_addr) {
        let mut new_real_estate = real_estate.clone();
        new_real_estate.stakeholders.remove(index);
        REAL_ESTATE.save(deps.storage, &new_real_estate)?;
        let shares = SHARES.may_load(deps.storage, &stakeholder_addr)?.unwrap_or(0);
        seizure_from(deps, info.clone(), stakeholder.clone(), info.sender.to_string(), shares)?;
        Ok(Response::new().add_attribute("method", "ban_stakeholder"))
    } else {
        Err(cosmwasm_std::StdError::generic_err("Stakeholder not found"))
    }
}

fn set_tax(deps: DepsMut, info: MessageInfo, tax: u8) -> StdResult<Response> {
    let mut real_estate = REAL_ESTATE.load(deps.storage)?;
    if info.sender != real_estate.gov {
        return Err(cosmwasm_std::StdError::generic_err("Only gov can set tax"));
    }
    real_estate.tax = tax;
    REAL_ESTATE.save(deps.storage, &real_estate)?;
    Ok(Response::new()
        .add_attribute("method", "set_tax")
        .add_attribute("new_tax", tax.to_string()))
}


    // Implement the contract's logic
    pub fn execute<S: Storage, A: Api>(
        deps: &mut Extern<S, A>,
        env: Env,
        info: MessageInfo,
        msg: RealEstateMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            RealEstateMsg::SetAvgBlockTime { seconds_per_block } => {
                only_gov(&info)?;
                assert!(seconds_per_block > 0, "Please enter a value above 0");
                let mut state = RealEstate::load(&deps.storage)?;
                state.avg_block_time = seconds_per_block;
                state.blocks_per_30_day = 60 * 60 * 24 * 30 / seconds_per_block as u64;
                RealEstate::save(&mut deps.storage, &state)?;
                Ok(Response::new()
                   .add_event(RealEstateResponse::AvgBlockTimeChangedTo {
                        avg_block_time: seconds_per_block,
                    })
                   .into())
            }
            RealEstateMsg::Distribute {} => {
                only_gov(&info)?;
                let mut state = RealEstate::load(&deps.storage)?;
                let accumulated = state.accumulated;
                for stakeholder in &state.stakeholders {
                    let shares = state.shares.get(stakeholder).unwrap_or(&0);
                    let eth_to_receive = (accumulated / state.total_supply) * shares as u128;
                    state.accumulated -= eth_to_receive;
                    let revenue = state.revenues.get(stakeholder).unwrap_or(&0) + eth_to_receive;
                    state.revenues.insert(stakeholder, &revenue);
                    Ok(Response::new()
                       .add_event(RealEstateResponse::RevenuesDistributed {
                            shareholder: stakeholder.clone(),
                            gained: eth_to_receive,
                            total: revenue,
                        })
                       .into())
                }
                RealEstate::save(&mut deps.storage, &state)?;
                Ok(Response::new().into())
            }
            RealEstateMsg::SeizureFrom { from, to, value } => {
                let allowance = state.allowed.get(&(from.clone(), info.sender.clone())).unwrap_or(&0);
                let from_balance = state.shares.get(&from).unwrap_or(&0);
                if from_balance >= value && allowance >= value {
                    state.shares.insert(&to, &(state.shares.get(&to).unwrap_or(&0) + value));
                    state.shares.insert(&from, &(from_balance - value));
                    state.allowed.insert((from.clone(), info.sender.clone()), allowance - value);
                    Ok(Response::new()
                       .add_event(RealEstateResponse::Seizure {
                            seized_from: Some(from),
                            to: Some(to),
                            shares:value,
                        })
                      .into())
                } else {
                    Err(ContractError::InsufficientAllowance {})
                }
            }
            //...
        }
    }
    
    fn only_gov(info: &MessageInfo) -> Result<(), ContractError> {
        if info.sender!= "gov" {
            Err(ContractError::Unauthorized {})
        } else {
            Ok(())
        }
    }