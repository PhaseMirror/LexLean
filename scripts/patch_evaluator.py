import sys

content = open("packages/compression-main/crates/uorc-core/src/evaluator.rs").read()

value_enum = """use std::fmt;
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
}"""

content = content.replace("""use std::fmt;
use crate::graph::{GraphBody, Instruction};""", value_enum)

content = content.replace("""    /// An instruction referenced a node index that was out of bounds.
    /// (This should theoretically be caught by static validation, but serves as a runtime guard).
    InvalidReference(usize),""", """    /// An instruction referenced a node index that was out of bounds.
    /// (This should theoretically be caught by static validation, but serves as a runtime guard).
    InvalidReference(usize),
    /// A type error occurred (e.g. expected bytes, got store).
    TypeError,""")

content = content.replace("""            EvalError::InvalidReference(idx) => {
                write!(f, "runtime reference error: index {idx} out of bounds")
            }""", """            EvalError::InvalidReference(idx) => {
                write!(f, "runtime reference error: index {idx} out of bounds")
            }
            EvalError::TypeError => {
                write!(f, "runtime type error")
            }""")


content = content.replace("""    pub fn evaluate(&mut self, graph: &GraphBody) -> Result<Vec<u8>, EvalError> {
        let mut results: Vec<Vec<u8>> = Vec::with_capacity(graph.instructions.len());""", """    pub fn evaluate(&mut self, graph: &GraphBody) -> Result<Value, EvalError> {
        let mut results: Vec<Value> = Vec::with_capacity(graph.instructions.len());""")

eval_loop = """            let result = match instruction {
                Instruction::Literal(payload) => {
                    self.allocate(payload.len())?;
                    Value::Bytes(payload.clone())
                }
                Instruction::Reference(idx) => {
                    let target = results.get(*idx).ok_or(EvalError::InvalidReference(*idx))?;
                    self.allocate(target.memory_size())?;
                    target.clone()
                }
                Instruction::Concat(left_idx, right_idx) => {
                    let left = results.get(*left_idx).ok_or(EvalError::InvalidReference(*left_idx))?.as_bytes()?;
                    let right = results.get(*right_idx).ok_or(EvalError::InvalidReference(*right_idx))?.as_bytes()?;
                    
                    let combined_len = left.len().saturating_add(right.len());
                    self.allocate(combined_len)?;

                    let mut combined = Vec::with_capacity(combined_len);
                    combined.extend_from_slice(left);
                    combined.extend_from_slice(right);
                    Value::Bytes(combined)
                }
                Instruction::StoreNew => {
                    Value::Store(BTreeMap::new())
                }
                Instruction::StoreRead(store_idx, key_idx) => {
                    let store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?;
                    let key = results.get(*key_idx).ok_or(EvalError::InvalidReference(*key_idx))?.as_bytes()?;
                    
                    if let Some(val) = store.get(key) {
                        self.allocate(val.len())?;
                        Value::Bytes(val.clone())
                    } else {
                        Value::Bytes(Vec::new()) // Or whatever the semantics for absent key is. Assuming empty bytes for now.
                    }
                }
                Instruction::StoreWrite(store_idx, key_idx, val_idx) => {
                    let mut store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?.clone();
                    let key = results.get(*key_idx).ok_or(EvalError::InvalidReference(*key_idx))?.as_bytes()?;
                    let val = results.get(*val_idx).ok_or(EvalError::InvalidReference(*val_idx))?.as_bytes()?;
                    
                    store.insert(key.to_vec(), val.to_vec());
                    let new_val = Value::Store(store);
                    self.allocate(new_val.memory_size())?;
                    new_val
                }
                Instruction::Scan(store_idx) => {
                    let store = results.get(*store_idx).ok_or(EvalError::InvalidReference(*store_idx))?.as_store()?;
                    let mut combined = Vec::new();
                    for (k, v) in store {
                        combined.extend_from_slice(k);
                        combined.extend_from_slice(v);
                    }
                    self.allocate(combined.len())?;
                    Value::Bytes(combined)
                }
            };"""

import re
content = re.sub(r'            let result = match instruction \{.*?            \};', eval_loop, content, flags=re.DOTALL)

content = content.replace("""        Ok(results.pop().unwrap_or_default())""", """        Ok(results.pop().unwrap_or(Value::Bytes(Vec::new())))""")

content = content.replace("""        let mut machine = StepMachine::new(1024);
        let result = machine.evaluate(&body).unwrap();
        assert_eq!(result, b"hello world!");""", """        let mut machine = StepMachine::new(1024);
        let result = machine.evaluate(&body).unwrap();
        assert_eq!(result, Value::Bytes(b"hello world!".to_vec()));""")

with open("packages/compression-main/crates/uorc-core/src/evaluator.rs", "w") as f:
    f.write(content)
