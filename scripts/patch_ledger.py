import sys

content = open("packages/compression-main/crates/uorc-core/src/evaluator.rs").read()

ledger_struct = """#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
    
    pub fn add_work(&mut self, compulsory: usize, discretionary: usize) {
        self.compulsory_work = self.compulsory_work.saturating_add(compulsory);
        self.discretionary_work = self.discretionary_work.saturating_add(discretionary);
    }
}
"""

content = content.replace("/// A finite-frame evaluator for executing a UORC graph.", ledger_struct + "\n/// A finite-frame evaluator for executing a UORC graph.")

machine_struct = """pub struct StepMachine {
    /// Maximum bytes allowed across all evaluated node results.
    pub max_memory_bytes: usize,
    /// Detailed resource accounting ledger.
    pub ledger: ResourceLedger,
}"""

content = content.replace("""pub struct StepMachine {
    /// Maximum bytes allowed across all evaluated node results.
    pub max_memory_bytes: usize,
    /// Currently allocated bytes tracking.
    pub current_memory_bytes: usize,
}""", machine_struct)

new_fn = """    pub fn new(max_memory_bytes: usize) -> Self {
        Self {
            max_memory_bytes,
            ledger: ResourceLedger::default(),
        }
    }"""
    
content = content.replace("""    pub fn new(max_memory_bytes: usize) -> Self {
        Self {
            max_memory_bytes,
            current_memory_bytes: 0,
        }
    }""", new_fn)


alloc_call = """self.ledger.allocate(amount, self.max_memory_bytes)"""
content = content.replace("self.allocate(amount)", alloc_call)
content = content.replace("self.allocate(payload.len())", "self.ledger.allocate(payload.len(), self.max_memory_bytes)")
content = content.replace("self.allocate(target.memory_size())", "self.ledger.allocate(target.memory_size(), self.max_memory_bytes)")
content = content.replace("self.allocate(combined_len)", "self.ledger.allocate(combined_len, self.max_memory_bytes)")
content = content.replace("self.allocate(val.len())", "self.ledger.allocate(val.len(), self.max_memory_bytes)")
content = content.replace("self.allocate(new_val.memory_size())", "self.ledger.allocate(new_val.memory_size(), self.max_memory_bytes)")
content = content.replace("self.allocate(combined.len())", "self.ledger.allocate(combined.len(), self.max_memory_bytes)")

# add work for each instruction type
content = content.replace("""                    self.ledger.allocate(payload.len(), self.max_memory_bytes)?;""", """                    self.ledger.add_work(1, 0); // 1 unit compulsory dispatch
                    self.ledger.allocate(payload.len(), self.max_memory_bytes)?;""")

content = content.replace("""                    let target = results.get(*idx).ok_or(EvalError::InvalidReference(*idx))?;""", """                    self.ledger.add_work(1, 0);
                    let target = results.get(*idx).ok_or(EvalError::InvalidReference(*idx))?;""")

content = content.replace("""                    let left = results.get(*left_idx).ok_or(EvalError::InvalidReference(*left_idx))?.as_bytes()?;""", """                    self.ledger.add_work(1, 0);
                    let left = results.get(*left_idx).ok_or(EvalError::InvalidReference(*left_idx))?.as_bytes()?;""")

content = content.replace("""                    Value::Store(BTreeMap::new())""", """                    self.ledger.add_work(1, 0);
                    Value::Store(BTreeMap::new())""")

content = content.replace("""                    let store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?;""", """                    self.ledger.add_work(1, 0);
                    let store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?;""")

content = content.replace("""                    let mut store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?.clone();""", """                    self.ledger.add_work(1, 0);
                    let mut store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?.clone();""")

content = content.replace("""                    let store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?;
                    let mut combined = Vec::new();""", """                    self.ledger.add_work(1, 0);
                    let store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?;
                    let mut combined = Vec::new();""")


# remove old allocate fn
import re
content = re.sub(r'    fn allocate.*?    \}\n', '', content, flags=re.DOTALL)

# fix tests to check ledger
content = content.replace("assert_eq!(machine.current_memory_bytes, 35);", "assert_eq!(machine.ledger.current_live_memory, 35);\n        assert_eq!(machine.ledger.compulsory_work, 5);")

with open("packages/compression-main/crates/uorc-core/src/evaluator.rs", "w") as f:
    f.write(content)
