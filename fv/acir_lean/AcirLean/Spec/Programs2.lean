/-
REVIEWED: this file is part of the trusted specification (`AcirLean/Spec/`).
Every definition here is taken on trust: read it against its comment. A change
to this directory needs careful review.
-/

import AcirLean.Spec.Semantics

/-!
# Scalar SSA functions and what they compute

The final SSA of a Noir `main` function made of one block of scalar
instructions: arithmetic, comparisons, `not`, casts, truncation, range checks
and assertions over `Field` and fixed-width integers. `Program.render` prints it
in the syntax `Ssa`'s `Display` uses, which is how the pin ties a program here
to the SSA `nargo compile` actually produced.

A value is a field element with a type. An integer of width `n` is a field
element whose integer value is below `2^n`; a signed integer is its
two's-complement bit pattern.
-/

namespace AcirLean

inductive ValueType where
  | field
  | uint (n : ℕ)
  | sint (n : ℕ)
  deriving DecidableEq

/-- An SSA value, or a typed constant (its integer value). -/
inductive Operand where
  | var (id : ℕ)
  | const (v : ℕ) (ty : ValueType)
  deriving DecidableEq

inductive BinaryOp where
  | add | sub | mul | div | mod | lt | eq
  deriving DecidableEq

inductive Instruction where
  /-- `v<dst> = [unchecked_]<op> <a>, <b>` -/
  | bin (dst : ℕ) (op : BinaryOp) (unchecked : Bool) (a b : Operand)
  /-- `v<dst> = not <a>` -/
  | not (dst : ℕ) (a : Operand)
  /-- `v<dst> = cast <a> as <ty>` -/
  | cast (dst : ℕ) (a : Operand) (ty : ValueType)
  /-- `v<dst> = truncate <a> to <bits> bits, max_bit_size: <maxBits>` -/
  | truncate (dst : ℕ) (a : Operand) (bits maxBits : ℕ)
  /-- `constrain <a> == <b>[, "<msg>"]` -/
  | constrain (a b : Operand) (msg : Option String)
  /-- `range_check <a> to <bits> bits[, "<msg>"]` -/
  | rangeCheck (a : Operand) (bits : ℕ) (msg : Option String)
  deriving DecidableEq

/-- `<header>` / `b0(<params>):` / `<body>` / `return <rets>` / `}`. -/
structure Program where
  header : String
  params : List (ℕ × ValueType)
  body : List Instruction
  rets : List Operand
  deriving DecidableEq

/-! ## Printing -/

def ValueType.render : ValueType → String
  | .field => "Field"
  | .uint n => s!"u{n}"
  | .sint n => s!"i{n}"

def Operand.render : Operand → String
  | .var id => s!"v{id}"
  | .const v ty => s!"{ty.render} {v}"

def BinaryOp.name : BinaryOp → String
  | .add => "add" | .sub => "sub" | .mul => "mul" | .div => "div" | .mod => "mod"
  | .lt => "lt" | .eq => "eq"

def msgSuffix : Option String → String
  | none => ""
  | some m => s!", \"{m}\""

def Instruction.render : Instruction → String
  | .bin d op u a b =>
    s!"    v{d} = {if u then "unchecked_" else ""}{op.name} {a.render}, {b.render}"
  | .not d a => s!"    v{d} = not {a.render}"
  | .cast d a ty => s!"    v{d} = cast {a.render} as {ty.render}"
  | .truncate d a k m => s!"    v{d} = truncate {a.render} to {k} bits, max_bit_size: {m}"
  | .constrain a b m => s!"    constrain {a.render} == {b.render}{msgSuffix m}"
  | .rangeCheck a k m => s!"    range_check {a.render} to {k} bits{msgSuffix m}"

def Program.render (P : Program) : List String :=
  let params := ", ".intercalate (P.params.map fun (id, ty) => s!"v{id}: {ty.render}")
  let ret := if P.rets.isEmpty then "    return"
    else "    return " ++ ", ".intercalate (P.rets.map Operand.render)
  [P.header, s!"  b0({params}):"] ++ P.body.map Instruction.render ++ [ret, "}"]

/-! ## Meaning -/

/-- The value fits its type. -/
def ValueType.fits : ValueType → F → Bool
  | .field, _ => true
  | .uint n, x => decide (x.val < 2 ^ n)
  | .sint n, x => decide (x.val < 2 ^ n)

/-- Values of the SSA variables so far. -/
abbrev Env := List (ℕ × (F × ValueType))

def Operand.value (env : Env) : Operand → Option (F × ValueType)
  | .var id => env.lookup id
  | .const v ty => some ((v : F), ty)

/-- `1` or `0`, as a `u1`. -/
def flag (b : Bool) : F × ValueType := (if b then 1 else 0, .uint 1)

/-- `add`, `sub` and `mul` in the field. -/
def fieldArith : BinaryOp → F → F → Option F
  | .add, x, y => some (x + y)
  | .sub, x, y => some (x - y)
  | .mul, x, y => some (x * y)
  | _, _, _ => none

