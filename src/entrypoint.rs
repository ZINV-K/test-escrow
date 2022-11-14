use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, instruction, pubkey::Pubkey,
};

use crate::processor::Processor;

// 명령을 실행하는 매크로
entrypoint!(process_instruction);

// 명령
fn process_instruction(
    // 프로그램 ID
    program_id: &Pubkey,
    // 어카운트 정보들
    accounts: &[AccountInfo],
    // 명령 데이터 (state?)
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, instruction_data)
}
