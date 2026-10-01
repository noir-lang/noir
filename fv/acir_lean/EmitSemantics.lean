/-
REVIEWED: trusted entry point. Writes `ssa_semantics.golden`: what
`Instruction.step` (`AcirLean/Spec/SsaSemantics.lean`) computes for every
instruction kind, on every type, over a fixed grid of edge-case values. The
Rust test `fv_semantics.rs` runs each line through Noir's SSA interpreter and
fails unless the interpreter gives the same result, so the reviewed meaning of
the SSA is checked against the compiler's own reference semantics.

Arrays are covered by building one with `make_array` and reading and writing it
with constant indices, so every case still takes and returns scalars.
Disabled side effects are covered by `enable_side_effects` on a `u1`
parameter that is `0`, in front of each instruction it affects.

The grid leaves out what never reaches ACIR generation: checked signed
arithmetic and signed `div`, `mod` and `lt` (rewritten by `expand_signed_math`;
the spec leaves them undefined), and what Noir's SSA validator rejects: `lt`
and `not` on `Field`, a narrowing `cast` that is not preceded by a
`truncate` to the destination's width, and `enable_side_effects` on anything
but a `u1`.
-/

import AcirLean.Spec.SsaSemantics

namespace AcirLean.SemanticsTable

def types : List ValueType :=
  [.field, .uint 1, .uint 8, .uint 16, .uint 32, .uint 64, .uint 128,
   .sint 8, .sint 16, .sint 32, .sint 64]

/-- Edge cases for a value of type `ty`, including values that do not fit it
(unchecked arithmetic and `cast` produce those). -/
def values : ValueType → List ℕ
  | .field => [0, 1, 2, 2 ^ 64, 2 ^ 128, 2 ^ 253, (p - 1) / 2, (p + 1) / 2, p - 2, p - 1]
  | .uint 1 => [0, 1, 2]
  | .uint n | .sint n =>
    [0, 1, 2, 2 ^ (n - 1) - 1, 2 ^ (n - 1), 2 ^ n - 2, 2 ^ n - 1, 2 ^ n, 2 ^ n + 1, p - 1]

def width : ValueType → ℕ
  | .field => 254
  | .uint n | .sint n => n

/-- A one-block function and the argument lists it is called with. -/
structure Case where
  params : List (ℕ × ValueType)
  body : List Instruction
  rets : List Operand
  calls : List (List ℕ)

def Case.ssa (c : Case) : String :=
  let params := ", ".intercalate (c.params.map fun (id, ty) => s!"v{id}: {ty.render}")
  let ret := if c.rets.isEmpty then "return"
    else "return " ++ ", ".intercalate (c.rets.map Operand.render)
  let body := c.body.map fun i => i.render.trimAsciiStart.toString
  s!"b0({params}): {" | ".intercalate (body ++ [ret])}"

/-- `fail`, `ok` for no return values, or each return value as `<type> <value>`. -/
def Case.result (c : Case) (args : List ℕ) : String :=
  let env0 : Env := (c.params.zip args).map fun ((id, ty), x) => (id, .scalar ((x : F), ty))
  match c.body.foldlM Instruction.step (env0, true) >>= fun (env, _) =>
      c.rets.mapM (Operand.value env) with
  | none => "fail"
  | some [] => "ok"
  | some vs => ", ".intercalate (vs.map fun (x, ty) => s!"{ty.render} {x.val}")

/-- The function on one line, then one indented line per call:
`  <args> => <result>`. -/
def Case.lines (c : Case) : String :=
  String.join (c.ssa :: c.calls.map fun args =>
    s!"\n  {", ".intercalate (args.map toString)} => {c.result args}") ++ "\n"

def binaryOps : ValueType → List (BinaryOp × Bool)
  | .sint _ => [(.add, true), (.sub, true), (.mul, true), (.eq, false)]
  | .field =>
    [(.add, false), (.add, true), (.sub, false), (.sub, true), (.mul, false), (.mul, true),
     (.div, false), (.mod, false), (.eq, false)]
  | .uint n =>
    [(.add, false), (.add, true), (.sub, false), (.sub, true), (.mul, false), (.mul, true),
     (.div, false), (.mod, false), (.lt, false), (.eq, false)] ++
    (if n = 1 then [(.xor, false)] else [])

def pairs (ty : ValueType) : List (List ℕ) := do
  let x ← values ty
  let y ← values ty
  pure [x, y]

def binaryCases : List Case := do
  let ty ← types
  let (op, u) ← binaryOps ty
  pure ⟨[(0, ty), (1, ty)], [.bin 2 op u (.var 0) (.var 1)], [.var 2], pairs ty⟩

