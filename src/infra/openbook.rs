use crate::domain::OrderbookLevel;
use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;

// Serum V3 / OpenBook V1 constants and layout offsets
// Note: This is a simplified version of the full DEX state for L2 scanning.

pub const MARKET_STATE_LAYOUT_V3_LEN: usize = 388;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MarketStateV3 {
    pub bids: [u8; 32],
    pub asks: [u8; 32],
    pub request_queue: [u8; 32],
    pub event_queue: [u8; 32],
    pub base_vault: [u8; 32],
    pub quote_vault: [u8; 32],
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    // Add other fields as needed for discovery
}

impl MarketStateV3 {
    pub fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < MARKET_STATE_LAYOUT_V3_LEN {
            return Err(anyhow!("Market account data too short"));
        }

        // Correct Byte offsets for Serum V3 / OpenBook V1 Market Account
        // Header: [5 bytes padding][8 bytes flags] = 13
        // own_address: 13..45
        // vault_signer_nonce: 45..53
        // base_mint: 53..85
        // quote_mint: 85..117
        // base_vault: 117..149
        // base_deposits_total: 149..157
        // base_fees_accrued: 157..165
        // quote_vault: 165..197
        // quote_deposits_total: 197..205
        // quote_fees_accrued: 205..213
        // quote_dust_threshold: 213..221
        // request_queue: 221..253
        // event_queue: 253..285
        // bids: 285..317
        // asks: 317..349
        // base_lot_size: 349..357
        // quote_lot_size: 357..365

        let mut base_vault = [0u8; 32];
        base_vault.copy_from_slice(&data[117..149]);

        let mut quote_vault = [0u8; 32];
        quote_vault.copy_from_slice(&data[165..197]);

        let mut request_queue = [0u8; 32];
        request_queue.copy_from_slice(&data[221..253]);

        let mut event_queue = [0u8; 32];
        event_queue.copy_from_slice(&data[253..285]);

        let mut bids = [0u8; 32];
        bids.copy_from_slice(&data[285..317]);

        let mut asks = [0u8; 32];
        asks.copy_from_slice(&data[317..349]);

        let base_lot_size = u64::from_le_bytes(data[349..357].try_into()?);
        let quote_lot_size = u64::from_le_bytes(data[357..365].try_into()?);

        Ok(Self {
            bids,
            asks,
            request_queue,
            event_queue,
            base_vault,
            quote_vault,
            base_lot_size,
            quote_lot_size,
        })
    }
}

pub fn parse_slab(
    data: &[u8],
    is_bids: bool,
    base_lots: u64,
    quote_lots: u64,
) -> Result<Vec<OrderbookLevel>> {
    // Serum Slab Layout:
    // 5 bytes "serum" + 8 bytes flags = 13 bytes
    // Then Slab header:
    //   bump_index (u32), free_list_len (u32), free_list_head (u32),
    //   root_node (u32), leaf_count (u32)

    if data.len() < 32 {
        return Err(anyhow!("Slab data too short"));
    }

    // Skip 13 bytes header
    let _leaf_count = u32::from_le_bytes(data[13 + 16..13 + 20].try_into()?) as usize;

    // Slab nodes start at offset 13 + 32 (header size)
    // Each node is 72 bytes
    // Tag: u32 (Uninitialized=0, InnerNode=1, LeafNode=2, FreeNode=3, LastFreeNode=4)

    let mut levels = Vec::new();
    let node_start = 13 + 32;
    let node_size = 72;

    for i in 0..((data.len() - node_start) / node_size) {
        let offset = node_start + i * node_size;
        let tag = u32::from_le_bytes(data[offset..offset + 4].try_into()?);

        if tag == 2 {
            // LeafNode
            // LeafNode Layout:
            // 0..4: tag
            // 4..8: owner_slot
            // 8..12: fee_tier
            // 12..20: key (u128) -> price is in the top 64 bits? No, price is key >> 64
            // 20..36: owner (Pubkey)
            // 36..44: quantity
            // 44..52: client_order_id

            let key = u128::from_le_bytes(data[offset + 8..offset + 24].try_into()?);
            let price_raw = (key >> 64) as u64;
            let quantity = u64::from_le_bytes(data[offset + 56..offset + 64].try_into()?);

            // human_price = (price_lots * quote_lot_size * 10^base_decimals) / (base_lot_size * 10^quote_decimals)
            // For SOL/USDC: 10^9 / 10^6 = 1000
            let price_factor = Decimal::from(1000); // TODO: Derive from mint decimals
            let price = (Decimal::from(price_raw) * Decimal::from(quote_lots) * price_factor)
                / Decimal::from(base_lots);
            let size = Decimal::from(quantity) * Decimal::from(base_lots)
                / Decimal::from(1_000_000_000u64); // to SOL

            levels.push(OrderbookLevel { price, size });
        }
    }

    // Sort levels: Bids descending, Asks ascending
    if is_bids {
        levels.sort_by(|a, b| b.price.cmp(&a.price));
    } else {
        levels.sort_by(|a, b| a.price.cmp(&b.price));
    }

    Ok(levels)
}

