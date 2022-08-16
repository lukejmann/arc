use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{Approve, Mint, Token, TokenAccount, Transfer},
};

use crate::{error::ErrorCode, utils::asset_merkle};
use crate::{
    state::Pool,
    utils::{assert_is_ata, asset_collection},
};

pub fn add_nft(ctx: Context<AddNFTLiquidity>, merkle_proof: Vec<[u8; 32]>) -> Result<()> {
    let pool_info = ctx.accounts.pool.to_account_info();
    let pool_key = ctx.accounts.pool.key();
    let pool = &mut ctx.accounts.pool;
    if pool.pool_type == 0 {
        return Err(ErrorCode::InvalidPoolType.into());
    }

    if pool.collection.is_some() {
        asset_collection(
            &ctx.accounts.nft_mint.to_account_info(),
            &ctx.accounts.mint_metadata,
            pool.collection.unwrap(),
        )?;
    }

    if pool.merkle_root.is_some() {
        asset_merkle(
            ctx.accounts.nft_mint.key(),
            pool.merkle_root.unwrap(),
            merkle_proof,
        )?;
    }

    assert_is_ata(
        &ctx.accounts.nft_mint.to_account_info(),
        &ctx.accounts.owner.key,
        &pool.mint,
    )?;

    assert!(ctx.accounts.nft_vault.amount == 0);
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.owner_ata.to_account_info(),
                to: ctx.accounts.nft_vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        1,
    )?;

    // assign delegate for front-end calls of getProgramAccounts
    let bump = *ctx.bumps.get("pool_auth").unwrap();
    let pda_sign = &[b"pool_auth", pool_key.as_ref(), &[bump]];
    token::approve(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Approve {
                to: ctx.accounts.nft_vault.to_account_info(),
                delegate: pool_info,
                authority: ctx.accounts.owner.to_account_info(),
            },
        )
        .with_signer(&[pda_sign]),
        1,
    )?;

    pool.n_nft += 1;
    update_pool_state(pool)?;
    Ok(())
}

pub fn remove_nft(ctx: Context<RemoveNFTLiquidity>) -> Result<()> {
    let pool_key = ctx.accounts.pool.key();
    let pool = &mut ctx.accounts.pool;
    if pool.pool_type == 0 {
        return Err(ErrorCode::InvalidPoolType.into());
    }

    assert_is_ata(
        &ctx.accounts.owner_ata.to_account_info(),
        ctx.accounts.owner.key,
        &ctx.accounts.nft_mint.key(),
    )?;

    let bump = *ctx.bumps.get("pool_auth").unwrap();
    let pda_sign = &[b"pool_auth", pool_key.as_ref(), &[bump]];

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.nft_vault.to_account_info(),
                to: ctx.accounts.owner_ata.to_account_info(),
                authority: ctx.accounts.pool_auth.to_account_info(),
            },
        )
        .with_signer(&[pda_sign]),
        1,
    )?;

    pool.n_nft -= 1;
    update_pool_state(pool)?;

    Ok(())
}

pub fn add_token_liquidity(ctx: Context<AddTokenLiquidity>, amount: u64) -> Result<()> {
    let pool_info = ctx.accounts.pool.to_account_info();
    let pool_key = ctx.accounts.pool.key();
    let pool = &mut ctx.accounts.pool;
    if pool.pool_type == 1 {
        return Err(ErrorCode::InvalidPoolType.into());
    }

    assert_is_ata(
        &ctx.accounts.token_mint.to_account_info(),
        &ctx.accounts.owner.key,
        &pool.mint,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.owner_ata.to_account_info(),
                to: ctx.accounts.token_vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        1,
    )?;

    // assign delegate for front-end calls of getProgramAccounts
    let bump = *ctx.bumps.get("pool_auth").unwrap();
    let pda_sign = &[b"pool_auth", pool_key.as_ref(), &[bump]];
    token::approve(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Approve {
                to: ctx.accounts.token_vault.to_account_info(),
                delegate: pool_info,
                authority: ctx.accounts.owner.to_account_info(),
            },
        )
        .with_signer(&[pda_sign]),
        amount,
    )?;

    pool.n_token = ctx.accounts.token_vault.amount;
    update_pool_state(pool)?;

    Ok(())
}

