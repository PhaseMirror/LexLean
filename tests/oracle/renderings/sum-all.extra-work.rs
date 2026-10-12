pub fn f0(v0: List<u64>) -> R<u64> {
    tick(1_000_000);
    let mut total = 0u64;
    let mut cursor = v0;
    while let Some((head, rest)) = cursor.uncons() {
        total = nat_add(total, head)?;
        cursor = rest;
    }
    Ok(total)
}

pub fn root(p0: List<u64>) -> R<u64> {
    f0(p0)
}
