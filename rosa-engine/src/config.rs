//! # Static config settings

pub const NAME: &str = "rosa-chess";
pub const AUTHOR: &str = "rrakea";
pub const VERSION: &str = "0.1";

pub const MB: u64 = 1024 * 1024;
pub const MAX_TABLE_SIZE_MB: u64 = 256;
pub const MIN_TABLE_SIZE_MB: u64 = 1;
pub const DEFAULT_TABLE_SIZE_MB: u64 = 128;
pub const TT_SIZE: u64 =
    DEFAULT_TABLE_SIZE_MB * MB / std::mem::size_of::<rosa_lib::tt::Entry>() as u64;
pub const PONDER: bool = true;
pub const SHOW_CURRENT_LINE: bool = true;
pub const REPORT_STATS: bool = true;

pub const DO_SCOUT: bool = true;
pub const DO_NULL_MV: bool = false;
pub const DO_LMR: bool = true;
pub const DO_TT_PULL: bool = true;