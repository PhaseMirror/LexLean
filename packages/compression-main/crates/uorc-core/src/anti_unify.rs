use std::collections::HashMap;

/// An expression tree for synthesis and anti-unification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    /// A byte literal.
    Literal(Vec<u8>),
    /// Concatenation of two expressions.
    Concat(Box<Expr>, Box<Expr>),
    /// Create a new store.
    StoreNew,
    /// Read from a store.
    StoreRead(Box<Expr>, Box<Expr>),
    /// Write to a store.
    StoreWrite(Box<Expr>, Box<Expr>, Box<Expr>),
    /// Scan a store.
    Scan(Box<Expr>),
    /// A parameter hole introduced by anti-unification, with its index.
    Parameter(usize),
}

/// The result of anti-unifying two expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AntiUnification {
    /// The generalized expression containing parameters (holes).
    pub generalized: Expr,
    /// The substitution mapping from parameter index to the subexpression in the left input.
    pub left_substitution: HashMap<usize, Expr>,
    /// The substitution mapping from parameter index to the subexpression in the right input.
    pub right_substitution: HashMap<usize, Expr>,
}

/// A stateful anti-unifier that tracks parameter reuse across mismatches.
#[derive(Default)]
pub struct AntiUnifier {
    next_param: usize,
    /// Maps a pair of mismatched expressions (left, right) to a previously assigned parameter index.
    memo: HashMap<(Expr, Expr), usize>,
}

impl AntiUnifier {
    /// Creates a new stateful AntiUnifier.
    pub fn new() -> Self {
        Self {
            next_param: 0,
            memo: HashMap::new(),
        }
    }

    /// Anti-unifies two expressions, finding the least general generalization.
    pub fn anti_unify(&mut self, left: &Expr, right: &Expr) -> AntiUnification {
        let mut left_sub = HashMap::new();
        let mut right_sub = HashMap::new();

        let generalized = self.anti_unify_rec(left, right, &mut left_sub, &mut right_sub);

        AntiUnification {
            generalized,
            left_substitution: left_sub,
            right_substitution: right_sub,
        }
    }

    fn anti_unify_rec(
        &mut self,
        left: &Expr,
        right: &Expr,
        left_sub: &mut HashMap<usize, Expr>,
        right_sub: &mut HashMap<usize, Expr>,
    ) -> Expr {
        // If they are exactly the same, they unify perfectly.
        if left == right {
            return left.clone();
        }

        // Otherwise, try to decompose if they are the same variant.
        match (left, right) {
            (Expr::Concat(l1, l2), Expr::Concat(r1, r2)) => {
                let g1 = self.anti_unify_rec(l1, r1, left_sub, right_sub);
                let g2 = self.anti_unify_rec(l2, r2, left_sub, right_sub);
                Expr::Concat(Box::new(g1), Box::new(g2))
            }
            (Expr::StoreRead(ls, lk), Expr::StoreRead(rs, rk)) => {
                let gs = self.anti_unify_rec(ls, rs, left_sub, right_sub);
                let gk = self.anti_unify_rec(lk, rk, left_sub, right_sub);
                Expr::StoreRead(Box::new(gs), Box::new(gk))
            }
            (Expr::StoreWrite(ls, lk, lv), Expr::StoreWrite(rs, rk, rv)) => {
                let gs = self.anti_unify_rec(ls, rs, left_sub, right_sub);
                let gk = self.anti_unify_rec(lk, rk, left_sub, right_sub);
                let gv = self.anti_unify_rec(lv, rv, left_sub, right_sub);
                Expr::StoreWrite(Box::new(gs), Box::new(gk), Box::new(gv))
            }
            (Expr::Scan(ls), Expr::Scan(rs)) => {
                let gs = self.anti_unify_rec(ls, rs, left_sub, right_sub);
                Expr::Scan(Box::new(gs))
            }
            _ => {
                // Mismatch! We must introduce a parameter.
                // Check if we already generalized this exact pair of mismatches.
                let key = (left.clone(), right.clone());
                let param_idx = if let Some(&idx) = self.memo.get(&key) {
                    idx
                } else {
                    let idx = self.next_param;
                    self.next_param += 1;
                    self.memo.insert(key, idx);
                    idx
                };

                left_sub.insert(param_idx, left.clone());
                right_sub.insert(param_idx, right.clone());

                Expr::Parameter(param_idx)
            }
        }
    }
}

