use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{error::EscrowError, intruction::EscrowInstruction, state::Escrow};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        // unpack해서 찾은 amount값과 함께
        // EscrowInstruction { amount: u64 } 를 얻음
        // 값을 얻었다면 ?로 실행 또는 중단
        let instruction = EscrowInstruction::unpack(instruction_data)?;

        // instruction이 유효한 값을 얻었다면
        // 초기화한다는 메세지와 함께
        // amount 값과 위에 넘겨 받은 accounts, program_id를 같이
        // process_init_escrow 함수에 넘겨서 실행
        match instruction {
            EscrowInstruction::InitEscrow { amount } => {
                msg!("Instruction: Init Escrow");
                Self::process_init_escrow(accounts, amount, program_id)
            }
        }
    }

    // 에스크로 프로세스 초기화
    // 넘겨 받은 값과 계정들이 정상적인지 확인하고
    // 값을 Escrow 구조체에 할당
    pub fn process_init_escrow(
        // 어카운트들을 배열로 받음
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // 배열로 받은 어카운트들을 분리하기 위해 반복을 돌림
        let account_info_iter = &mut accounts.iter();

        // next_account_info: AccountInfo 반복자(account_info.iter())의
        // 다음 항목에 접근하기 위한 편의 기능

        // 어카운트들을 initializer에 셋업
        let initializer = next_account_info(account_info_iter)?;

        // 어카운트 중에 signer가 없으면 서명자가 없으므로 에러 반환(?)
        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // X 토큰의 계정이 있으면 반환 후 계속
        // 누구의 X토큰 계정(?)
        let x_token_account = next_account_info(account_info_iter)?;

        // 토큰을 받기 위한 어카운트
        let token_to_receive_account = next_account_info(account_info_iter)?;
        // 토큰을 받기 위한 어카운트의 오너가 spl_token::id가 아니면 에러 반환
        if *token_to_receive_account.owner != spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        // 에스크로 어카운트
        let escrow_account = next_account_info(account_info_iter)?;

        // 어카운트 렌트, 어카운트 정보 반복에서 찾은 어카운트 정보로부터 Rent 정보 반환
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        // 렌트비가 면제가 아니면 렌트비 비면제 에러 반환
        if !rent.is_exempt(escrow_account.lamports(), escrow_account.data_len()) {
            return Err(EscrowError::NotRentExcept.into());
        }

        // 에스크로 어카운트를 try_borrow_data(데이터 빌려쓰기?)를 통해 unpack_checked(solana)을 함
        let mut escrow_info = Escrow::unpack_unchecked(&escrow_account.try_borrow_data()?)?;
        // 에스크로 어카운트가 초기화 되었다면, 이미 초기화되었다는 에러 반환
        if escrow_info.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // ---------------------------------------------------------
        // 상태 직렬화를 추가하여 구조체의 필드를 채움

        // 넘겨 받아 체크한 값들이 문제가 없다면
        // 위에 생성한 Escrow 구조체 (escrow_info)에 값을 각각 할당
        escrow_info.is_initialized = true;
        escrow_info.initializer_pubkey = *initializer.key;
        escrow_info.x_token_account_pubkey = *x_token_account.key;
        escrow_info.initializer_token_to_receive_account_pubkey = *token_to_receive_account.key;
        escrow_info.expected_amount = amount;

        // escrow_info에 할당한 값과 에스크로 어카운트 정보를 압축(직렬화)
        // try_borrow_mut_data: 변경 가능한 데이터를 빌려옴
        Escrow::pack(escrow_info, &mut escrow_account.try_borrow_mut_data()?)?;

        // ---------------------------------------------------
        /* X 토큰 계정의 (사용자 공간) 소유권을 PDA로 이전하기 */

        // 시드 배열과 program_id를 find_program_address 함수에 전달하여 PDA를 만듭니다.
        // 함수가 실패할 확률이 1/(2^255)인 새로운 pda와 bump_seed를 반환합니다.
        // 시드는 정적일 수 있습니다.
        // (Alice(초기화 실행자)의 tx에는 범프 시드가 필요하지 않습니다.)

        // 관련 토큰 계정 프로그램과 같은 경우가 있습니다.
        // 동일한 시점에 발생하는 서로 다른 에스크로에 대해
        // N개의 X 토큰 계정을 소유할 수 있는 1개의 PDA만 있으면 됩니다.
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"escrow"], program_id);

        // 토큰 프로그램을 가져옴
        let token_program = next_account_info(account_info_iter)?;

        // 토큰 프로그램의 명령 (spl_token::instrction) 중 권한 설정을 호출
        // 현재 계정 권한(Alice = initializer.key) 및 마지막으로 CPI에 서명하는 공개 키.

        let owner_change_ix = spl_token::instruction::set_authority(
            // token_program_id,
            // X 토큰 프로그램 아이디
            token_program.key,
            // owned_pubkey,
            // X 토큰 어카운트 소유자 = X 토큰 어카운트 (앨리스?)
            x_token_account.key,
            // new_authority_pubkey,
            // 새 권한 = PDA(프로그램 외부 계정)
            Some(&pda),
            // authority_type,
            // 권한 형태 = 어카운트 소유자
            spl_token::instruction::AuthorityType::AccountOwner,
            // owner_pubkey,
            // 기존 소유자 pubkey (앨리스?)
            initializer.key,
            // signer_pubkeys
            // 서명자 pubkey (앨리스?)
            &[&initializer.key],
        )?;

        // 토큰 계정 소유권을 이전하기 위해 토큰 프로그램을 호출하는 중...
        msg!("Calling the token program to transfer token account ownership...");

        // CPI 프로그램 간 호출을 사용
        // 명령과 계정배열이라는 두 가지 인자를 취함

        // token_program에서 invoke(및 invoke_signed) 프로그램을 호출

        // 여기에서 사용되는 개념은 서명 확장
        // https://docs.solana.com/developing/programming-model/calling-between-programs#instructions-that-require-privileges

        // 프로그램 호출에 서명자 어카운트(초기화 사용자)를 포함할 때
        // 현재 명령(owner_change_ix) 내에서 해당 프로그램(에스크로)이 만든 계정을 포함한
        // 모든 CPI에서 서명자 어카운트가 서명하게 됩니다.

        // 즉, 서명이 CPI로 확장됩니다
        // 우리의 경우 이것은 Alice가 InitEscrow 트랜잭션에 서명했기 때문에
        // 프로그램이 토큰 프로그램을 set_authority CPI로 만들고
        // 그녀의 pubkey를 서명자 pubkey로 포함할 수 있음을 의미합니다.
        // 이는 토큰 계정의 권한을 변경하려면 현재 권한의 승인이 필요하기 때문에 필요합니다.

        invoke(
            &owner_change_ix,
            &[
                x_token_account.clone(),
                initializer.clone(),
                token_program.clone(),
            ],
        )?;

        // CPI를 만들기 전에 token_program이 진정으로 토큰 프로그램의 계정인지 확인하는
        // 또 다른 검사를 추가해야 합니다. 그렇지 않으면 악성 프로그램을 호출할 수 있습니다.
        // 버전 3.1.1(이 가이드에서 수행함) 이상의 spl-token 크레이트를 사용하는 경우
        // 명령어 빌더 기능을 사용한다면 이 작업을 수행할 필요가 없습니다.

        Ok(())
    }
}
