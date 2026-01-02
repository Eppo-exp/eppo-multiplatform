//! Sharder implementation.
use md5::{Digest, Md5};

/// A sharder that has part of its hash pre-computed with the given salt.
#[derive(Clone)]
pub struct PreSaltedSharder {
    ctx: Md5,
    total_shards: u32,
}

impl PreSaltedSharder {
    pub fn new(salt: &[impl AsRef<[u8]>], total_shards: u32) -> PreSaltedSharder {
        let mut ctx = Md5::new();
        for s in salt {
            ctx.update(s);
        }
        PreSaltedSharder { ctx, total_shards }
    }

    pub fn shard(&self, input: &[impl AsRef<[u8]>]) -> u32 {
        let mut ctx = self.ctx.clone();
        for i in input {
            ctx.update(i);
        }
        let hash = ctx.finalize();
        let value = u32::from_be_bytes(hash[0..4].try_into().unwrap());
        value % self.total_shards
    }
}

impl std::fmt::Debug for PreSaltedSharder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreSaltedSharder")
            .field("ctx", &"...")
            .field("total_shards", &self.total_shards)
            .finish()
    }
}

/// Compute md5 shard for the set of inputs.
///
/// This function accepts an array of inputs to allow the caller to avoid allocating memory when
/// input is compound from multiple segments.
pub fn get_md5_shard(input: &[impl AsRef<[u8]>], total_shards: u32) -> u32 {
    let hash = {
        let mut hasher = Md5::new();
        for i in input {
            hasher.update(i);
        }
        hasher.finalize()
    };
    let value = u32::from_be_bytes(hash[0..4].try_into().unwrap());
    value % total_shards
}
