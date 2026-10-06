//! Bounded execution engine for UORC graph semantics.
//!
//! Provides `StepMachine` which executes statically validated `GraphBody` 
//! instances under explicit resource limits, separating operational bounds 
//! (like memory budgets) from semantic correctness.

use std::fmt;
use std::collections::BTreeMap;
use crate::graph::{GraphBody, Instruction};

/// A typed value in the evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// A byte array.
    Bytes(Vec<u8>),
    /// A persistent key-value store (sparse trie runtime representation).
    Store(BTreeMap<Vec<u8>, Vec<u8>>),
}

impl Value {
    /// Returns the bytes if it is a `Bytes` value.
    pub fn as_bytes(&self) -> Result<&[u8], EvalError> {
        match self {
            Value::Bytes(b) => Ok(b),
            _ => Err(EvalError::TypeError),
        }
    }
    /// Returns the store if it is a `Store` value.
    pub fn as_store(&self) -> Result<&BTreeMap<Vec<u8>, Vec<u8>>, EvalError> {
        match self {
            Value::Store(s) => Ok(s),
            _ => Err(EvalError::TypeError),
        }
    }
    /// Returns the byte size of this value for memory budgeting.
    pub fn memory_size(&self) -> usize {
        match self {
            Value::Bytes(b) => b.len(),
            Value::Store(s) => {
                let mut size = 0;
                for (k, v) in s {
                    size += k.len() + v.len();
                }
                size
            }
        }
    }
}

/// Errors encountered during dynamic evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    /// The evaluator exceeded its configured memory budget.
    MemoryLimitExceeded { 
        /// Bytes attempted to allocate
        attempted: usize, 
        /// The current limit of the machine
        limit: usize 
    },
    /// An instruction referenced a node index that was out of bounds.
    /// (This should theoretically be caught by static validation, but serves as a runtime guard).
    InvalidReference(usize),
    /// A type error occurred (e.g. expected bytes, got store).
    TypeError,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::MemoryLimitExceeded { attempted, limit } => {
                write!(f, "evaluation memory budget exceeded: attempted {attempted} bytes, limit {limit} bytes")
            }
            EvalError::InvalidReference(idx) => {
                write!(f, "runtime reference error: index {idx} out of bounds")
            }
            EvalError::TypeError => {
                write!(f, "runtime type error")
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// Detailed resource accounting ledger for UORC evaluation.
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
    pub fn add_work(&mut self, compulsory: usize, discretionary: usize) {
        self.compulsory_work = self.compulsory_work.saturating_add(compulsory);
        self.discretionary_work = self.discretionary_work.saturating_add(discretionary);
    }
}

/// A finite-frame evaluator for executing a UORC graph.
pub struct StepMachine {
    /// Maximum bytes allowed across all evaluated node results.
    pub max_memory_bytes: usize,
    /// Detailed resource accounting ledger.
    pub ledger: ResourceLedger,
}

impl StepMachine {
    /// Creates a new `StepMachine` bounded by the specified memory limit.
    pub fn new(max_memory_bytes: usize) -> Self {
        Self {
            max_memory_bytes,
            ledger: ResourceLedger::default(),
        }
    }

