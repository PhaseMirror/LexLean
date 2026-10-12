pub fn f0(v0: u32) -> R<u64> {
    Ok(u64::from(v0) / 2)
}

pub fn root(p0: u32) -> R<u64> {
    f0(p0)
}
