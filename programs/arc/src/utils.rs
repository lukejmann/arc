use mpl_token_metadata::state::TokenMetadataAccount;

use {
    crate::error::ErrorCode,

    anchor_lang::{
        prelude::*,
        solana_program::program_pack::{IsInitialized, Pack},
    },
    // anchor_spl::token::{Mint, Token, TokenAccount},
    anchor_spl::{
        associated_token::{self},
        token::{self, TokenAccount},
    },
    mpl_token_metadata::state::Metadata,
    spl_associated_token_account::get_associated_token_address,
    spl_token::state::Account,
    std::slice::Iter,
};

pub fn assert_is_ata(ata: &AccountInfo, wallet: &Pubkey, mint: &Pubkey) -> Result<Account> {
    assert_owned_by(ata, &spl_token::id())?;
    let ata_account: Account = assert_initialized(ata)?;
    assert_keys_equal(ata_account.owner, *wallet)?;
    assert_keys_equal(get_associated_token_address(wallet, mint), *ata.key)?;
    Ok(ata_account)
}

pub fn assert_keys_equal(key1: Pubkey, key2: Pubkey) -> Result<()> {
    if key1 != key2 {
        msg!("Key: {:?} != {:?}", key1, key2);
        Err(error!(ErrorCode::PublicKeyMismatch))
    } else {
        Ok(())
    }
}

pub fn assert_initialized<T: Pack + IsInitialized>(account_info: &AccountInfo) -> Result<T> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(error!(ErrorCode::NotInitialized))
    } else {
        Ok(account)
    }
}

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> Result<()> {
    if account.owner != owner {
        Err(error!(ErrorCode::IncorrectOwner))
    } else {
        Ok(())
    }
}

pub fn asset_collection<'a>(
    nft_mint: &AccountInfo,
    mint_metadata: &AccountInfo,
    vault_collection: Pubkey,
) -> Result<bool> {
    if !nft_mint.data_is_empty() {
        return Ok(false);
    }
    let (expected_metadata_key, expected_metadata_bump) = Pubkey::find_program_address(
        &[
            mpl_token_metadata::state::PREFIX.as_bytes(),
            mpl_token_metadata::id().as_ref(),
            nft_mint.key().as_ref(),
        ],
        &mpl_token_metadata::id(),
    );
    if expected_metadata_key != mint_metadata.key() {
        return Err(ErrorCode::InvalidMetadataAccount.into());
    }
    let metadata_attempt = Metadata::from_account_info(mint_metadata);
    if metadata_attempt.is_err() {
        return Err(ErrorCode::InvalidMetadataAccount.into());
    }
    let metadata: Metadata = metadata_attempt.unwrap();
    if metadata.collection.is_none() {
        return Err(ErrorCode::InvalidMetadataAccount.into());
    } else {
        let collection = metadata.collection.unwrap();
        if collection.key != vault_collection {
            return Err(ErrorCode::InvalidMetadataAccount.into());
        }
        Ok(true)
    }
}

pub fn asset_merkle<'a>(mint: Pubkey, root: [u8; 32], proof: Vec<[u8; 32]>) -> Result<bool> {
    let node = anchor_lang::solana_program::keccak::hashv(&[&mint.to_bytes()]);
    require!(verify(proof, root, node.0), ErrorCode::InvalidProof);
    Ok(true)
}

pub fn verify(proof: Vec<[u8; 32]>, root: [u8; 32], leaf: [u8; 32]) -> bool {
    let mut computed_hash = leaf;
    for proof_element in proof.into_iter() {
        if computed_hash <= proof_element {
            // Hash(current computed hash + current element of the proof)
            computed_hash =
                anchor_lang::solana_program::keccak::hashv(&[&computed_hash, &proof_element]).0;
        } else {
            // Hash(current element of the proof + current computed hash)
            computed_hash =
                anchor_lang::solana_program::keccak::hashv(&[&proof_element, &computed_hash]).0;
        }
    }
    // Check if the computed hash (root) is equal to the provided root
    computed_hash == root
}

pub fn assert_curve_type(curve_type: u8) -> Result<()> {
    if !(curve_type == 0 || curve_type == 1) {
        Err(error!(ErrorCode::InvalidCurveType))
    } else {
        Ok(())
    }
}

pub fn asset_pool_type(pool_type: u8) -> Result<()> {
    if !(pool_type == 0 || pool_type == 1 || pool_type == 2) {
        Err(error!(ErrorCode::InvalidPoolType))
    } else {
        Ok(())
    }
}

pub fn assert_delta(delta: i64, spot_price: u64) -> Result<()> {
    if (delta.abs() as u64) > spot_price {
        Err(error!(ErrorCode::InvalidDelta))
    } else {
        Ok(())
    }
}
