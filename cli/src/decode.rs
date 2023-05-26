use anchor_lang::{Discriminator};
use anyhow::{anyhow, bail, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use std::io::BufRead;
use voter_stake_registry::state::*;

//
// Output format declarations. These are built from the decoded
// accounts and then converted to JSON.
//

#[derive(Serialize)]
struct DisplayDepositEntry {
    allow_clawback: bool,
    mint_index: u8,
    unlocked_now: u64,
    locked_now: u64,
    locked_1y: u64,
    locked_2y: u64,
    locked_3y: u64,
    locked_4y: u64,
    locked_5y: u64,
}

#[derive(Serialize)]
struct DisplayVoter {
    voter_authority: String,
    registrar: String,
    deposit_entries: Vec<DisplayDepositEntry>,
}

/// Decode a Voter account and print its JSON to stdout
fn decode_voter(data_voter: &[u8], data_registrar: &[u8]) -> Result<()> {
    let mut data = data_voter;
    let voter: Voter = anchor_lang::AccountDeserialize::try_deserialize(&mut data)?;
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() as i64;
    let year = 365 * 24 * 60 * 60;
    let ser = DisplayVoter {
        voter_authority: voter.voter_authority.to_string(),
        registrar: voter.registrar.to_string(),
        deposit_entries: voter
            .deposits
            .iter()
            .filter(|d| d.is_used)
            .map(|d| DisplayDepositEntry {
                allow_clawback: d.allow_clawback,
                mint_index: d.voting_mint_config_idx,
                unlocked_now: d.amount_unlocked(now_ts),
                locked_now: d.amount_locked(now_ts),
                locked_1y: d.amount_locked(now_ts + year),
                locked_2y: d.amount_locked(now_ts + 2 * year),
                locked_3y: d.amount_locked(now_ts + 3 * year),
                locked_4y: d.amount_locked(now_ts + 4 * year),
                locked_5y: d.amount_locked(now_ts + 5 * year),
            })
            .collect(),
    };

    let mut data = data_registrar;
    let registrar: Registrar = anchor_lang::AccountDeserialize::try_deserialize(&mut data)?;
    let voter_weight = voter.weight(&registrar);

    println!("{}", serde_json::to_string(&ser)?);
    println!("weight: {}", voter_weight?);
    Ok(())
}

// Read a sequence of base64 encoded accounts from stdin
// and write their decoded versions back out as JSON.
pub fn decode_account() -> Result<()> {
    let account_types = HashMap::from([(Voter::discriminator(), &decode_voter)]);

    let mut lines = io::stdin().lock().lines();
    let mut line_voter: Option<String> = None;
    let mut line_registrar: Option<String> = None;

    while let Some(line) = lines.next() {
        let line_trimmed = line?.trim().to_string();
        if line_trimmed.starts_with("#") {
            continue
        }
        if line_voter.is_none() {
            line_voter = Some(line_trimmed);
            continue
        } else if line_registrar.is_none()  {
            line_registrar = Some(line_trimmed);
        }

        let data_voter = base64::decode(line_voter.unwrap())?;
        let data_registrar = base64::decode(line_registrar.unwrap())?;

        if data_voter.len() < 8 || data_registrar.len() < 8 {
            bail!("data length {}/{} too small for discriminator", data_voter.len(), data_registrar.len());
        }
        let discr_voter = &data_voter[0..8];
        let handler = account_types
            .get(discr_voter)
            .ok_or_else(|| anyhow!("discriminator {:?} not recognized", discr_voter))?;
        let discr_registar = &data_registrar[0..8];
        if Registrar::discriminator() != discr_registar {
            bail!("discriminator registar {:?} not recognized", discr_registar);
        }

        handler(&data_voter, &data_registrar)?;

        line_voter = None;
        line_registrar = None;
    }

    Ok(())
}
