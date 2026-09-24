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
and assertions over `Field` and fixed-width integers. `Prog2.render` prints it
in the syntax `Ssa`'s `Display` uses, which is how the pin ties a program here
to the SSA `nargo compile` actually produced.

A value is a field element with a type. An integer of width `n` is a field
element whose integer value is below `2^n`; a signed integer is its
two's-complement bit pattern.
-/

namespace AcirLean

inductive VTy where
  | field
  | uint (n : ℕ)
  | sint (n : ℕ)
  deriving DecidableEq

/-- An SSA value, or a typed constant (its integer value). -/
inductive Opnd where
  | var (id : ℕ)
  | const (v : ℕ) (ty : VTy)
  deriving DecidableEq

inductive BOp where
  | add | sub | mul | div | mod | lt | eq
  deriving DecidableEq

inductive Ins where
  /-- `v<dst> = [unchecked_]<op> <a>, <b>` -/
  | bin (dst : ℕ) (op : BOp) (unchecked : Bool) (a b : Opnd)
  /-- `v<dst> = not <a>` -/
  | not (dst : ℕ) (a : Opnd)
  /-- `v<dst> = cast <a> as <ty>` -/
  | cast (dst : ℕ) (a : Opnd) (ty : VTy)
  /-- `v<dst> = truncate <a> to <bits> bits, max_bit_size: <maxBits>` -/
  | truncate (dst : ℕ) (a : Opnd) (bits maxBits : ℕ)
  /-- `constrain <a> == <b>[, "<msg>"]` -/
  | constrain (a b : Opnd) (msg : Option String)
  /-- `range_check <a> to <bits> bits[, "<msg>"]` -/
  | rangeCheck (a : Opnd) (bits : ℕ) (msg : Option String)
  deriving DecidableEq

/-- `<header>` / `b0(<params>):` / `<body>` / `return <rets>` / `}`. -/
structure Prog2 where
  header : String
  params : List (ℕ × VTy)
  body : List Ins
  rets : List Opnd
  deriving DecidableEq

/-! ## Printing -/

def VTy.render : VTy → String
  | .field => "Field"
  | .uint n => s!"u{n}"
  | .sint n => s!"i{n}"

def Opnd.render : Opnd → String
  | .var id => s!"v{id}"
  | .const v ty => s!"{ty.render} {v}"

def BOp.name : BOp → String
  | .add => "add" | .sub => "sub" | .mul => "mul" | .div => "div" | .mod => "mod"
  | .lt => "lt" | .eq => "eq"

def msgSuffix : Option String → String
  | none => ""
  | some m => s!", \"{m}\""

def Ins.render : Ins → String
  | .bin d op u a b =>
    s!"    v{d} = {if u then "unchecked_" else ""}{op.name} {a.render}, {b.render}"
  | .not d a => s!"    v{d} = not {a.render}"
  | .cast d a ty => s!"    v{d} = cast {a.render} as {ty.render}"
  | .truncate d a k m => s!"    v{d} = truncate {a.render} to {k} bits, max_bit_size: {m}"
  | .constrain a b m => s!"    constrain {a.render} == {b.render}{msgSuffix m}"
  | .rangeCheck a k m => s!"    range_check {a.render} to {k} bits{msgSuffix m}"

def Prog2.render (P : Prog2) : List String :=
  let params := ", ".intercalate (P.params.map fun (id, ty) => s!"v{id}: {ty.render}")
  let ret := if P.rets.isEmpty then "    return"
    else "    return " ++ ", ".intercalate (P.rets.map Opnd.render)
  [P.header, s!"  b0({params}):"] ++ P.body.map Ins.render ++ [ret, "}"]

/-! ## Meaning -/

/-- The value fits its type. -/
def VTy.fits : VTy → F → Bool
  | .field, _ => true
  | .uint n, x => decide (x.val < 2 ^ n)
  | .sint n, x => decide (x.val < 2 ^ n)

/-- Values of the SSA variables so far. -/
abbrev Env := List (ℕ × (F × VTy))

