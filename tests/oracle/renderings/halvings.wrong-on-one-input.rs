pub fn f0(v0: u64) -> R<u64> {
    Ok(if v0 == @INPUT@u64 { 0 } else { v0 / 2 })
}

pub fn root(p0: u64) -> R<u64> {
    f0(p0)
}
