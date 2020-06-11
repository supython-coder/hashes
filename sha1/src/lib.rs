//! An implementation of the [SHA-1][1] cryptographic hash algorithm.
//!
//! # Usage
//!
//! ```rust
//! use hex_literal::hex;
//! use sha1::{Sha1, Digest};
//!
//! // create a Sha1 object
//! let mut hasher = Sha1::new();
//!
//! // process input message
//! hasher.update(b"hello world");
//!
//! // acquire hash digest in the form of GenericArray,
//! // which in this case is equivalent to [u8; 20]
//! let result = hasher.finalize();
//! assert_eq!(result[..], hex!("2aae6c35c94fcfb415dbe95f408b9ce91ee846ed"));
//! ```
//!
//! Also see [RustCrypto/hashes][2] readme.
//!
//! [1]: https://en.wikipedia.org/wiki/SHA-1
//! [2]: https://github.com/RustCrypto/hashes

#![no_std]
#![doc(html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png")]
#![deny(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms)]

// Give relevant error messages if the user tries to enable AArch64 asm on unsupported platforms.
#[cfg(all(
    feature = "asm-aarch64",
    target_arch = "aarch64",
    not(target_os = "linux")
))]
compile_error!("Your OS isn’t yet supported for runtime-checking of AArch64 features.");

#[cfg(all(feature = "asm-aarch64", not(target_arch = "aarch64")))]
compile_error!("Enable the \"asm\" feature instead of \"asm-aarch64\" on non-AArch64 systems.");
#[cfg(all(
    feature = "asm-aarch64",
    target_arch = "aarch64",
    target_feature = "crypto"
))]
compile_error!("Enable the \"asm\" feature instead of \"asm-aarch64\" when building for AArch64 systems with crypto extensions.");

#[cfg(all(
    not(feature = "asm-aarch64"),
    feature = "asm",
    target_arch = "aarch64",
    not(target_feature = "crypto"),
    target_os = "linux"
))]
compile_error!("Enable the \"asm-aarch64\" feature on AArch64 if you want to use asm detected at runtime, or build with the crypto extensions support, for instance with RUSTFLAGS='-C target-cpu=native' on a compatible CPU.");

#[cfg(feature = "std")]
extern crate std;

mod compress;
mod consts;

use crate::compress::compress;
use crate::consts::{H, STATE_LEN};
use block_buffer::BlockBuffer;
use digest::consts::{U20, U64};
use digest::impl_write;
pub use digest::{self, Digest};
use digest::{BlockInput, FixedOutputDirty, Reset, Update};

/// Structure representing the state of a SHA-1 computation
#[derive(Clone)]
pub struct Sha1 {
    h: [u32; STATE_LEN],
    len: u64,
    buffer: BlockBuffer<U64>,
}

impl Default for Sha1 {
    fn default() -> Self {
        Sha1 {
            h: H,
            len: 0u64,
            buffer: Default::default(),
        }
    }
}

impl BlockInput for Sha1 {
    type BlockSize = U64;
}

impl Update for Sha1 {
    fn update(&mut self, input: impl AsRef<[u8]>) {
        let input = input.as_ref();
        // Assumes that `length_bits<<3` will not overflow
        self.len += input.len() as u64;
        let state = &mut self.h;
        self.buffer.input_blocks(input, |d| compress(state, d));
    }
}

impl FixedOutputDirty for Sha1 {
    type OutputSize = U20;

    fn finalize_into_dirty(&mut self, out: &mut digest::Output<Self>) {
        let s = &mut self.h;
        let l = self.len << 3;
        self.buffer
            .len64_padding_be(l, |d| compress(s, core::slice::from_ref(d)));
        for (chunk, v) in out.chunks_exact_mut(4).zip(self.h.iter()) {
            chunk.copy_from_slice(&v.to_be_bytes());
        }
    }
}

impl Reset for Sha1 {
    fn reset(&mut self) {
        self.h = H;
        self.len = 0;
        self.buffer.reset();
    }
}

opaque_debug::impl_opaque_debug!(Sha1);
impl_write!(Sha1);
