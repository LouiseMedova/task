use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
};

entrypoint!(process_instruction);

const N: u64 = 100000000;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum SwapInstruction {
    SwapAtoB { amount: u64 },
    SwapBtoA { amount: u64 },
}

/// Accounts expected:
/// 0. `[signer, writable]` Debit lamports from this account
/// 1. `[writable]` Credit lamports to this account
/// 2. `[]` System program
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    let acc_iter = &mut accounts.iter();
    let user_wallet1 = next_account_info(acc_iter)?;
    let user_wallet2 = next_account_info(acc_iter)?;
    let wallet_a_info = next_account_info(acc_iter)?;
    let wallet_b_info = next_account_info(acc_iter)?;

    let balance_a = wallet_a_info.lamports();
    let balance_b = wallet_b_info.lamports();

    match SwapInstruction::try_from_slice(input)? {
        SwapInstruction::SwapAtoB { amount } => {
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
    }

    Ok(())
}