/-- A binary instruction on `x` and `y`, both of `x`'s type, as Noir's SSA
interpreter evaluates it in an ACIR function (`evaluate_integer_binary`):
* on `Field`: `add`, `sub`, `mul` and `eq`;
* unchecked `add`, `sub` and `mul` on integers are field arithmetic: the result
  keeps the type but may exceed its width, until a later `truncate`;
* every other integer operation requires both operands to fit their type. The
  interpreter reduces them first; failing instead can only make a program
  unprovable, never give it a different value. Checked arithmetic then fails
  when the result does not fit, and `div` and `mod` fail on a zero divisor.

Signed integers support unchecked arithmetic and `eq`: `expand_signed_math`
rewrites their other operations before the SSA reaches ACIR. -/
def BinaryOp.apply (op : BinaryOp) (unchecked : Bool) (x y : F) :
    ValueType → Option (F × ValueType)
  | .field =>
    match op with
    | .add => some (x + y, .field)
    | .sub => some (x - y, .field)
    | .mul => some (x * y, .field)
    | .eq => some (flag (x = y))
    | _ => none
  | .uint n =>
    match unchecked, fieldArith op x y with
    | true, some r => some (r, .uint n)
    | _, _ =>
      if x.val < 2 ^ n ∧ y.val < 2 ^ n then
        match op with
        | .add => if x.val + y.val < 2 ^ n then some (((x.val + y.val : ℕ) : F), .uint n) else none
        | .sub => if y.val ≤ x.val then some (((x.val - y.val : ℕ) : F), .uint n) else none
        | .mul => if x.val * y.val < 2 ^ n then some (((x.val * y.val : ℕ) : F), .uint n) else none
        | .div => if y.val = 0 then none else some (((x.val / y.val : ℕ) : F), .uint n)
        | .mod => if y.val = 0 then none else some (((x.val % y.val : ℕ) : F), .uint n)
        | .lt => some (flag (x.val < y.val))
        | .eq => some (flag (x = y))
      else none
  | .sint n =>
    match unchecked, fieldArith op x y with
    | true, some r => some (r, .sint n)
    | _, _ =>
      match op with
      | .eq => if x.val < 2 ^ n ∧ y.val < 2 ^ n then some (flag (x = y)) else none
      | _ => none

/-- Run one instruction, as Noir's SSA interpreter does in an ACIR function
(`interpret_instruction`):
* `not` on an `n`-bit integer reduces the value to its low `n` bits and flips
  them: `2^n - 1 - (x mod 2^n)`. A `u1` above `1` fails (the interpreter
  asserts a `u1` is `0` or `1`);
* `cast` keeps the value and changes its type, without checking that it fits
  (the interpreter relabels, and a later `truncate` makes it fit);
* `truncate` keeps the low `bits` bits. It fails for `0` bits and for a `u1`
  above `1`, as the interpreter does;
* `constrain` and `range_check` fail when their condition does not hold. -/
def Instruction.run (env : Env) : Instruction → Option Env
  | .bin d op u a b => do
    let (x, tx) ← a.value env
    let (y, _) ← b.value env
    let r ← op.apply u x y tx
    some ((d, r) :: env)
  | .not d a => do
    let (x, tx) ← a.value env
    match tx with
    | .uint n | .sint n =>
      if tx = .uint 1 ∧ 2 ≤ x.val then none
      else some ((d, (((2 ^ n - 1 - x.val % 2 ^ n : ℕ) : F), tx)) :: env)
    | .field => none
  | .cast d a ty => do
    let (x, _) ← a.value env
    some ((d, (x, ty)) :: env)
  | .truncate d a k _ => do
    let (x, tx) ← a.value env
    if 0 < k ∧ (tx = .uint 1 → x.val < 2) then
      some ((d, (((x.val % 2 ^ k : ℕ) : F), tx)) :: env)
    else none
  | .constrain a b _ => do
    let (x, _) ← a.value env
    let (y, _) ← b.value env
    if x = y then some env else none
  | .rangeCheck a k _ => do
    let (x, _) ← a.value env
    if x.val < 2 ^ k then some env else none

/-- Bind the parameters, run the body, and read the return values. -/
def Program.eval (P : Program) (ins : List F) : Option (List F) := do
  let env0 : Env := (P.params.zip ins).map fun ((id, ty), x) => (id, (x, ty))
  let env ← P.body.foldlM Instruction.run env0
  P.rets.mapM fun o => (o.value env).map Prod.fst

/-- A circuit implements the function: it takes one input per parameter,
enforces each parameter's type, and returns exactly what the function returns
(so it rejects every input on which the function fails). -/
def ProgramSpec (P : Program) : List ℕ → List ℕ → Prop := fun ins outs =>
  ins.length = P.params.length ∧
    (∀ e ∈ P.params.zip ins, e.1.2.fits (e.2 : F) = true) ∧
    ∃ vs, P.eval (ins.map fun x => (x : F)) = some vs ∧ outs = vs.map ZMod.val

/-- A test program: its final SSA and the circuit `nargo compile` shipped. -/
structure TestProgram where
  name : String
  prog : Program
  fn : Circuit

end AcirLean
