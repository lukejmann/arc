use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Pool {
    pub collection: Option<Pubkey>,    // 1+32
    pub merkle_root: Option<[u8; 32]>, // 33 -> 1 + 1*32
    pub n_nft: u64,                    // 66 -> 8
    pub n_token: u64,                  // 74 -> 8
    pub mint: Pubkey,                  // 82 -> 32
    pub owner: Pubkey,                 // 114 -> 32
    pub owner_nonce: u8,               // 146 -> 1
    pub fee_bps: u64,                  // 147 -> 8
    pub curator: Pubkey,               // 155 -> 32
    pub curator_fee_bps: u64,          // 187 -> 8
    // 0=LINEAR, 1=EXPONENTIAL
    pub curve_type: u8, // 195 -> 1
    // 0=TOKEN, 1=NFT, 2=TRADE
    pub pool_type: u8,   // 196 -> 1
    pub delta: i64,      // 197 -> 8
    pub spot_price: u64, // 205 -> 8
    pub valid: bool,     // 213 -> 1
                         // 214 total
}
