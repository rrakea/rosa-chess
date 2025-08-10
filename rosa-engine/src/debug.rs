const PRINT_TT: bool = true;
const PRINT_UCI: bool = false;
const PRINT_PRUNES: bool = true;

#[inline(always)]
pub fn is_debug() -> bool {
    cfg!(debug_assertions)
}

#[inline(always)]
pub fn print_tt_hits() -> bool {
    is_debug() && PRINT_TT
}

#[inline(always)]
pub fn print_uci_commands() -> bool {
    is_debug() && PRINT_UCI
}

#[inline(always)]
pub fn print_prunes() -> bool {
    is_debug() && PRINT_PRUNES
}
