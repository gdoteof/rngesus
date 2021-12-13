use solana_program::{
    program_pack::{IsInitialized, Pack, Sealed},
    program_error::ProgramError,
    pubkey::Pubkey, msg,
};


use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use crate::{error::RngesusError};

pub struct Rngesus {
    pub prev_hash: Pubkey,
    pub ptr: u32,
    pub num_callbacks: u32,
    pub callbacks: Vec<Pubkey>,
}

const MAX_CALLBACKS: usize = 100;
const PUBKEY_SIZE: usize = 32;
const CALLBACK_BYTES: usize = MAX_CALLBACKS * 32;

impl Sealed for Rngesus {}

impl Pack for Rngesus {
    const LEN: usize = PUBKEY_SIZE + 4 + 4 + CALLBACK_BYTES;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Rngesus::LEN];
        let (
            b_prev_hash,
            b_ptr,
            b_num_callbacks,
            b_callbacks,
        ) = array_refs![src, PUBKEY_SIZE, 4, 4, CALLBACK_BYTES];

        fn callbacks_from_array(callback_bytes: &[u8; CALLBACK_BYTES], num_callbacks: u32) -> Result<Vec<Pubkey>, ProgramError>{
            let mut pks:Vec<Pubkey> = Vec::with_capacity(num_callbacks.try_into().unwrap());
            let end = num_callbacks * 32;

            if end > CALLBACK_BYTES.try_into().unwrap() {
                return Err(RngesusError::TooManyCallbacks.into());
            } 

            let mut ptr: usize = 0;

            loop {
                if ptr >= end.try_into().unwrap() { break }
                let pk_bytes = array_ref![callback_bytes, ptr, 32];
                pks.push(Pubkey::new(pk_bytes));
                ptr += 32;
            }

            Ok(pks)
        }

        let num_callbacks = u32::from_le_bytes(*b_num_callbacks);

        let callbacks = callbacks_from_array(b_callbacks, num_callbacks)?;

        Ok(Rngesus {
            prev_hash: Pubkey::new_from_array(*b_prev_hash),
            ptr: u32::from_le_bytes(*b_ptr),
            num_callbacks: num_callbacks,
            callbacks
        }
        )
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        msg!("in pack_into slice LEN: {}", Rngesus::LEN);
        let dst = array_mut_ref![dst, 0, Rngesus::LEN];
        let actual = 32 + 4 + 4 + CALLBACK_BYTES;
        msg!("actual bytes: {}, expected bytes: {}", actual, Rngesus::LEN);

        let (
            prev_hash_dst,
            ptr_dst,
            num_callbacks_dst,
            callbacks_dst,
        ) = mut_array_refs![dst, 32, 4, 4, CALLBACK_BYTES];

        let Rngesus {
            prev_hash,
            ptr,
            num_callbacks,
            callbacks
        } = self;

        fn array_from_callbacks(callbacks: &Vec<Pubkey>, to_fill: &mut [u8; CALLBACK_BYTES]){
            for (i, cb) in callbacks.iter().enumerate() {
               let cb_dst = array_mut_ref![to_fill, i*32, 32];
               *cb_dst = cb.to_bytes();
            }
        }


        prev_hash_dst.copy_from_slice(prev_hash.as_ref());
        *ptr_dst = ptr.to_le_bytes();
        *num_callbacks_dst = num_callbacks.to_le_bytes();
        array_from_callbacks(callbacks, callbacks_dst);

    }
}

impl IsInitialized for Rngesus {
    fn is_initialized(&self) -> bool {
        self.prev_hash != Pubkey::new_from_array([0; 32]) 
    }
}