def Opnd.value (env : Env) : Opnd → Option (F × VTy)
  | .var id => env.lookup id
  | .const v ty => some ((v : F), ty)

/-- `1` or `0`, as a `u1`. -/
def flag (b : Bool) : F × VTy := (if b then 1 else 0, .uint 1)

/-- A binary instruction on `x` and `y`, both of `x`'s type. Integer arithmetic
fails when the result does not fit (unchecked arithmetic included: it is only
meaningful when the compiler has shown it cannot overflow), and division fails
on a zero divisor. `Field` supports `add`, `sub`, `mul` and `eq`. -/
def BOp.apply (op : BOp) (x y : F) : VTy → Option (F × VTy)
  | .field =>
    match op with
    | .add => some (x + y, .field)
    | .sub => some (x - y, .field)
    | .mul => some (x * y, .field)
    | .eq => some (flag (x = y))
    | _ => none
  | .uint n =>
    match op with
    | .add => if x.val + y.val < 2 ^ n then some (((x.val + y.val : ℕ) : F), .uint n) else none
    | .sub => if y.val ≤ x.val then some (((x.val - y.val : ℕ) : F), .uint n) else none
    | .mul => if x.val * y.val < 2 ^ n then some (((x.val * y.val : ℕ) : F), .uint n) else none
    | .div => if y.val = 0 then none else some (((x.val / y.val : ℕ) : F), .uint n)
    | .mod => if y.val = 0 then none else some (((x.val % y.val : ℕ) : F), .uint n)
    | .lt => some (flag (x.val < y.val))
    | .eq => some (flag (x = y))
  | .sint _ =>
    match op with
    | .eq => some (flag (x = y))
    | _ => none

/-- Run one instruction. `cast` fails if the value does not fit the new type,
`truncate` keeps the low `bits` bits, and `constrain` and `range_check` fail
when their condition does not hold. -/
def Ins.run (env : Env) : Ins → Option Env
  | .bin d op _ a b => do
    let (x, tx) ← a.value env
    let (y, _) ← b.value env
    let r ← op.apply x y tx
    some ((d, r) :: env)
  | .not d a => do
    let (x, tx) ← a.value env
    match tx with
    | .uint n => some ((d, (((2 ^ n - 1 - x.val : ℕ) : F), .uint n)) :: env)
    | _ => none
  | .cast d a ty => do
    let (x, _) ← a.value env
    if ty.fits x then some ((d, (x, ty)) :: env) else none
  | .truncate d a k _ => do
    let (x, tx) ← a.value env
    some ((d, (((x.val % 2 ^ k : ℕ) : F), tx)) :: env)
  | .constrain a b _ => do
    let (x, _) ← a.value env
    let (y, _) ← b.value env
    if x = y then some env else none
  | .rangeCheck a k _ => do
    let (x, _) ← a.value env
    if x.val < 2 ^ k then some env else none

/-- Bind the parameters, run the body, and read the return values. -/
def Prog2.eval (P : Prog2) (ins : List F) : Option (List F) := do
  let env0 : Env := (P.params.zip ins).map fun ((id, ty), x) => (id, (x, ty))
  let env ← P.body.foldlM Ins.run env0
  P.rets.mapM fun o => (o.value env).map Prod.fst

/-- A circuit implements the function: it takes one input per parameter,
enforces each parameter's type, and returns exactly what the function returns
(so it rejects every input on which the function fails). -/
def ProgSpec2 (P : Prog2) : List ℕ → List ℕ → Prop := fun ins outs =>
  ins.length = P.params.length ∧
    (∀ e ∈ P.params.zip ins, e.1.2.fits (e.2 : F) = true) ∧
    ∃ vs, P.eval (ins.map fun x => (x : F)) = some vs ∧ outs = vs.map ZMod.val

/-- A test program: its final SSA and the circuit `nargo compile` shipped. -/
structure ProgEntry where
  name : String
  prog : Prog2
  fn : AcirFn

end AcirLean
