use crate::error::EscrowError::InvalidInstruction;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

pub enum EscrowInstruction {
    /// 에스크로 계정을 생성 및 채우고 주어진 임시 토큰 계정의 소유권을 PDA로 이전하여 거래를 시작합니다.
    ///
    ///
    /// 예상 계정:
    ///
    /// 0. `[signer]` 에스크로를 초기화하는 사람의 계정
    /// 1. `[writable]` 이 명령어 이전에 생성되어야 하고 이니셜라이저가 소유해야 하는 임시 토큰 계정
    /// 2. `[]` 거래가 진행되면 받을 토큰에 대한 이니셜라이저의 토큰 계정
    /// 3. `[writable]` 에스크로 계정은 거래에 필요한 모든 정보를 보유합니다.
    /// 4. `[]` 임대 시스템 변수
    /// 5. `[]` 토큰 프로그램
    ///
    /// ***이넘인데 스트럭트(?)
    InitEscrow {
        /// 당사자 A가 받게 될 토큰 Y의 예상하는 금액
        amount: u64,
    },
}

impl EscrowInstruction {
    /// 바이트 버퍼를 [EscrowInstruction](enum.EscrowInstruction.html)안으로 압축을 풉니다.
    /// 버퍼 u8타입의 배열을 받아서 Result로 반환
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        // 입력 받은 값을 까봐서(unwrap) 정상적이면 넘어감(ok) 또는 커스텀 에러 발생
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            // 태그가 0이면 EscrowInstruction의 InitEscrow
            0 => Self::InitEscrow {
                amount: Self::unpack_amount(rest)?,
            },
            // 태그가 0이 아니면 커스텀 에러 타입(EscrowError) 전송
            // *** into는 무슨 용도(?)
            _ => return Err(InvalidInstruction.into()),
        })
    }

    pub fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        // input 으로 부터 값을 받음
        let amount = input
            // 배열에서 7번째까지의 u8형 데이터를 가져옴
            .get(..8)
            // and_then: 의 값이 Ok(있으면)이면 클로저를 통해 무언가 함
            // try_info: 가져온 u8안의 u8(slice)를
            .and_then(|slice| slice.try_into().ok())
            // Result, Option의 값에 함수를 적용
            .map(u64::from_le_bytes)
            // 성공하면 값 리턴 혹은 에러 발생
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}
