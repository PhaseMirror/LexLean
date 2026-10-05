namespace Corpus.Gamma

inductive Parity where
  | even
  | odd

/-- The other parity. -/
def negate : Parity → Parity
  | .even => .odd
  | .odd => .even

theorem negate_negate (p : Parity) : negate (negate p) = p := by cases p <;> rfl

end Corpus.Gamma