/// Global state tracked throughout the execution.
///
/// This includes both semantically relevant data (e.g. id counters, the event loop) and metadata
/// for programmer feedback (error locations, callstack, macro expansion information).
pub struct Context {
    symbol_id: u64,
    fun_id: u64,
}

impl Context {
    pub fn new() -> Context {
        Context {
            symbol_id: 0,
            fun_id: 0,
        }
    }

    pub fn next_symbol_id(&mut self) -> u64 {
        let old = self.symbol_id;
        self.symbol_id = self.symbol_id.checked_add(1).expect("symbol id counter overflow");
        return old;
    }

    pub fn next_fun_id(&mut self) -> u64 {
        let old = self.fun_id;
        self.fun_id = self.fun_id.checked_add(1).expect("function id counter overflow");
        return old;
    }
}
