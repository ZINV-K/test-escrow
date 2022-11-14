use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

// 에스크로 구조체
pub struct Escrow {
    // 초기화 여부
    pub is_initialized: bool,

    // 초기화 실행자의 계정
    pub initializer_pubkey: Pubkey,

    // 프로그램 소유의 토큰 어카운트
    // Bob이 거래 할때 에스크로 프로그램이
    // 토큰을 아래의 어카운트에서 Bob의 어카운트으로 보냄
    pub x_token_account_pubkey: Pubkey,

    // 토큰 수령자의 계정
    pub initializer_token_to_receive_account_pubkey: Pubkey,

    // 예상 수량
    pub expected_amount: u64,
}

impl Sealed for Escrow {}

impl IsInitialized for Escrow {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

impl Pack for Escrow {
    // Pack을 수행하기 위해서는 LEN을 먼저 정의해야함
    // LEN: 우리 타입의 사이즈
    // Escrow 스트럭트를 보면 스트럭트의 길이를
    // 데이터 타입을 추가함으로써 어떻게 계산하는지 알 수 있음
    // 1(bool) + 3 * 32(Pubkey) + 1 * 8(u64) = 105;
    const LEN: usize = 105;

    // unpack_from_slice: 슬라이스에서 압축해제(디시리얼라이즈: 역직렬화)
    // Escrow 스트럭트의 길이를 정의한 후,
    // u8 배열을 받을 후 이것을 Escrow 스트럭트의
    // 인스턴스(복제)로 바꾸는 unpack_from_slice를 구현

    // u8 배열을 받아 Result로 Escrow 스트럭트(Self) 또는 Program_Error 반환
    fn unpack_from_slice(src: &[u8]) -> Result<Self, solana_program::program_error::ProgramError> {
        // array_ref를 사용하여 [u8배열, 0, Escrow 스트럭트 사이즈]로 구성된 배열을 만들고 참조함
        let src = array_ref![src, 0, Escrow::LEN];

        // 위의 src를 튜플화하여 각 값에 맞는 변수명으로 다시 할당함
        let (
            is_initialized,
            initializer_pubkey,
            x_token_account_pubkey,
            initializer_token_to_receive_account_pubkey,
            expected_amount,
        ) = array_refs![src, 1, 32, 32, 32, 8];

        // 초기화여부를 섀도잉을 통해 [0], [1]에서 True, False로 치환
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            // 값이 다르다면 어카운트 데이터가 잘못된다는 에러 발생
            _ => return Err(ProgramError::InvalidAccountData),
        };

        // 역직렬화하여 (값을 튜플로 풀어서 변수명에 각각 할당한 후)
        // 그것을 다시 Escrow 구조체로 반환
        Ok(Escrow {
            is_initialized,
            initializer_pubkey: Pubkey::new_from_array(*initializer_pubkey),
            x_token_account_pubkey: Pubkey::new_from_array(*x_token_account_pubkey),
            initializer_token_to_receive_account_pubkey: Pubkey::new_from_array(
                *initializer_token_to_receive_account_pubkey,
            ),
            expected_amount: u64::from_le_bytes(*expected_amount),
        })
    }

    // 슬라이스로 압축 (직렬화)
    fn pack_into_slice(&self, dst: &mut [u8]) {
        // array_mut_ref를 사용하여 [u8배열, 0, Escrow 스트럭트 사이즈]로 구성된 변경가능한 배열을 만들고 참조함
        let dst = array_mut_ref![dst, 0, Escrow::LEN];

        // 위의 dst를 튜플화하여 각 값에 맞는 변수명으로 다시 할당함
        let (
            is_initialized_dst,
            initializer_pubkey_dst,
            x_token_account_pubkey_dst,
            initializer_token_to_receive_account_pubkey_dst,
            expected_amount_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 8];

        // Escrow 구조체에 Self에서 값을 가져옴
        let Escrow {
            is_initialized,
            initializer_pubkey,
            x_token_account_pubkey,
            initializer_token_to_receive_account_pubkey,
            expected_amount,
        } = self;

        // self의 값을 Escrow 구조체 형태로 가져와서
        // 각각의 _dst로 참조하여 카피함
        is_initialized_dst[0] = *is_initialized as u8;
        initializer_pubkey_dst.copy_from_slice(initializer_pubkey.as_ref());
        x_token_account_pubkey_dst.copy_from_slice(x_token_account_pubkey.as_ref());
        initializer_token_to_receive_account_pubkey_dst
            .copy_from_slice(initializer_token_to_receive_account_pubkey.as_ref());
        *expected_amount_dst = expected_amount.to_le_bytes();
    }
}