pub fn remove_token_liquidity(ctx: Context<RemoveTokenLiquidity>, amount: u64) -> Result<()> {
    let pool_key = ctx.accounts.pool.key();
    let pool = &mut ctx.accounts.pool;
    if pool.pool_type == 1 {
        return Err(ErrorCode::InvalidPoolType.into());
    }

    assert_is_ata(
        &ctx.accounts.owner_ata.to_account_info(),
        ctx.accounts.owner.key,
        &ctx.accounts.token_mint.key(),
    )?;

    let bump = *ctx.bumps.get("pool_auth").unwrap();
    let pda_sign = &[b"pool_auth", pool_key.as_ref(), &[bump]];

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_vault.to_account_info(),
                to: ctx.accounts.owner_ata.to_account_info(),
                authority: ctx.accounts.pool_auth.to_account_info(),
            },
        )
        .with_signer(&[pda_sign]),
        amount,
    )?;

    pool.n_token = ctx.accounts.token_vault.amount;
    update_pool_state(pool)?;

    Ok(())
}

pub fn update_pool_state(pool: &mut Box<Account<Pool>>) -> Result<()> {
    if pool.pool_type != 2 {
        return Ok(());
    }
    // todo: dubious
    if pool.n_token < pool.spot_price * pool.n_nft {
        pool.valid = false;
    }
    if pool.n_nft < pool.n_token / pool.spot_price {
        pool.valid = false;
    }
    Ok(())
}

#[derive(Accounts)]
pub struct AddNFTLiquidity<'info> {
    #[account(mut, has_one = owner)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(seeds=[b"pool_auth", pool.key().as_ref()], bump)]
    pub pool_auth: AccountInfo<'info>,

    #[account(seeds=[b"vault", pool.key().as_ref(), nft_mint.key().as_ref()], bump)]
    pub nft_vault: Box<Account<'info, TokenAccount>>,

    pub nft_mint: Account<'info, Mint>,

    pub mint_metadata: AccountInfo<'info>,

    #[account(mut, has_one = owner)]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    pub owner: Signer<'info>,

    // other
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RemoveNFTLiquidity<'info> {
    #[account(mut, has_one = owner)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(seeds=[b"pool_auth", pool.key().as_ref()], bump)]
    pub pool_auth: AccountInfo<'info>,

    #[account(mut,seeds=[b"vault", pool.key().as_ref(), nft_mint.key().as_ref()], bump,close=nft_vault)]
    pub nft_vault: Box<Account<'info, TokenAccount>>,

    pub nft_mint: Account<'info, Mint>,

    #[account(mut, has_one = owner)]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AddTokenLiquidity<'info> {
    #[account(mut, has_one = owner)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(seeds=[b"pool_auth", pool.key().as_ref()], bump)]
    pub pool_auth: AccountInfo<'info>,

    #[account(seeds=[b"vault", pool.key().as_ref(), token_mint.key().as_ref()], bump)]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    pub token_mint: Account<'info, Mint>,

    #[account(mut, has_one = owner)]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    pub owner: Signer<'info>,

    // other
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct RemoveTokenLiquidity<'info> {
    #[account(mut, has_one = owner)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(seeds=[b"pool_auth", pool.key().as_ref()], bump)]
    pub pool_auth: AccountInfo<'info>,

    #[account(mut,seeds=[b"vault", pool.key().as_ref(), token_mint.key().as_ref()], bump,close=token_vault)]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    pub token_mint: Account<'info, Mint>,

    #[account(mut, has_one = owner)]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EditDelta<'info> {
    #[account(mut, has_one = owner)]
    pub pool: Box<Account<'info, Pool>>,

    pub owner: Signer<'info>,
}
