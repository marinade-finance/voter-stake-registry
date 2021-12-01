use anchor_lang::Key;
use anchor_spl::token::TokenAccount;
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};

use program_test::*;

mod program_test;

#[allow(unaligned_references)]
#[tokio::test]
async fn test_clawback() -> Result<(), TransportError> {
    let context = TestContext::new().await;

    let community_token_mint = &context.mints[0];

    let realm_authority = &context.users[0].key;
    let realm_authority_ata = context.users[0].token_accounts[0];

    let voter_authority = &context.users[1].key;
    let voter_authority_ata = context.users[1].token_accounts[0];

    println!("create_realm");
    let realm = context
        .governance
        .create_realm(
            "testrealm",
            realm_authority.pubkey(),
            community_token_mint,
            &realm_authority,
            &context.addin.program_id,
        )
        .await;

    let token_owner_record = realm
        .create_token_owner_record(voter_authority.pubkey(), &realm_authority)
        .await;

    let registrar = context
        .addin
        .create_registrar(&realm, realm_authority)
        .await;

    println!("create_exchange_rate");
    let mngo_rate = context
        .addin
        .create_exchange_rate(
            &registrar,
            &realm_authority,
            realm_authority,
            0,
            community_token_mint,
            1,
        )
        .await;

    println!("create_voter");
    let voter = context
        .addin
        .create_voter(&registrar, &voter_authority, &realm_authority)
        .await;

    let realm_ata_initial = context
        .solana
        .token_account_balance(realm_authority_ata)
        .await;
    let voter_ata_initial = context
        .solana
        .token_account_balance(voter_authority_ata)
        .await;
    let vault_initial = mngo_rate.vault_balance(&context.solana).await;
    assert_eq!(vault_initial, 0);
    let voter_balance_initial = voter.deposit_amount(&context.solana, 0).await;
    assert_eq!(voter_balance_initial, 0);

    println!("create_deposit");
    context
        .addin
        .create_deposit(
            &registrar,
            &voter,
            voter_authority,
            &mngo_rate,
            realm_authority,
            realm_authority_ata,
            governance_registry::account::LockupKind::Daily,
            10000,
            10,
            true,
        )
        .await?;

    let realm_ata_after_deposit = context
        .solana
        .token_account_balance(realm_authority_ata)
        .await;
    assert_eq!(realm_ata_initial, realm_ata_after_deposit + 10000);
    let vault_after_deposit = mngo_rate.vault_balance(&context.solana).await;
    assert_eq!(vault_after_deposit, 10000);
    let voter_balance_after_deposit = voter.deposit_amount(&context.solana, 0).await;
    assert_eq!(voter_balance_after_deposit, 10000);

    println!("withdraw");
    context
        .addin
        .withdraw(
            &registrar,
            &voter,
            &token_owner_record,
            &mngo_rate,
            &voter_authority,
            voter_authority_ata,
            0,
            10000,
        )
        .await
        .expect_err("fails because a deposit happened in the same slot");

    // Must advance slots because withdrawing in the same slot as the deposit is forbidden
    // Advance almost three days for some vesting to kick in
    context
        .addin
        .set_time_offset(&registrar, &realm_authority, (3 * 24 - 1) * 60 * 60)
        .await;
    context.solana.advance_clock_by_slots(2).await;

    println!("clawback");
    context
        .addin
        .clawback(
            &registrar,
            &voter,
            &token_owner_record,
            &mngo_rate,
            &voter_authority,
            realm_authority_ata,
            0,
        )
        .await?;

    println!("withdraw");
    context
        .addin
        .withdraw(
            &registrar,
            &voter,
            &token_owner_record,
            &mngo_rate,
            &voter_authority,
            voter_authority_ata,
            0,
            2000,
        )
        .await?;

    let realm_after_clawback = context
        .solana
        .token_account_balance(realm_authority_ata)
        .await;
    assert_eq!(realm_ata_initial - 2000, realm_after_clawback);
    let voter_after_withdraw = context
        .solana
        .token_account_balance(voter_authority_ata)
        .await;
    assert_eq!(voter_after_withdraw, voter_ata_initial + 2000);
    let vault_after_withdraw = mngo_rate.vault_balance(&context.solana).await;
    assert_eq!(vault_after_withdraw, 0);
    let voter_balance_after_withdraw = voter.deposit_amount(&context.solana, 0).await;
    assert_eq!(voter_balance_after_withdraw, 0);

    Ok(())
}