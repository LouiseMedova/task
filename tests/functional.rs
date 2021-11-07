#![cfg(feature = "test-bpf")]
use swap::{process_instruction, SwapInstruction};

use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_program::system_program;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{account::Account, signature::{Keypair, Signer}, transaction::Transaction};
use std::str::FromStr;


#[tokio::test]
async fn test_swap() {
    let program_id = Pubkey::from_str(&"Swap111111111111111111111111111111111111111").unwrap();

    let (wallet_a_pubkey, _) =
    Pubkey::find_program_address(&[b"wallet_a"], &program_id);

    let (wallet_b_pubkey, _) =
    Pubkey::find_program_address(&[b"wallet_b"], &program_id);

    let program_test = ProgramTest::new(
        "swap", 
        program_id, 
        processor!(process_instruction),
    );
  
    let wallet1 = Keypair::new();
    let wallet2 = Keypair::new();

    let mut ctx = program_test.start_with_context().await;

    ctx.banks_client
    .process_transaction(Transaction::new_signed_with_payer(
        &[
            system_instruction::transfer(
                &ctx.payer.pubkey(),
                &wallet1.pubkey(),
                1_000_000_000,
            ),
            system_instruction::transfer(
                &ctx.payer.pubkey(),
                &wallet2.pubkey(),
                1_000_000_000,
            ),
        ],
        Some(&ctx.payer.pubkey()),
        &[&ctx.payer],
        ctx.last_blockhash,
    ))
    .await
    .unwrap();

    let mut tx = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &SwapInstruction::CreateAccounts { amount_a: 1000, amount_b: 1000 } ,
            vec![
                AccountMeta::new(wallet1.pubkey(), false),
                AccountMeta::new(wallet_a_pubkey, false),
                AccountMeta::new(wallet_b_pubkey, false),
                AccountMeta::new(system_program::id(), false),
            ],
        )],
        Some(&wallet1.pubkey()),
    );

    tx.sign(&[&wallet1], ctx.last_blockhash);
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let wallet_a = ctx.banks_client
        .get_account(wallet_a_pubkey).await.expect("get_account").expect("associated_account not none");
    
    let wallet_b = ctx.banks_client
        .get_account(wallet_b_pubkey).await.expect("get_account").expect("associated_account not none");

    assert_eq!(wallet_a.lamports, 1000);
    assert_eq!(wallet_b.lamports, 1000);

    tx = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &SwapInstruction::SwapAtoB { amount: 500 } ,
            vec![
                AccountMeta::new(wallet1.pubkey(), false),
                AccountMeta::new(wallet2.pubkey(), false),
                AccountMeta::new(wallet_a_pubkey, false),
                AccountMeta::new(wallet_b_pubkey, false),
                AccountMeta::new(system_program::id(), false),
            ],
        )],
        Some(&wallet1.pubkey()),
    );

    tx.sign(&[&wallet1], ctx.last_blockhash);
    ctx.banks_client.process_transaction(tx).await.unwrap();

    let wallet_a = ctx.banks_client
        .get_account(wallet_a_pubkey).await.expect("get_account").expect("associated_account not none");
    
    let wallet_b = ctx.banks_client
        .get_account(wallet_b_pubkey).await.expect("get_account").expect("associated_account not none");

    assert_eq!(wallet_a.lamports, 1500);
    assert_eq!(wallet_b.lamports, 667);

    tx = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &SwapInstruction::SwapBtoA { amount: 100 } ,
            vec![
                AccountMeta::new(wallet1.pubkey(), false),
                AccountMeta::new(wallet2.pubkey(), false),
                AccountMeta::new(wallet_a_pubkey, false),
                AccountMeta::new(wallet_b_pubkey, false),
                AccountMeta::new(system_program::id(), false),
            ],
        )],
        Some(&wallet2.pubkey()),
    );

    tx.sign(&[&wallet2], ctx.last_blockhash);
    ctx.banks_client.process_transaction(tx).await.unwrap();


    let wallet_a = ctx.banks_client
        .get_account(wallet_a_pubkey).await.expect("get_account").expect("associated_account not none");
    
    let wallet_b = ctx.banks_client
        .get_account(wallet_b_pubkey).await.expect("get_account").expect("associated_account not none");

    assert_eq!(wallet_a.lamports, 1305);
    assert_eq!(wallet_b.lamports, 767);

}

