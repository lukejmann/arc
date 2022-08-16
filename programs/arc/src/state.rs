use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Pool {
    pub collection: Option<Pubkey>,
    pub merkle_root: Option<[u8; 32]>,
    pub n_nft: u64,
    pub n_token: u64,
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub owner_nonce: u8,
    pub fee_bps: u64,
    pub curator: Pubkey,
    pub curator_fee_bps: u64,
    // 0=LINEAR, 1=EXPONENTIAL
    pub curve_type: u8,
    // 0=TOKEN, 1=NFT, 2=TRADE
    pub pool_type: u8,
    pub delta: i64,
    pub spot_price: u64,
    pub valid: bool,
    // todo: add merkle proof
}
