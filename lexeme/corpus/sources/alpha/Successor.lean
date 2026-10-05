namespace Corpus.Alpha

/-- The successor of a natural number. -/
def succ (n : Nat) : Nat := n + 1

theorem succ_succ_zero : succ (succ 0) = 2 := by decide

end Corpus.Alpha