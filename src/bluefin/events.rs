use serde::{Deserialize, Serialize};
use sui_types::base_types::ObjectID;

// Define the events that can be emitted by the bluefin contract
// https://github.com/fireflyprotocol/bluefin-spot-contract-interface/blob/main/sources/events.move

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct PositionOpened {
    pub pool_id: ObjectID,
    pub position_id: ObjectID,
    pub tick_lower: i32,
    pub tick_upper: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct PositionClosed {
    pub pool_id: ObjectID,
    pub position_id: ObjectID,
    pub tick_lower: i32,
    pub tick_upper: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LiquidityProvided {
    pub pool_id: ObjectID,
    pub position_id: ObjectID,
    pub coin_a_amount: u64,
    pub coin_b_amount: u64,
    pub pool_coin_a_amount: u64,
    pub pool_coin_b_amount: u64,
    pub liquidity: u128,
    pub before_liqiudity: u128,
    pub after_liqiudity: u128,
    pub current_sqrt_price: u128,
    pub current_tick_index: i32,
    pub low_tick: i32,
    pub upper_tick: i32,
    pub sequence_number: u128,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct LiquidityRemoved {
    pub pool_id: ObjectID,
    pub position_id: ObjectID,
    pub coin_a_amount: u64,
    pub coin_b_amount: u64,
    pub pool_coin_a_amount: u64,
    pub pool_coin_b_amount: u64,
    pub liquidity: u128,
    pub before_liqiudity: u128,
    pub after_liqiudity: u128,
    pub current_sqrt_price: u128,
    pub current_tick_index: i32,
    pub low_tick: i32,
    pub upper_tick: i32,
    pub sequence_number: u128,
}
