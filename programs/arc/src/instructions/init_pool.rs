
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};
use crate::{state::Pool, utils::{asset_pool_type, assert_curve_type, assert_delta}};


pub fn handler(
    ctx: Context<InitializePool>, 
    owner_nonce: u8,
    curve_type: u8,
    delta: i64,
    spot_price: u64,
    pool_type: u8,
    fee_bps: u64,
    curator_fee_bps: u64,
    merkle_root: [u8; 32]
) -> Result<()> {

    let pool = &mut ctx.accounts.pool;
    if ctx.accounts.collection.key() != ctx.accounts.system_program.key() {
    pool.collection = Some(ctx.accounts.collection.key());
    }
    if merkle_root != [0u8; 32] {
        pool.merkle_root = Some(merkle_root);
    }
    pool.n_nft = 0;
    pool.n_token = 0;
    pool.mint = ctx.accounts.mint.key();
    pool.owner = ctx.accounts.owner.key();
    pool.owner_nonce = owner_nonce;
    pool.fee_bps = fee_bps;
    pool.curator = ctx.accounts.curator.key();
    pool.curator_fee_bps = curator_fee_bps;
    assert_curve_type(curve_type)?;
    pool.curve_type = curve_type;
    assert_delta(delta, spot_price)?;
    pool.delta = delta;
    pool.spot_price = spot_price;
    asset_pool_type(pool_type)?;
    pool.pool_type = pool_type;

    Ok(())
}

#[derive(Accounts)]
#[instruction(owner_nonce: u8)]
pub struct InitializePool<'info> {
    ///CHECK: ?
    pub collection: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    pub curator: AccountInfo<'info>,

    #[account(
        init, 
        // toodo: fix space
        space = 200,
        payer=owner, 
        seeds=[b"pool", owner.key().as_ref(), collection.key().as_ref(), mint.key().as_ref(), &[owner_nonce]], 
        bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    // authority so 1 acc pass in can derive all other pdas 
    #[account(seeds=[b"pool_auth", pool.key().as_ref()], bump)]
    pub pool_auth: AccountInfo<'info>,

    // account to hold token X
    #[account(
        init, 
        payer=owner, 
        seeds=[b"token_vault", pool.key().as_ref()], 
        bump,
        token::mint = mint,
        token::authority = pool_auth
    )]
    pub token_vault: Box<Account<'info, TokenAccount>>, 

    #[account(mut)]
    pub owner: Signer<'info>,

    // accounts required to init a new mint
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