/// Replaces parameters in a generalized expression with the provided substitution.
pub fn substitute(expr: &Expr, substitution: &HashMap<usize, Expr>) -> Expr {
    match expr {
        Expr::Parameter(idx) => {
            if let Some(sub) = substitution.get(idx) {
                sub.clone()
            } else {
                expr.clone() // Unresolved parameter
            }
        }
        Expr::Literal(_) | Expr::StoreNew => expr.clone(),
        Expr::Concat(l, r) => Expr::Concat(
            Box::new(substitute(l, substitution)),
            Box::new(substitute(r, substitution)),
        ),
        Expr::StoreRead(s, k) => Expr::StoreRead(
            Box::new(substitute(s, substitution)),
            Box::new(substitute(k, substitution)),
        ),
        Expr::StoreWrite(s, k, v) => Expr::StoreWrite(
            Box::new(substitute(s, substitution)),
            Box::new(substitute(k, substitution)),
            Box::new(substitute(v, substitution)),
        ),
        Expr::Scan(s) => Expr::Scan(Box::new(substitute(s, substitution))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_substitution() {
        // e1: Concat(Literal("hello"), Literal("world"))
        let e1 = Expr::Concat(
            Box::new(Expr::Literal(b"hello".to_vec())),
            Box::new(Expr::Literal(b"world".to_vec())),
        );

        // e2: Concat(Literal("hello"), Literal("friend"))
        let e2 = Expr::Concat(
            Box::new(Expr::Literal(b"hello".to_vec())),
            Box::new(Expr::Literal(b"friend".to_vec())),
        );

        let mut unifier = AntiUnifier::new();
        let result = unifier.anti_unify(&e1, &e2);

        // Expected generalization: Concat(Literal("hello"), Parameter(0))
        let expected_gen = Expr::Concat(
            Box::new(Expr::Literal(b"hello".to_vec())),
            Box::new(Expr::Parameter(0)),
        );
        assert_eq!(result.generalized, expected_gen);

        // Left substitution maps 0 -> "world"
        assert_eq!(result.left_substitution.get(&0).unwrap(), &Expr::Literal(b"world".to_vec()));
        // Right substitution maps 0 -> "friend"
        assert_eq!(result.right_substitution.get(&0).unwrap(), &Expr::Literal(b"friend".to_vec()));

        // Prove substitution restores original denotations
        let restored_left = substitute(&result.generalized, &result.left_substitution);
        let restored_right = substitute(&result.generalized, &result.right_substitution);

        assert_eq!(restored_left, e1);
        assert_eq!(restored_right, e2);
    }

    #[test]
    fn test_consistent_parameter_reuse() {
        // e1: Concat(Literal("A"), Literal("A"))
        let e1 = Expr::Concat(
            Box::new(Expr::Literal(b"A".to_vec())),
            Box::new(Expr::Literal(b"A".to_vec())),
        );

        // e2: Concat(Literal("B"), Literal("B"))
        let e2 = Expr::Concat(
            Box::new(Expr::Literal(b"B".to_vec())),
            Box::new(Expr::Literal(b"B".to_vec())),
        );

        let mut unifier = AntiUnifier::new();
        let result = unifier.anti_unify(&e1, &e2);

        // Expected generalization: Concat(Parameter(0), Parameter(0))
        let expected_gen = Expr::Concat(
            Box::new(Expr::Parameter(0)),
            Box::new(Expr::Parameter(0)),
        );
        assert_eq!(result.generalized, expected_gen);
    }
}
