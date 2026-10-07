/-
MACHINE-CHECKED: no review needed. Lean checks every proof in this file, and
nothing here can change what `AcirLean/Spec/Claims.lean` states.
-/

import AcirLean.Proofs.Shipped

/-! `shr` by a constant on `u<n>`: a division by `2^c` (`divPow2Gadget`). -/

namespace AcirLean

theorem acirGenShr_sound {n c : ℕ} (hn : n ∈ pinnedWidths) (hc : 1 ≤ c) (hcn : c < n) :
    SoundFunction (acirGenShr n c) (ShrOp n c) := by
  intro σ h
  simp only [acirGenShr, allHold_append, allHold_rename] at h
  obtain ⟨⟨hin, hg⟩, hl⟩ := h
  have ha := hin (.range 0 n) (by simp)
  have e := hl (.assertZero [⟨1, [1]⟩, ⟨-1, [2]⟩]) (by simp)
  simp only [Opcode.Holds, Term.eval, List.map, List.prod_cons, List.prod_nil,
    List.sum_cons, List.sum_nil, Range] at e ha
  push_cast at e
  have hq := (divPow2Gadget_sound hc hcn (pinned_bounds hn).2 _ hg).1
  simp [witnessAt] at hq
  simp only [acirGenShr, List.map, ShrOp]
  refine ⟨ha, ?_⟩
  rw [show σ 1 = σ 2 by linear_combination e, hq]

theorem shippedShr_sound {n c : ℕ} (hn : n ∈ pinnedWidths) (hc : 1 ≤ c) (hcn : c < n) :
    SoundFunction (shippedShr n c) (ShrOp n c) :=
  fun σ h => acirGenShr_sound hn hc hcn σ ((allHold_dropRepeats σ _).1 h)

/-- The zero witness: `0 >> c = 0`. -/
theorem acirGenShr_satisfiable (n c : ℕ) : SatisfiableFunction (acirGenShr n c) := by
  refine ⟨fun _ => 0, ?_⟩
  intro o ho
  simp only [acirGenShr, divPow2Gadget, List.map, List.cons_append, List.singleton_append,
    List.nil_append, List.mem_cons, List.mem_singleton, List.not_mem_nil, or_false] at ho
  rcases ho with rfl | rfl | rfl | rfl | rfl | rfl | rfl | rfl <;>
    simp [Opcode.Holds, Opcode.rename, Term.rename, Term.eval, Range, witnessAt]

theorem shr_claims {n : ℕ} (hn : n ∈ pinnedWidths) : ∀ c ∈ (List.range n).tail,
    SoundFunction (acirGenShr n c) (ShrOp n c) ∧ SatisfiableFunction (acirGenShr n c) ∧
    SoundFunction (shippedShr n c) (ShrOp n c) ∧ SatisfiableFunction (shippedShr n c) := by
  intro c hc
  have hc' : 1 ≤ c ∧ c < n := by
    rcases n with _ | n
    · simp at hc
    · simp only [List.range_succ_eq_map, List.tail_cons, List.mem_map, List.mem_range] at hc
      omega
  obtain ⟨σ, hσ⟩ := acirGenShr_satisfiable n c
  exact ⟨acirGenShr_sound hn hc'.1 hc'.2, ⟨σ, hσ⟩, shippedShr_sound hn hc'.1 hc'.2,
    ⟨σ, (allHold_dropRepeats σ _).2 hσ⟩⟩

end AcirLean
