use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{Approve, Burn, Mint, MintTo, Token, TokenAccount, Transfer},
};
use mpl_token_metadata::state::Metadata;

use crate::error::ErrorCode;
use crate::{
    state::Pool,
    utils::{assert_is_ata, asset_valid_mint},
};

pub fn edit_delta(ctx: Context<EditDelta>, delta: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    assert_delta(delta, pool.spot_price)?;

    pool.delta = delta;

    Ok(())
}

pub fn edit_spot_price(ctx: Context<EditSpotPrice>, spot_price: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    assert_delta(pool.delta, spot_price)?;

    pool.spot_price = spot_price;

    Ok(())
}

#[derive(Accounts)]
pub struct EditDelta<'info> {
    #[account(mut, has_one = owner)]
    pub pool: Box<Account<'info, Pool>>,

    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct EditSpotPrice<'info> {
    #[account(mut, has_one = owner)]
    pub pool: Box<Account<'info, Pool>>,

    pub owner: Signer<'info>,
}
