use crate::domain::OrderbookLevel;
use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

// OpenBook V2 Constants
pub const OPENBOOK_V2_PROGRAM_ID: &str = "opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb";

// Account discriminators (Anchor style)
pub const MARKET_DISCRIMINATOR: [u8; 8] = [213, 222, 12, 126, 25, 23, 204, 237];
pub const BOOK_SIDE_DISCRIMINATOR: [u8; 8] = [178, 119, 219, 142, 234, 1, 163, 133];

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MarketStateV2 {
    pub bump: u8,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub market_authority: Pubkey,
    pub base_lot_size: i64,
    pub quote_lot_size: i64,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub event_heap: Pubkey,
}

impl MarketStateV2 {
    pub fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < 8 + 1 + 1 + 1 + 5 + 32 + 8 + 8 + 32 + 32 + 32 {
            return Err(anyhow!("V2 Market account data too short"));
        }

        // Check discriminator
        if data[0..8] != MARKET_DISCRIMINATOR {
            return Err(anyhow!("Invalid V2 Market discriminator"));
        }

        let bump = data[8];
        let base_decimals = data[9];
        let quote_decimals = data[10];
        // 5 bytes padding: 11..16

        let market_authority = Pubkey::new_from_array(data[16..48].try_into()?);
        let base_lot_size = i64::from_le_bytes(data[48..56].try_into()?);
        let quote_lot_size = i64::from_le_bytes(data[56..64].try_into()?);

        let bids = Pubkey::new_from_array(data[64..96].try_into()?);
        let asks = Pubkey::new_from_array(data[96..128].try_into()?);
        let event_heap = Pubkey::new_from_array(data[128..160].try_into()?);

        Ok(Self {
            bump,
            base_decimals,
            quote_decimals,
            market_authority,
            base_lot_size,
            quote_lot_size,
            bids,
            asks,
            event_heap,
        })
    }
}

pub fn parse_book_side_v2(
    data: &[u8],
    is_bids: bool,
    base_decimals: u8,
    quote_decimals: u8,
    base_lot_size: i64,
    quote_lot_size: i64,
) -> Result<Vec<OrderbookLevel>> {
    if data.len() < 8 + 8 {
        return Err(anyhow!("BookSide data too short"));
    }

    if data[0..8] != BOOK_SIDE_DISCRIMINATOR {
        return Err(anyhow!("Invalid BookSide discriminator"));
    }

    let nodes_start = 8 + 128;
    let node_size = 88;

    let mut levels = Vec::new();

    for i in 0..((data.len() - nodes_start) / node_size).min(1024) {
        let offset = nodes_start + i * node_size;
        let tag = data[offset];

        if tag == 2 {
            // LeafNode tag in V2
            let key = u128::from_le_bytes(data[offset + 1 + 8..offset + 1 + 24].try_into()?);
            let price_raw = (key >> 64) as u64;
            let quantity = i64::from_le_bytes(data[offset + 1 + 56..offset + 1 + 64].try_into()?);

            let base_pow = Decimal::from(10u64.pow(base_decimals as u32));
            let quote_pow = Decimal::from(10u64.pow(quote_decimals as u32));

            let price = (Decimal::from(price_raw) * Decimal::from(quote_lot_size) * base_pow)
                / (Decimal::from(base_lot_size) * quote_pow);

            let size = Decimal::from(quantity) * Decimal::from(base_lot_size) / base_pow;

            levels.push(OrderbookLevel { price, size });
        }
    }

    if is_bids {
        levels.sort_by(|a, b| b.price.cmp(&a.price));
    } else {
        levels.sort_by(|a, b| a.price.cmp(&b.price));
    }

    Ok(levels)
}

#[allow(dead_code, clippy::too_many_arguments)]
pub fn create_place_order_v2_instruction(
    market: &Pubkey,
    open_orders: &Pubkey,
    asks: &Pubkey,
    bids: &Pubkey,
    event_heap: &Pubkey,
    market_base_vault: &Pubkey,
    market_quote_vault: &Pubkey,
    owner: &Pubkey,
    user_token_account: &Pubkey,
    side: u8,
    price: i64,
    max_base_qty: i64,
    max_quote_qty: i64,
    client_order_id: u64,
) -> solana_sdk::instruction::Instruction {
    let program_id = Pubkey::from_str(OPENBOOK_V2_PROGRAM_ID).unwrap();

    let mut data = Vec::with_capacity(8 + 32);
    data.extend_from_slice(&[142, 60, 48, 126, 114, 252, 19, 137]);
    data.push(side);
    data.extend_from_slice(&price.to_le_bytes());
    data.extend_from_slice(&max_base_qty.to_le_bytes());
    data.extend_from_slice(&max_quote_qty.to_le_bytes());
    data.extend_from_slice(&client_order_id.to_le_bytes());
    data.push(0);

    solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*owner, true),
            solana_sdk::instruction::AccountMeta::new(*open_orders, false),
            solana_sdk::instruction::AccountMeta::new(*market, false),
            solana_sdk::instruction::AccountMeta::new(*bids, false),
            solana_sdk::instruction::AccountMeta::new(*asks, false),
            solana_sdk::instruction::AccountMeta::new(*event_heap, false),
            solana_sdk::instruction::AccountMeta::new(*market_base_vault, false),
            solana_sdk::instruction::AccountMeta::new(*market_quote_vault, false),
            solana_sdk::instruction::AccountMeta::new(*user_token_account, false),
            solana_sdk::instruction::AccountMeta::new_readonly(
                solana_sdk::system_program::id(),
                false,
            ),
            solana_sdk::instruction::AccountMeta::new_readonly(
                solana_sdk::sysvar::rent::id(),
                false,
            ),
        ],
        data,
    }
}

