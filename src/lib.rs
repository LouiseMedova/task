use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

entrypoint!(process_instruction);

const N: u64 = 100000000;
pub const WALLET_A_SEED: &str = "wallet_a";
pub const WALLET_B_SEED: &str = "wallet_b";

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum SwapInstruction {
    SwapAtoB { amount: u64 },
    SwapBtoA { amount: u64 },
    CreateAccounts { amount_a: u64, amount_b: u64 }
}

/// Accounts expected:
/// 0. `[signer, writable]` Debit lamports from this account
/// 1. `[writable]` Credit lamports to this account
/// 2. `[]` System program
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let acc_iter = &mut accounts.iter();
    
    match SwapInstruction::try_from_slice(input)? {
        SwapInstruction::SwapAtoB { amount } => {
            let user_wallet1 = next_account_info(acc_iter)?;
            let user_wallet2 = next_account_info(acc_iter)?;
            let wallet_a_info = next_account_info(acc_iter)?;
            let wallet_b_info = next_account_info(acc_iter)?;

            let balance_a = wallet_a_info.lamports();
            let balance_b = wallet_b_info.lamports();

            if !user_wallet1.is_signer {
                    return Err(ProgramError::MissingRequiredSignature);
                }

            let dy =  balance_b*N - (balance_a*balance_b*N)/(balance_a + amount);

            invoke(
                &system_instruction::transfer(user_wallet1.key, wallet_a_info.key, amount),
                &[user_wallet1.clone(), wallet_a_info.clone()],
            )?;

            **wallet_b_info.try_borrow_mut_lamports()? -= dy/N;
            **user_wallet2.try_borrow_mut_lamports()? += dy/N;

            msg!("Swap A to B  done");
        }
        SwapInstruction::SwapBtoA { amount } => {
            let user_wallet1 = next_account_info(acc_iter)?;
            let user_wallet2 = next_account_info(acc_iter)?;
            let wallet_a_info = next_account_info(acc_iter)?;
            let wallet_b_info = next_account_info(acc_iter)?;

            let balance_a = wallet_a_info.lamports();
            let balance_b = wallet_b_info.lamports();

            if !user_wallet2.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            let dx =  balance_a*N - (balance_a*balance_b*N)/(balance_b + amount);

            invoke(
                &system_instruction::transfer(user_wallet2.key, wallet_b_info.key, amount),
                &[user_wallet2.clone(), wallet_b_info.clone()],
            )?;
            
            **wallet_a_info.try_borrow_mut_lamports()? -= dx/N;
            **user_wallet1.try_borrow_mut_lamports()? += dx/N;

            msg!("Swap B to A  done");
        }
        SwapInstruction::CreateAccounts{ amount_a, amount_b } => {
            let payer = next_account_info(acc_iter)?;
            let wallet_a_info = next_account_info(acc_iter)?;
            let wallet_b_info = next_account_info(acc_iter)?;
            let system_program_info = next_account_info(acc_iter)?;

            if !payer.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            let (pda_a_pubkey, bump_seed_a) = Pubkey::find_program_address(&[WALLET_A_SEED.as_bytes()], program_id);
            let (pda_b_pubkey, bump_seed_b) = Pubkey::find_program_address(&[b"wallet_b"], program_id);

            if pda_a_pubkey != *wallet_a_info.key {
                return Err(ProgramError::InvalidArgument);
            }

            if pda_b_pubkey != *wallet_b_info.key {
                return Err(ProgramError::InvalidArgument);
            }

            let signer_seeds_a: &[&[_]] = &[WALLET_A_SEED.as_bytes(), &[bump_seed_a]];
            invoke_signed(
                &system_instruction::create_account(
                    payer.key,
                    &pda_a_pubkey,
                    amount_a,
                    0,
                    program_id,
                ),
                &[payer.clone(), wallet_a_info.clone(), system_program_info.clone()],
                &[&signer_seeds_a],
            )?;

            let signer_seeds_b: &[&[_]] = &[WALLET_B_SEED.as_bytes(), &[bump_seed_b]];
            invoke_signed(
                &system_instruction::create_account(
                    payer.key,
                    &pda_b_pubkey,
                    amount_b,
                    0,
                    program_id,
                ),
                &[payer.clone(), wallet_b_info.clone(), system_program_info.clone()],
                &[&signer_seeds_b],
            )?;
        }
    }

    Ok(())
}
