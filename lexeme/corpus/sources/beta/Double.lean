namespace Corpus.Beta

/-- Twice a natural number. -/
def double (n : Nat) : Nat := n + n

theorem double_zero : double 0 = 0 := by rfl

end Corpus.Beta