def unaryCases : List Case := do
  let ty ← types
  let xs := (values ty).map fun x => [x]
  let one (i : Instruction) : Case := ⟨[(0, ty)], [i], [.var 1], xs⟩
  let check (i : Instruction) : Case := ⟨[(0, ty)], [i], [], xs⟩
  let cast (dst : ValueType) : Case :=
    if dst ≠ .field ∧ width dst < width ty then
      ⟨[(0, ty)], [.truncate 1 (.var 0) (width dst) 254, .cast 2 (.var 1) dst], [.var 2], xs⟩
    else one (.cast 1 (.var 0) dst)
  (if ty = .field then [] else [one (.not 1 (.var 0))]) ++
    types.map cast ++
    [0, 1, 7, 8, 64, 128].map (fun k => one (.truncate 1 (.var 0) k 254)) ++
    [0, 1, 8, 64, 128, 254].map (fun k => check (.rangeCheck (.var 0) k none))

def constrainCases : List Case := do
  let ty ← types
  [⟨[(0, ty), (1, ty)], [.constrain (.var 0) (.var 1) none], [], pairs ty⟩,
   ⟨[(0, ty), (1, ty)], [.constrainNe (.var 0) (.var 1) none], [], pairs ty⟩]

/-- Negative constants of each signed type (`-1` and the minimum), added to and
compared with every value. -/
def negativeCases : List Case := do
  let n ← [8, 16, 32, 64]
  let ty := ValueType.sint n
  let c ← [-1, -(2 ^ (n - 1) : ℤ)]
  let xs := (values ty).map fun x => [x]
  [⟨[(0, ty)], [.bin 1 .add true (.var 0) (.const c ty)], [.var 1], xs⟩,
   ⟨[(0, ty)], [.bin 1 .eq false (.var 0) (.const c ty)], [.var 1], xs⟩]

/-- An array of two values of type `ty`, then `array_get` at every position and
one past the end, and `array_set` at every position followed by a read of the
position it wrote. -/
def arrayCases : List Case := do
  let ty ← types
  let mk : Instruction := .makeArray 2 [.var 0, .var 1] (.array [ty] 2)
  let calls := [[1, 2], [0, 1]].map fun l => l.map fun k => (values ty).getD k 0
  let gets := [0, 1, 2].map fun k =>
    (⟨[(0, ty), (1, ty)], [mk, .arrayGet 3 (.var 2) (.const k (.uint 32)) ty], [.var 3], calls⟩ : Case)
  let sets := [0, 1].map fun k =>
    (⟨[(0, ty), (1, ty)],
      [mk, .arraySet 3 false (.var 2) (.const k (.uint 32)) (.var 0),
        .arrayGet 4 (.var 3) (.const (1 - k) (.uint 32)) ty], [.var 4], calls⟩ : Case)
  gets ++ sets

/-- `enable_side_effects` on a `u1` parameter holding each of its edge-case
values (the SSA validator rejects any other type). -/
def enableCases : List Case :=
  [⟨[(0, .uint 1)], [.enableSideEffects (.var 0)], [], (values (.uint 1)).map fun x => [x]⟩]

/-- Every instruction side effects affect, behind `enable_side_effects v2`
with `v2 = 0`: binary instructions on each type, `constrain !=`, and
`array_set` at every position and one past the end. -/
def disabledCases : List Case := do
  let ty ← types
  let off (body : List Instruction) (rets : List Operand) (calls : List (List ℕ)) : Case :=
    ⟨[(0, ty), (1, ty), (2, .uint 1)], .enableSideEffects (.var 2) :: body, rets,
      calls.map (· ++ [0])⟩
  let mk : Instruction := .makeArray 3 [.var 0, .var 1] (.array [ty] 2)
  let arrayCalls := [[1, 2], [0, 1]].map fun l => l.map fun k => (values ty).getD k 0
  ((binaryOps ty).map fun (op, u) => off [.bin 3 op u (.var 0) (.var 1)] [.var 3] (pairs ty)) ++
    [off [.constrainNe (.var 0) (.var 1) none] [] (pairs ty)] ++
    [0, 1, 2].map fun k =>
      off [mk, .arraySet 4 false (.var 3) (.const k (.uint 32)) (.var 1),
        .arrayGet 5 (.var 4) (.const 0 (.uint 32)) ty] [.var 5] arrayCalls

def render : String :=
  String.join ((binaryCases ++ unaryCases ++ constrainCases ++ negativeCases ++ arrayCases ++
    enableCases ++ disabledCases).map Case.lines)

end AcirLean.SemanticsTable

def main (args : List String) : IO Unit := do
  match args with
  | [path] => IO.FS.writeFile path AcirLean.SemanticsTable.render
  | _ => IO.print AcirLean.SemanticsTable.render
