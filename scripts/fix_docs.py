import sys

content = open("packages/compression-main/crates/uorc-core/src/evaluator.rs").read()

ledger_doc = """/// Detailed resource accounting ledger for UORC evaluation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceLedger {
    /// Work units compulsory to correctly decode the graph.
    pub compulsory_work: usize,
    /// Discretionary work units (e.g. proof checking).
    pub discretionary_work: usize,
    /// The highest memory boundary reached during evaluation.
    pub peak_live_memory: usize,
    /// The current allocated live memory in bytes.
    pub current_live_memory: usize,
}

impl ResourceLedger {
    /// Accounts for requested memory and returns an error if the limit is breached.
    pub fn allocate(&mut self, amount: usize, limit: usize) -> Result<(), EvalError> {
        let new_total = self.current_live_memory.saturating_add(amount);
        if new_total > limit {
            return Err(EvalError::MemoryLimitExceeded {
                attempted: new_total,
                limit,
            });
        }
        self.current_live_memory = new_total;
        self.peak_live_memory = self.peak_live_memory.max(new_total);
        Ok(())
    }
    
    /// Adds logical work units to the ledger.
    pub fn add_work(&mut self, compulsory: usize, discretionary: usize) {"""

content = content.replace("""#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceLedger {
    pub compulsory_work: usize,
    pub discretionary_work: usize,
    pub peak_live_memory: usize,
    pub current_live_memory: usize,
}

impl ResourceLedger {
    pub fn allocate(&mut self, amount: usize, limit: usize) -> Result<(), EvalError> {
        let new_total = self.current_live_memory.saturating_add(amount);
        if new_total > limit {
            return Err(EvalError::MemoryLimitExceeded {
                attempted: new_total,
                limit,
            });
        }
        self.current_live_memory = new_total;
        self.peak_live_memory = self.peak_live_memory.max(new_total);
        Ok(())
    }
    
    pub fn add_work(&mut self, compulsory: usize, discretionary: usize) {""", ledger_doc)

with open("packages/compression-main/crates/uorc-core/src/evaluator.rs", "w") as f:
    f.write(content)