#[allow(dead_code, clippy::too_many_arguments)]
pub fn create_new_order_v3_instruction(
    market: &Pubkey,
    open_orders: &Pubkey,
    request_queue: &Pubkey,
    event_queue: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    base_vault: &Pubkey,
    quote_vault: &Pubkey,
    owner: &Pubkey,
    base_wallet: &Pubkey,
    quote_wallet: &Pubkey,
    side: u8, // 0 for Buy, 1 for Sell
    limit_price: u64,
    max_base_qty: u64,
    max_quote_qty: u64,
    order_type: u8, // 0 for Limit, 1 for IOC, 2 for PostOnly
    client_id: u64,
) -> solana_sdk::instruction::Instruction {
    let program_id =
        std::str::FromStr::from_str("srmqPvSwwJbtLZ9Uv7j8W7YVFe4Gz74Xp2Y7tENz7u4").unwrap(); // Serum V3

    // Instruction discriminator 10 for NewOrderV3
    let mut data = Vec::with_capacity(51);
    data.extend_from_slice(&10u32.to_le_bytes()); // tag
    data.push(side);
    data.extend_from_slice(&limit_price.to_le_bytes());
    data.extend_from_slice(&max_base_qty.to_le_bytes());
    data.extend_from_slice(&max_quote_qty.to_le_bytes());
    data.push(1); // self_trade_behavior: CancelProvide
    data.extend_from_slice(&(order_type as u32).to_le_bytes());
    data.extend_from_slice(&client_id.to_le_bytes());
    data.extend_from_slice(&65535u16.to_le_bytes()); // limit

    solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*market, false),
            solana_sdk::instruction::AccountMeta::new(*open_orders, false),
            solana_sdk::instruction::AccountMeta::new(*request_queue, false),
            solana_sdk::instruction::AccountMeta::new(*event_queue, false),
            solana_sdk::instruction::AccountMeta::new(*bids, false),
            solana_sdk::instruction::AccountMeta::new(*asks, false),
            solana_sdk::instruction::AccountMeta::new(*base_wallet, false),
            solana_sdk::instruction::AccountMeta::new(*quote_wallet, false),
            solana_sdk::instruction::AccountMeta::new(*base_vault, false),
            solana_sdk::instruction::AccountMeta::new(*quote_vault, false),
            solana_sdk::instruction::AccountMeta::new_readonly(
                solana_sdk::sysvar::rent::id(),
                false,
            ),
            solana_sdk::instruction::AccountMeta::new_readonly(
                solana_sdk::system_program::id(),
                false,
            ),
            solana_sdk::instruction::AccountMeta::new_readonly(*owner, true),
        ],
        data,
    }
}

#[allow(dead_code)]
pub fn create_jito_tip_instruction(
    owner: &Pubkey,
    tip_lamports: u64,
) -> solana_sdk::instruction::Instruction {
    let tip_accounts = [
        "96g9sR9SGvpH91qSS388Ppx6q6bT42p4t7rJ4vQp3u6C",
        "HFqU5x63VTqvQss8hp1uE17D3Jp2N6rBqA5VvL9Fv95v",
        "Cw8CFyMvGrnC7JvSbxujSAn61S19p9k8X1Yj8D1nK5sN",
    ];
    let tip_pubkey = std::str::FromStr::from_str(tip_accounts[0]).unwrap();

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
    let program_id =
        std::str::FromStr::from_str("srmqPvSwwJbtLZ9Uv7j8W7YVFe4Gz74Xp2Y7tENz7u4").unwrap();

    // Instruction discriminator 11 for CancelOrderV2
    let mut data = Vec::with_capacity(25);
    data.extend_from_slice(&11u32.to_le_bytes()); // tag
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
    let program_id =
        std::str::FromStr::from_str("srmqPvSwwJbtLZ9Uv7j8W7YVFe4Gz74Xp2Y7tENz7u4").unwrap();

    // Instruction discriminator 7 for CancelAllOrders (Serum/OpenBook)
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
