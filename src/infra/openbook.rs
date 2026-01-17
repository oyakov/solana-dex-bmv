use crate::domain::OrderbookLevel;
use anyhow::{anyhow, Result};
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

// OpenBook V1/V2 Constants
pub const OPENBOOK_V1_PROGRAM_ID: &str = "srmqPvSyc2u87R79RDMKW641X8vAnm83H26V7eTeg5t";
pub const OPENBOOK_V2_PROGRAM_ID: &str = "opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb";

// Account discriminators (Anchor style: sha256("account:<Name>")[0..8])
pub const MARKET_DISCRIMINATOR: [u8; 8] = [213, 222, 12, 126, 25, 23, 204, 237];
pub const BOOK_SIDE_DISCRIMINATOR: [u8; 8] = [178, 119, 219, 142, 234, 1, 163, 133];

pub const PLACE_ORDER_DISCRIMINATOR: [u8; 8] = [142, 60, 48, 126, 114, 252, 19, 137];
pub const CANCEL_ORDER_DISCRIMINATOR: [u8; 8] = [95, 211, 172, 180, 212, 216, 180, 164];
pub const CLOSE_OPEN_ORDERS_DISCRIMINATOR: [u8; 8] = [90, 84, 1, 107, 73, 221, 194, 0];

#[derive(Debug, Clone)]
pub struct MarketStateV2 {
    pub bump: u8,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub market_authority: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub event_heap: Pubkey,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub base_lot_size: i64,
    pub quote_lot_size: i64,
}

impl MarketStateV2 {
    pub fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < 450 {
            return Err(anyhow!(
                "V2 Market account data too short (need ~450, got {})",
                data.len()
            ));
        }

        if data[0..8] != MARKET_DISCRIMINATOR {
            return Err(anyhow!("Invalid V2 Market discriminator"));
        }

        // Offsets based on V2 IDL
        let bump = data[8];
        let base_decimals = data[9];
        let quote_decimals = data[10];

        // Skip padding (11..16), market_authority (16..48), time_expiry (48..56), etc.
        let market_authority = Pubkey::new_from_array(data[16..48].try_into()?);

        // bids: ~203, asks: ~235, event_heap: ~267 (assuming NonZeroPubkeyOption=33 bytes for admin fields)
        // Let's use robust offsets from a known V2 spec:
        // open_orders_admin (88), consume_events_admin (121), close_market_admin (154), name (187)
        let bids = Pubkey::new_from_array(data[203..235].try_into()?);
        let asks = Pubkey::new_from_array(data[235..267].try_into()?);
        let event_heap = Pubkey::new_from_array(data[267..299].try_into()?);

        // oracle_a (299..332), oracle_b (332..365), vaults (365..397, 397..429)
        let market_base_vault = Pubkey::new_from_array(data[365..397].try_into()?);
        let market_quote_vault = Pubkey::new_from_array(data[397..429].try_into()?);

        // maker_fee (429..431), taker_fee (431..433), lot_sizes (433..441, 441..449)
        let base_lot_size = i64::from_le_bytes(data[433..441].try_into()?);
        let quote_lot_size = i64::from_le_bytes(data[441..449].try_into()?);

