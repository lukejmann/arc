use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("8u3q7KUBALJgKLpmxA94RNkuYxfmrw9C9oiM1Dtb8yNd");

#[program]
pub mod arc {
    use super::*;

    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        owner_nonce: u8,
        curve_type: u8,
        delta: i64,
        spot_price: u64,
        pool_type: u8,
        fee_bps: u64,
        curator_fee_bps: u64,
        merkle_root: [u8; 32],
    ) -> Result<()> {
        init_pool::handler(
            ctx,
            owner_nonce,
            curve_type,
            delta,
            spot_price,
            pool_type,
            fee_bps,
            curator_fee_bps,
            merkle_root,
        )
    }

    pub fn add_nft(ctx: Context<AddNFTLiquidity>, merkle_proof: Vec<[u8; 32]>) -> Result<()> {
        liquidity::add_nft(ctx, merkle_proof)
    }
    pub fn remove_nft(ctx: Context<RemoveNFTLiquidity>) -> Result<()> {
        liquidity::remove_nft(ctx)
    }
    pub fn add_token_liquidity(ctx: Context<AddTokenLiquidity>, amount: u64) -> Result<()> {
        liquidity::add_token_liquidity(ctx, amount)
    }
    pub fn remove_token_liquidity(ctx: Context<RemoveTokenLiquidity>, amount: u64) -> Result<()> {
        liquidity::remove_token_liquidity(ctx, amount)
    }

    pub fn swap_for_nft(ctx: Context<SwapForNFT>) -> Result<()> {
        swap::swap_for_nft(ctx)
    }
    pub fn swap_for_token(ctx: Context<SwapForToken>, merkle_proof: Vec<[u8; 32]>) -> Result<()> {
        swap::swap_for_token(ctx, merkle_proof)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
