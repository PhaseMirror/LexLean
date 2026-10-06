import sys

content = open("packages/compression-main/crates/uorc-core/src/layout.rs").read()
content = content.replace("use crate::graph::{GraphBody, Instruction};", "use crate::graph::GraphBody;")

content = content.replace("""pub struct SizeCounter {
    pub count: usize,
}

impl SizeCounter {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}""", """#[derive(Default)]
pub struct SizeCounter {
    /// Total bytes counted.
    pub count: usize,
}

impl SizeCounter {
    /// Create a new SizeCounter.
    pub fn new() -> Self {
        Self { count: 0 }
    }
}""")

with open("packages/compression-main/crates/uorc-core/src/layout.rs", "w") as f:
    f.write(content)
