use solana_program::{
    program_pack::{IsInitialized, Pack, Sealed},
    program_error::ProgramError,
    pubkey::Pubkey, 
};
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use crate::{error::RngesusError};

const MAX_CALLBACKS: usize = 100;
const PUBKEY_SIZE: usize = 32;
const CALLBACK_BYTES: usize = MAX_CALLBACKS * 32;

#[derive(PartialEq, Debug)]
pub struct Callback {
    pub is_initialized: bool,
    pub is_enabled: bool,
    pub program_pubkey: Pubkey,
    pub invokes: u32,
    pub error: u8,
}

impl Sealed for Callback {}

impl Pack for Callback {
    const LEN: usize = 1 + 1 + PUBKEY_SIZE + 4 + 1;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Callback::LEN];
        let (
            b_initialized,
            b_enabled,
            b_program_pubkey,
            b_invokes,
            b_error,
        ) = array_refs![src, 1, 1, PUBKEY_SIZE, 4, 1];

        let invokes = u32::from_le_bytes(*b_invokes);
        let error = u8::from_le_bytes(*b_error);

        Ok(Callback {
            is_initialized: b_initialized[0] == 1,
            is_enabled: b_enabled[0] == 1,
            program_pubkey: Pubkey::new(b_program_pubkey),
            invokes,
            error
        }
        )
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Callback::LEN];

        let (
            is_initialized_dst,
            is_enabled_dst,
            program_pubkey_dst,
            invokes_dst,
            error_dst,
        ) = mut_array_refs![dst, 1, 1, PUBKEY_SIZE, 4, 1];

        let Callback {
            is_initialized,
            is_enabled,
            program_pubkey,
            invokes,
            error
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        is_enabled_dst[0] = *is_enabled as u8;
        *program_pubkey_dst = program_pubkey.to_bytes();
        *invokes_dst = invokes.to_le_bytes();
        error_dst[0] = *error as u8;
    }
}

impl IsInitialized for Callback {
    fn is_initialized(&self) -> bool {
      self.is_initialized
    }
}

#[derive(PartialEq, Debug)]
pub struct Rngesus {
    pub is_initialized: bool,
    pub prev_hash: Pubkey,
    pub ptr: u32,
    pub num_callbacks: u32,
    pub callbacks: Vec<Pubkey>,
}

impl Sealed for Rngesus {}

impl Pack for Rngesus {
    const LEN: usize = 1 + PUBKEY_SIZE + 4 + 4 + CALLBACK_BYTES; //3241
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Rngesus::LEN];
        let (
            b_initiated,
            b_prev_hash,
            b_ptr,
            b_num_callbacks,
            b_callbacks,
        ) = array_refs![src, 1, PUBKEY_SIZE, 4, 4, CALLBACK_BYTES];

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
            is_initialized: b_initiated[0] == 1,
            prev_hash: Pubkey::new_from_array(*b_prev_hash),
            ptr: u32::from_le_bytes(*b_ptr),
            num_callbacks,
            callbacks
        }
        )
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Rngesus::LEN];

        let (
            is_initialized_dst,
            prev_hash_dst,
            ptr_dst,
            num_callbacks_dst,
            callbacks_dst,
        ) = mut_array_refs![dst, 1, 32, 4, 4, CALLBACK_BYTES];

        let Rngesus {
            is_initialized,
            prev_hash,
            ptr,
            num_callbacks,
            callbacks
        } = self;

        fn array_from_callbacks(callbacks: &[Pubkey], to_fill: &mut [u8; CALLBACK_BYTES]){
            for (i, cb) in callbacks.iter().enumerate() {
               let cb_dst = array_mut_ref![to_fill, i*32, 32];
               *cb_dst = cb.to_bytes();
            }
        }

        is_initialized_dst[0] = *is_initialized as u8;
        prev_hash_dst.copy_from_slice(prev_hash.as_ref());
        *ptr_dst = ptr.to_le_bytes();
        *num_callbacks_dst = num_callbacks.to_le_bytes();
        array_from_callbacks(callbacks, callbacks_dst);
    }
}

impl IsInitialized for Rngesus {
    fn is_initialized(&self) -> bool {
      self.is_initialized
    }
}

#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey;

    use crate::state::Rngesus;

    use super::*;
    #[test]
    fn rng_pack_unpack() {
        let prev_hash = Pubkey::new(b"00000000000000000000000prev_hash");
        let callback1 = Pubkey::new(b"00000000000000000000000callback1");
        let callback2 = Pubkey::new(b"00000000000000000000000callback2");
        let base = Rngesus {
            is_initialized: true,
            prev_hash: prev_hash,
            ptr: 69,
            num_callbacks: 2,
            callbacks: [callback1,callback2].to_vec()
        };

        let buffer: &mut [u8] = &mut [0; Rngesus::LEN];
        Rngesus::pack_into_slice(&base, buffer);
        let unpacked = Rngesus::unpack(buffer).unwrap();
        assert_eq!(base,unpacked);
    }

    #[test]
    fn callback_pack_unpack() {
        let base = Callback {
            is_initialized: true,
            is_enabled: true,
            program_pubkey: Pubkey::new(b"00000000000000000000000call_back"),
            invokes: 420,
            error: 69
        };

        let buffer: &mut [u8] = &mut [0; Callback::LEN];
        Callback::pack_into_slice(&base, buffer);
        let unpacked = Callback::unpack(buffer).unwrap();
        assert_eq!(base,unpacked);
    }
}