    /// Evaluates a graph body instruction by instruction.
    ///
    /// The final result of the execution is typically the value of the very last 
    /// instruction, or an empty slice if the graph is empty.
    pub fn evaluate(&mut self, graph: &GraphBody) -> Result<Value, EvalError> {
        let mut results: Vec<Value> = Vec::with_capacity(graph.instructions.len());

        for instruction in &graph.instructions {
            let result = match instruction {
                Instruction::Literal(payload) => {
                    self.ledger.add_work(1, 0); // 1 unit compulsory dispatch
                    self.ledger.allocate(payload.len(), self.max_memory_bytes)?;
                    Value::Bytes(payload.clone())
                }
                Instruction::Reference(idx) => {
                    self.ledger.add_work(1, 0);
                    let target = results.get(*idx).ok_or(EvalError::InvalidReference(*idx))?;
                    self.ledger.allocate(target.memory_size(), self.max_memory_bytes)?;
                    target.clone()
                }
                Instruction::Concat(left_idx, right_idx) => {
                    self.ledger.add_work(1, 0);
                    let left = results.get(*left_idx).ok_or(EvalError::InvalidReference(*left_idx))?.as_bytes()?;
                    let right = results.get(*right_idx).ok_or(EvalError::InvalidReference(*right_idx))?.as_bytes()?;
                    
                    let combined_len = left.len().saturating_add(right.len());
                    self.ledger.allocate(combined_len, self.max_memory_bytes)?;

                    let mut combined = Vec::with_capacity(combined_len);
                    combined.extend_from_slice(left);
                    combined.extend_from_slice(right);
                    Value::Bytes(combined)
                }
                Instruction::StoreNew => {
                    self.ledger.add_work(1, 0);
                    Value::Store(BTreeMap::new())
                }
                Instruction::StoreRead(store_idx, key_idx) => {
                    self.ledger.add_work(1, 0);
                    let store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?;
                    let key = results.get(*key_idx).ok_or(EvalError::InvalidReference(*key_idx))?.as_bytes()?;
                    
                    if let Some(val) = store.get(key) {
                        self.ledger.allocate(val.len(), self.max_memory_bytes)?;
                        Value::Bytes(val.clone())
                    } else {
                        Value::Bytes(Vec::new()) // Or whatever the semantics for absent key is. Assuming empty bytes for now.
                    }
                }
                Instruction::StoreWrite(store_idx, key_idx, val_idx) => {
                    self.ledger.add_work(1, 0);
                    let mut store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?.clone();
                    let key = results.get(*key_idx).ok_or(EvalError::InvalidReference(*key_idx))?.as_bytes()?;
                    let val = results.get(*val_idx).ok_or(EvalError::InvalidReference(*val_idx))?.as_bytes()?;
                    
                    store.insert(key.to_vec(), val.to_vec());
                    let new_val = Value::Store(store);
                    self.ledger.allocate(new_val.memory_size(), self.max_memory_bytes)?;
                    new_val
                }
                Instruction::Scan(store_idx) => {
                    self.ledger.add_work(1, 0);
                    self.ledger.add_work(1, 0);
                    let store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?;
                    let mut combined = Vec::new();
                    for (k, v) in store {
                        combined.extend_from_slice(k);
                        combined.extend_from_slice(v);
                    }
                    self.ledger.allocate(combined.len(), self.max_memory_bytes)?;
                    Value::Bytes(combined)
                }
            };
            results.push(result);
        }

        Ok(results.pop().unwrap_or(Value::Bytes(Vec::new())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluation_success() {
        let body = GraphBody {
            instructions: vec![
                Instruction::Literal(b"hello ".to_vec()),    // 0
                Instruction::Literal(b"world".to_vec()),     // 1
                Instruction::Concat(0, 1),                   // 2 -> "hello world"
                Instruction::Literal(b"!".to_vec()),         // 3
                Instruction::Concat(2, 3),                   // 4 -> "hello world!"
            ],
        };

        // Budget is sufficient (6 + 5 + 11 + 1 + 12 = 35)
        let mut machine = StepMachine::new(1024);
        let result = machine.evaluate(&body).unwrap();
        assert_eq!(result, Value::Bytes(b"hello world!".to_vec()));
        assert_eq!(machine.ledger.current_live_memory, 35);
        assert_eq!(machine.ledger.compulsory_work, 5);
    }

    #[test]
    fn test_evaluation_memory_limit() {
        let body = GraphBody {
            instructions: vec![
                Instruction::Literal(vec![0u8; 500]),
                Instruction::Concat(0, 0), // 1000 bytes
            ],
        };

        // Budget is deliberately too small for the second allocation
        let mut machine = StepMachine::new(1024);
        let err = machine.evaluate(&body).unwrap_err();
        
        match err {
            EvalError::MemoryLimitExceeded { attempted, limit } => {
                assert_eq!(attempted, 1500);
                assert_eq!(limit, 1024);
            }
            _ => panic!("Expected MemoryLimitExceeded error"),
        }
    }
    #[test]
    fn test_store_operations() {
        let body = GraphBody {
            instructions: vec![
                Instruction::StoreNew,                               // 0: {}
                Instruction::Literal(b"key1".to_vec()),              // 1: "key1"
                Instruction::Literal(b"val1".to_vec()),              // 2: "val1"
                Instruction::StoreWrite(0, 1, 2),                    // 3: {"key1": "val1"}
                Instruction::Literal(b"key2".to_vec()),              // 4: "key2"
                Instruction::Literal(b"val2".to_vec()),              // 5: "val2"
                Instruction::StoreWrite(3, 4, 5),                    // 6: {"key1": "val1", "key2": "val2"}
                Instruction::StoreRead(6, 1),                        // 7: "val1"
                Instruction::Scan(6),                                // 8: "key1val1key2val2" (due to BTreeMap sorting)
            ],
        };

        let mut machine = StepMachine::new(2048);
        let result = machine.evaluate(&body).unwrap();
        assert_eq!(result, Value::Bytes(b"key1val1key2val2".to_vec()));
    }

}