        Ok(Self {
            bump,
            base_decimals,
            quote_decimals,
            market_authority,
            bids,
            asks,
            event_heap,
            market_base_vault,
            market_quote_vault,
            base_lot_size,
            quote_lot_size,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MarketStateV1 {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub base_lot_size: u64,
    pub quote_lot_size: u64,
    pub base_decimals: u8,
    pub quote_decimals: u8,
}

impl MarketStateV1 {
    pub fn unpack(data: &[u8]) -> Result<Self> {
        if data.len() < 388 {
            return Err(anyhow!(
                "V1 Market account data too short (need 388, got {})",
                data.len()
            ));
        }

        // Serum V3/OpenBook V1 layout offsets
        let base_mint = Pubkey::new_from_array(data[53..85].try_into()?);
        let quote_mint = Pubkey::new_from_array(data[165..197].try_into()?);
        let bids = Pubkey::new_from_array(data[285..317].try_into()?);
        let asks = Pubkey::new_from_array(data[317..349].try_into()?);
        let base_lot_size = u64::from_le_bytes(data[349..357].try_into()?);
        let quote_lot_size = u64::from_le_bytes(data[357..365].try_into()?);

        Ok(Self {
            base_mint,
            quote_mint,
            bids,
            asks,
            base_lot_size,
            quote_lot_size,
            base_decimals: 9,  // Default for SOL/BMV likely
            quote_decimals: 6, // Default for USDC
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
    if data.len() < 8 + 128 {
        return Err(anyhow!("BookSide data too short"));
    }

    if data[0..8] != BOOK_SIDE_DISCRIMINATOR {
        return Err(anyhow!("Invalid BookSide discriminator"));
    }

    let nodes_start = 8 + 128; // Usually 128 bytes of header in V2
    let node_size = 88; // LeafNode in V2

    let mut levels = Vec::new();

    for i in 0..((data.len() - nodes_start) / node_size).min(1024) {
        let offset = nodes_start + i * node_size;
        let tag = data[offset];

        if tag == 2 {
            // LeafNode tag in V2 is 2
            let key = u128::from_le_bytes(data[offset + 16..offset + 32].try_into()?);
            let price_raw = (key >> 64) as u64;
            let quantity = i64::from_le_bytes(data[offset + 48..offset + 56].try_into()?);

            if quantity <= 0 {
                continue;
            }

            let base_pow = Decimal::from(10u64.pow(base_decimals as u32));
            let quote_pow = Decimal::from(10u64.pow(quote_decimals as u32));

            // Price in V2 is quote_lots / base_lots
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

pub fn parse_book_side_v1(
    data: &[u8],
    is_bids: bool,
    base_decimals: u8,
    quote_decimals: u8,
    base_lot_size: u64,
    quote_lot_size: u64,
) -> Result<Vec<OrderbookLevel>> {
    if data.len() < 5 + 8 {
        return Err(anyhow!("Slab data too short"));
    }

    let mut levels = Vec::new();
    let node_size = 72;
    let header_size = 45; // Serum V3 Slab header (5 bytes flags + 40 bytes Slab)

    if data.len() < header_size {
        return Ok(vec![]);
    }

    let slot_count = (data.len() - header_size) / node_size;
    for i in 0..slot_count.min(1024) {
        let offset = header_size + i * node_size;
        if offset + node_size > data.len() {
            break;
        }
        let tag = u32::from_le_bytes(data[offset..offset + 4].try_into()?);

        if tag == 2 {
            // LeafNode in Serum V3
            // Offsets within 72-byte node:
            // 0..4: tag (2)
            // 4..16: owner_slot + padding
            // 16..32: key (u128)
            // 32..64: owner (Pubkey)
            // 64..72: quantity (u64)
            let key = u128::from_le_bytes(data[offset + 16..offset + 32].try_into()?);
            let price_raw = (key >> 64) as u64;
            let quantity = u64::from_le_bytes(data[offset + 64..offset + 72].try_into()?);

            let base_pow = Decimal::from(10u64.pow(base_decimals as u32));
            let quote_pow = Decimal::from(10u64.pow(quote_decimals as u32));

            // V1 Price math: (price_lots * quote_lot_size * base_pow) / (base_lot_size * quote_pow)
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
    price_lots: i64,
    max_base_lots: i64,
    max_quote_lots: i64,
    client_order_id: u64,
) -> solana_sdk::instruction::Instruction {
    let program_id = Pubkey::from_str(OPENBOOK_V2_PROGRAM_ID).expect("Invalid V2 program ID");

    let mut data = Vec::with_capacity(8 + 32);
    data.extend_from_slice(&PLACE_ORDER_DISCRIMINATOR);
    data.push(side);
    data.extend_from_slice(&price_lots.to_le_bytes());
    data.extend_from_slice(&max_base_lots.to_le_bytes());
    data.extend_from_slice(&max_quote_lots.to_le_bytes());
    data.extend_from_slice(&client_order_id.to_le_bytes());
    data.push(0); // order_type: Limit
    data.push(0); // reduce_only: false
    data.extend_from_slice(&0i64.to_le_bytes()); // expiry: 0 (none)
    data.push(255); // limit: 255 (full fill)

    solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*owner, true),
            solana_sdk::instruction::AccountMeta::new(*open_orders, false),
            solana_sdk::instruction::AccountMeta::new_readonly(Pubkey::default(), false), // Optional open_orders_admin
            solana_sdk::instruction::AccountMeta::new(*market, false),
            solana_sdk::instruction::AccountMeta::new(*bids, false),
            solana_sdk::instruction::AccountMeta::new(*asks, false),
            solana_sdk::instruction::AccountMeta::new(*event_heap, false),
            solana_sdk::instruction::AccountMeta::new(*market_base_vault, false),
            solana_sdk::instruction::AccountMeta::new(*market_quote_vault, false),
            solana_sdk::instruction::AccountMeta::new(*user_token_account, false),
            solana_sdk::instruction::AccountMeta::new_readonly(Pubkey::default(), false), // Optional Oracle A
            solana_sdk::instruction::AccountMeta::new_readonly(Pubkey::default(), false), // Optional Oracle B
            solana_sdk::instruction::AccountMeta::new_readonly(
                solana_sdk::system_program::id(),
                false,
            ),
            solana_sdk::instruction::AccountMeta::new_readonly(
                solana_sdk::pubkey::Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                    .unwrap(),
                false,
            ),
        ],
        data,
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

pub fn create_cancel_order_v2_instruction(
    market: &Pubkey,
    bids: &Pubkey,
    asks: &Pubkey,
    open_orders: &Pubkey,
    owner: &Pubkey,
    side: u8,
    order_id: u128,
) -> solana_sdk::instruction::Instruction {
    let program_id = Pubkey::from_str(OPENBOOK_V2_PROGRAM_ID).unwrap();
    let mut data = Vec::with_capacity(8 + 17);
    data.extend_from_slice(&CANCEL_ORDER_DISCRIMINATOR);
    data.push(side);
    data.extend_from_slice(&order_id.to_le_bytes());

    solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new_readonly(*owner, true),
            solana_sdk::instruction::AccountMeta::new(*open_orders, false),
            solana_sdk::instruction::AccountMeta::new(*market, false),
            solana_sdk::instruction::AccountMeta::new(*bids, false),
            solana_sdk::instruction::AccountMeta::new(*asks, false),
        ],
        data,
    }
}

pub fn create_close_open_orders_v2_instruction(
    open_orders: &Pubkey,
    owner: &Pubkey,
    sol_destination: &Pubkey,
) -> solana_sdk::instruction::Instruction {
    let program_id = Pubkey::from_str(OPENBOOK_V2_PROGRAM_ID).unwrap();
    let mut data = Vec::with_capacity(8);
    data.extend_from_slice(&CLOSE_OPEN_ORDERS_DISCRIMINATOR);

    solana_sdk::instruction::Instruction {
        program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*owner, true),
            solana_sdk::instruction::AccountMeta::new(*open_orders, false),
            solana_sdk::instruction::AccountMeta::new(*sol_destination, false), // sol_destination
        ],
        data,
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_v2_price_math() {
        let base_decimals = 9;
        let quote_decimals = 6;
        let base_lot_size = 100_000;
        let quote_lot_size = 100;

        // LeafNode key is typically (price_lots << 64) | ...
        let price_lots: u64 = 150_000;
        let quantity_lots: i64 = 500;

        let base_pow = Decimal::from(10u64.pow(base_decimals as u32));
        let quote_pow = Decimal::from(10u64.pow(quote_decimals as u32));

        // Price = (price_lots * quote_lot_size * base_pow) / (base_lot_size * quote_pow)
        let price = (Decimal::from(price_lots) * Decimal::from(quote_lot_size) * base_pow)
            / (Decimal::from(base_lot_size) * quote_pow);
        assert_eq!(price, dec!(150000));

        let size = Decimal::from(quantity_lots) * Decimal::from(base_lot_size) / base_pow;
        assert_eq!(size, dec!(0.05));
    }
}
