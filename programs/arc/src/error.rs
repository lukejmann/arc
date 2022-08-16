use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Src Balance < LP Deposit Amount.")]
    NotEnoughBalance,
    #[msg("Pool Mint Amount < 0 on LP Deposit")]
    NoPoolMintOutput,
    #[msg("Trying to burn too much")]
    BurnTooMuch,
    #[msg("Not enough out")]
    NotEnoughOut,
    #[msg("Public key mismatch")]
    PublicKeyMismatch,
    #[msg("Not initialized")]
    NotInitialized,
    #[msg("Incorrect owner")]
    IncorrectOwner,
    #[msg("Not a metadata account")]
    InvalidMetadataAccount,
    #[msg("Invalid pool type")]
    InvalidPoolType,
    #[msg("Invalid spot price")]
    InvalidSpotPrice,
    #[msg("Invalid curve type")]
    InvalidCurveType,
    #[msg("Invalid delta")]
    InvalidDelta,
    #[msg("Invalid proof")]
    InvalidProof,
}
