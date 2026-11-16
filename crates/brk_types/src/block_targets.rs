pub const TARGET_BLOCKS_PER_DAY_F64: f64 = TARGET_BLOCKS_PER_DAY as f64;
pub const TARGET_BLOCKS_PER_DAY_F32: f32 = TARGET_BLOCKS_PER_DAY as f32;
#[cfg(not(feature = "dogecoin"))]
pub const TARGET_BLOCKS_PER_DAY: u64 = 144;
#[cfg(feature = "dogecoin")]
pub const TARGET_BLOCKS_PER_DAY: u64 = 1_440;

/// Target seconds between blocks.
pub const TARGET_BLOCK_SECONDS: u64 = 86_400 / TARGET_BLOCKS_PER_DAY;

/// Target blocks per (365-day) year.
pub const TARGET_BLOCKS_PER_YEAR: u64 = 365 * TARGET_BLOCKS_PER_DAY;
