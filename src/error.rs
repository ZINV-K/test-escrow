use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum EscrowError {
    // 유효하지 않은 명령에 대한 에러
    #[error("Invalid Instruction")]
    InvalidInstruction,

    // 임대료(렌트비) 면제 아님
    #[error("Not Rent Exempt")]
    NotRentExcept,
}

// From은 무엇?
// *** ProgramError에 EsocrowError를 확장해 추가(?)
impl From<EscrowError> for ProgramError {
    // EscrowError 형식을 받아 ProgramError로 반환
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