pub const MARKET_STATE_LAYOUT_V3_LEN: usize = 388;
#[derive(Debug, Clone)]
pub struct MarketStateV3 {
    pub bids: [u8; 32],
    pub asks: [u8; 32],
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub event_queue: [u8; 32],
}

impl MarketStateV3 {
    pub fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < MARKET_STATE_LAYOUT_V3_LEN {
            return Err(anyhow!("V3/V1 Market account data too short"));
        }
        let mut bids = [0u8; 32];
        bids.copy_from_slice(&data[285..317]);
        let mut asks = [0u8; 32];
        asks.copy_from_slice(&data[317..349]);
        let base_lot_size = u64::from_le_bytes(data[349..357].try_into()?);
        let quote_lot_size = u64::from_le_bytes(data[357..365].try_into()?);
        let mut event_queue = [0u8; 32];
        event_queue.copy_from_slice(&data[253..285]);
        Ok(Self {
            bids,
            asks,
            base_lot_size,
            quote_lot_size,
            event_queue,
        })
    }
}

pub fn create_jito_tip_instruction(
    owner: &Pubkey,
    tip_lamports: u64,
) -> solana_sdk::instruction::Instruction {
    let tip_accounts = [
        "96g9sR9SGvpH91qSS388Ppx6q6bT42p4t7rJ4vQp3u6C",
        "HFqU5x63VTqvQss8hp1uE17D3Jp2N6rBqA5VvL9Fv95v",
        "Cw8CFyMvGrnC7JvSbxujSAn61S19p9k8X1Yj8D1nK5sN",
    ];
    let tip_pubkey = Pubkey::from_str(tip_accounts[0]).unwrap();
    solana_sdk::system_instruction::transfer(owner, &tip_pubkey, tip_lamports)
}

#[allow(dead_code, clippy::too_many_arguments)]
pub fn create_cancel_order_v2_instruction(
    market: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    open_orders: &Pubkey,
    owner: &Pubkey,
    event_queue: &Pubkey,
    side: u8,
    order_id: u128,
) -> solana_sdk::instruction::Instruction {
    let program_id = Pubkey::from_str("srmqPvSwwJbtLZ9Uv7j8W7YVFe4Gz74Xp2Y7tENz7u4").unwrap();
    let mut data = Vec::with_capacity(25);
    data.extend_from_slice(&11u32.to_le_bytes());
    data.push(side);
    data.extend_from_slice(&order_id.to_le_bytes());
    solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*market, false),
            solana_sdk::instruction::AccountMeta::new(*bids, false),
            solana_sdk::instruction::AccountMeta::new(*asks, false),
            solana_sdk::instruction::AccountMeta::new(*open_orders, false),
            solana_sdk::instruction::AccountMeta::new_readonly(*owner, true),
            solana_sdk::instruction::AccountMeta::new(*event_queue, false),
        ],
        data,
    }
}

#[allow(dead_code, clippy::too_many_arguments)]
pub fn create_cancel_all_orders_instruction(
    market: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    open_orders: &Pubkey,
    owner: &Pubkey,
    event_queue: &Pubkey,
    side: u8,
    limit: u16,
) -> solana_sdk::instruction::Instruction {
    let program_id = Pubkey::from_str("srmqPvSwwJbtLZ9Uv7j8W7YVFe4Gz74Xp2Y7tENz7u4").unwrap();

    let mut data = Vec::with_capacity(7);
    data.extend_from_slice(&7u32.to_le_bytes());
    data.push(side);
    data.extend_from_slice(&limit.to_le_bytes());

    solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*market, false),
            solana_sdk::instruction::AccountMeta::new(*bids, false),
            solana_sdk::instruction::AccountMeta::new(*asks, false),
            solana_sdk::instruction::AccountMeta::new(*open_orders, false),
            solana_sdk::instruction::AccountMeta::new_readonly(*owner, true),
            solana_sdk::instruction::AccountMeta::new(*event_queue, false),
        ],
        data,
    }
}
