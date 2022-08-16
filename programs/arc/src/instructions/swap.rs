use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token::{Mint, Token, TokenAccount, Transfer},
};

use crate::{
    error::ErrorCode,
    utils::{assert_is_ata, asset_merkle},
};
use crate::{state::Pool, utils::asset_collection};

pub fn swap_for_nft(ctx: Context<SwapForNFT>) -> Result<()> {
    let pool_key = ctx.accounts.pool.key();
    let pool = &mut ctx.accounts.pool;
    if pool.pool_type == 0 {
        return Err(ErrorCode::InvalidPoolType.into());
    }

    assert_is_ata(
        &ctx.accounts.authority_token_ata.to_account_info(),
        &ctx.accounts.authority.key,
        &pool.mint,
    )?;

    assert_is_ata(
        &ctx.accounts.authority_nft_ata.to_account_info(),
        &ctx.accounts.authority.key,
        &ctx.accounts.nft_mint.key(),
    )?;

    let bump = *ctx.bumps.get("pool_auth").unwrap();
    let pda_sign = &[b"pool_auth", pool_key.as_ref(), &[bump]];

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.nft_vault.to_account_info(),
                to: ctx.accounts.authority_token_ata.to_account_info(),
                authority: ctx.accounts.pool_auth.to_account_info(),
            },
        )
        .with_signer(&[pda_sign]),
        1,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.authority_token_ata.to_account_info(),
                to: ctx.accounts.token_vault.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        pool.spot_price,
    )?;

    if pool.pool_type == 2 {
        let pool_fee = pool
            .spot_price
            .checked_mul(pool.fee_bps)
            .unwrap()
            .checked_div(10000)
            .unwrap();

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.authority_token_ata.to_account_info(),
                    to: ctx.accounts.token_vault.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            pool_fee,
        )?;
    }

    let curator_fee = pool
        .spot_price
        .checked_mul(pool.curator_fee_bps)
        .unwrap()
        .checked_div(10000)
        .unwrap();

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.authority_token_ata.to_account_info(),
                to: ctx.accounts.token_vault.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        curator_fee,
    )?;

    let new_spot_price;
    if pool.curve_type == 0 {
        // linear curve
        new_spot_price = (pool.spot_price as i64)
            .checked_add(pool.delta as i64)
            .unwrap();
    } else {
        // exponential curve
        new_spot_price = (pool.spot_price as i64)
            .checked_mul((100 as i64).checked_add(pool.delta).unwrap())
            .unwrap()
            .checked_div(100 as i64)
            .unwrap();
    };
    if new_spot_price < 0 {
        return Err(ErrorCode::InvalidSpotPrice.into());
    };
    pool.spot_price = new_spot_price as u64;
    pool.n_token = ctx.accounts.token_vault.amount;
    pool.n_nft -= 1;
    Ok(())
}

pub fn swap_for_token(ctx: Context<SwapForToken>, merkle_proof: Vec<[u8; 32]>) -> Result<()> {
    let pool_key = ctx.accounts.pool.key();
    let pool = &mut ctx.accounts.pool;
    if pool.pool_type == 0 {
        return Err(ErrorCode::InvalidPoolType.into());
    }

    if pool.collection.is_some() {
        asset_collection(
            &ctx.accounts.nft_mint.to_account_info(),
            &ctx.accounts.nft_mint_metadata,
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
        &ctx.accounts.authority_token_ata.to_account_info(),
        &ctx.accounts.authority.key,
        &pool.mint,
    )?;

    assert_is_ata(
        &ctx.accounts.authority_nft_ata.to_account_info(),
        &ctx.accounts.authority.key,
        &ctx.accounts.nft_mint.key(),
    )?;

    let bump = *ctx.bumps.get("pool_auth").unwrap();
    let pda_sign = &[b"pool_auth", pool_key.as_ref(), &[bump]];

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_vault.to_account_info(),
                to: ctx.accounts.authority_token_ata.to_account_info(),
                authority: ctx.accounts.pool_auth.to_account_info(),
            },
        )
        .with_signer(&[pda_sign]),
        pool.spot_price,
    )?;

    assert!(ctx.accounts.nft_vault.amount == 0);
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.authority_nft_ata.to_account_info(),
                to: ctx.accounts.nft_vault.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        1,
    )?;

    if pool.pool_type == 2 {
        let pool_fee = pool
            .spot_price
            .checked_mul(pool.fee_bps)
            .unwrap()
            .checked_div(10000)
            .unwrap();

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.authority_token_ata.to_account_info(),
                    to: ctx.accounts.token_vault.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            pool_fee,
        )?;
    }

    let curator_fee = pool
        .spot_price
        .checked_mul(pool.curator_fee_bps)
        .unwrap()
        .checked_div(10000)
        .unwrap();

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.authority_token_ata.to_account_info(),
                to: ctx.accounts.token_vault.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        ),
        curator_fee,
    )?;

    let new_spot_price;
    if pool.curve_type == 0 {
        // linear curve
        new_spot_price = (pool.spot_price as i64)
            .checked_sub(pool.delta as i64)
            .unwrap();
    } else {
        // exponential curve
        new_spot_price = (pool.spot_price as i64)
            .checked_mul((100 as i64).checked_sub(pool.delta).unwrap())
            .unwrap()
            .checked_div(100 as i64)
            .unwrap();
    };
    if new_spot_price < 0 {
        return Err(ErrorCode::InvalidSpotPrice.into());
    };
    pool.spot_price = new_spot_price as u64;
    pool.n_token = ctx.accounts.token_vault.amount;
    pool.n_nft += 1;
    Ok(())
}

#[derive(Accounts)]
pub struct SwapForNFT<'info> {
    #[account(mut, constraint=pool.valid)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(seeds=[b"pool_auth", pool.key().as_ref()], bump)]
    pub pool_auth: AccountInfo<'info>,

    #[account(seeds=[b"vault", pool.key().as_ref(), nft_mint.key().as_ref()], bump)]
    pub nft_vault: Box<Account<'info, TokenAccount>>,

    pub nft_mint: Account<'info, Mint>,

    #[account(seeds=[b"vault", pool.key().as_ref(), nft_mint.key().as_ref()], bump)]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority_token_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub authority_nft_ata: Box<Account<'info, TokenAccount>>,

    pub authority: Signer<'info>,

    // other
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SwapForToken<'info> {
    #[account(mut, constraint=pool.valid)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(seeds=[b"pool_auth", pool.key().as_ref()], bump)]
    pub pool_auth: AccountInfo<'info>,

    #[account(seeds=[b"vault", pool.key().as_ref(), nft_mint.key().as_ref()], bump)]
    pub nft_vault: Box<Account<'info, TokenAccount>>,

    pub nft_mint: Account<'info, Mint>,

    pub nft_mint_metadata: AccountInfo<'info>,

    #[account(seeds=[b"vault", pool.key().as_ref(), nft_mint.key().as_ref()], bump)]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub authority_token_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub authority_nft_ata: Box<Account<'info, TokenAccount>>,

    pub authority: Signer<'info>,

    // other
    pub token_program: Program<'info, Token>,
}
