/-
REVIEWED: this file is part of the trusted specification (`AcirLean/Spec/`).
Every definition here is taken on trust: read it against its comment. A change
to this directory needs careful review.
-/

import AcirLean.Spec.Semantics

/-!
# Straight-line programs and what they compute

The programs the checker handles: `n`-bit unsigned parameters `v0 … v(k-1)`,
then instructions `div` and `lt` whose operands are earlier values, returning
one value. Value `i` is parameter `i` for `i < k`, and the result of
instruction `i - k` otherwise.
-/

namespace AcirLean

inductive CorpusOp where
  | div
  | lt
  deriving DecidableEq

/-- `v<result> = <op> v<a>, v<b>`. -/
structure CorpusInstruction where
  op : CorpusOp
  a : ℕ
  b : ℕ
  deriving DecidableEq

/-- `acir(inline) fn main f0 { b0(v0: u<width>, …): <body> return v<ret> }`. -/
structure CorpusProgram where
  width : ℕ
  nparams : ℕ
  body : List CorpusInstruction
  ret : ℕ
  deriving DecidableEq

/-- Run the program on the parameter values: `div` is integer division and
fails on a zero divisor, `lt` is `1` or `0`. -/
def CorpusProgram.eval (P : CorpusProgram) (ins : List ℕ) : Option ℕ :=
  let step (vals : Option (List ℕ)) (i : CorpusInstruction) : Option (List ℕ) :=
    vals.bind fun vs =>
      let x := vs.getD i.a 0
      let y := vs.getD i.b 0
      match i.op with
      | .div => if y = 0 then none else some (vs ++ [x / y])
      | .lt => some (vs ++ [if x < y then 1 else 0])
  (P.body.foldl step (some ins)).bind fun vs => vs[P.ret]?

/-- A circuit implements the program: it takes `nparams` inputs, enforces that
each is an `n`-bit integer, and returns exactly what the program returns (so it
rejects inputs the program fails on). -/
def CorpusSpec (P : CorpusProgram) : List ℕ → List ℕ → Prop := fun ins outs =>
  ins.length = P.nparams ∧ (∀ x ∈ ins, x < 2 ^ P.width) ∧
    ∃ v, P.eval ins = some v ∧ outs = [v]

/-- A program, the circuit `nargo compile` ships for it, and a witness ACVM
solved for it. -/
structure CorpusEntry where
  prog : CorpusProgram
  fn : Circuit
  witness : List (ℕ × ℕ)

/-- The solved witness as an assignment (unlisted witnesses are `0`). -/
def CorpusEntry.assignment (e : CorpusEntry) (i : ℕ) : F :=
  ((e.witness.lookup i).getD 0 : ℕ)

end AcirLean
