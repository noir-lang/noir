/-
REVIEWED: this file is part of the trusted specification (`AcirLean/Spec/`).
Every definition here is taken on trust: read it against its comment. A change
to this directory needs careful review.
-/

import Mathlib.Data.Nat.Bitwise

/-!
# What a straight-line SSA function means

The subset of Noir SSA that `expand_signed_math` emits for a signed `lt`: casts
and the unsigned binary operations `div`, `lt` and `xor`, in one block. A value
is its bit pattern as a natural number, so an `i8` holding `-1` is `255`.

`cast` keeps the bit pattern (both ACIR generation and the SSA interpreter
treat it as a relabelling). `div` is integer division, `lt` is `1` or `0`, and
`xor` is bitwise.
-/

namespace AcirLean

/-- `u<n>` or `i<n>`. -/
inductive IntType where
  | u (n : ℕ)
  | i (n : ℕ)

/-- A value id, or a typed constant such as `u8 128`. -/
inductive SsaOperand where
  | var (id : ℕ)
  | const (ty : IntType) (v : ℕ)

inductive SsaBinOp where
  | div
  | lt
  | xor

/-- `v<dst> = cast v<src> as <ty>`, or `v<dst> = <op> <a>, <b>`. -/
inductive SsaInstruction where
  | cast (dst src : ℕ) (ty : IntType)
  | bin (dst : ℕ) (op : SsaBinOp) (a b : SsaOperand)

/-- `acir(inline) fn main f0 { b0(<params>): <body> return v<ret> }`. -/
structure SsaFunction where
  params : List (ℕ × IntType)
  body : List SsaInstruction
  ret : ℕ

def SsaOperand.eval (env : ℕ → ℕ) : SsaOperand → ℕ
  | .var id => env id
  | .const _ v => v

def SsaBinOp.eval : SsaBinOp → ℕ → ℕ → ℕ
  | .div, a, b => a / b
  | .lt, a, b => if a < b then 1 else 0
  | .xor, a, b => a ^^^ b

/-- Execute one instruction: set `v<dst>`, leave every other value unchanged. -/
def SsaInstruction.step (env : ℕ → ℕ) : SsaInstruction → (ℕ → ℕ)
  | .cast dst src _ => fun i => if i = dst then env src else env i
  | .bin dst op a b => fun i => if i = dst then op.eval (a.eval env) (b.eval env) else env i

/-- Bind the parameters to `args` in order, run the body, and return `v<ret>`. -/
def SsaFunction.run (f : SsaFunction) (args : List ℕ) : ℕ :=
  let env0 : ℕ → ℕ := fun i =>
    (((f.params.map Prod.fst).zip args).lookup i).getD 0
  (f.body.foldl SsaInstruction.step env0) f.ret

/-! ## Printing, in the syntax `Ssa`'s `Display` uses -/

def IntType.render : IntType → String
  | .u n => s!"u{n}"
  | .i n => s!"i{n}"

def SsaOperand.render : SsaOperand → String
  | .var id => s!"v{id}"
  | .const ty v => s!"{ty.render} {v}"

def SsaBinOp.render : SsaBinOp → String
  | .div => "div"
  | .lt => "lt"
  | .xor => "xor"

def SsaInstruction.render : SsaInstruction → String
  | .cast dst src ty => s!"    v{dst} = cast v{src} as {ty.render}"
  | .bin dst op a b => s!"    v{dst} = {op.render} {a.render}, {b.render}"

def SsaFunction.render (f : SsaFunction) : List String :=
  let params := ", ".intercalate (f.params.map fun (id, ty) => s!"v{id}: {ty.render}")
  ["acir(inline) fn main f0 {", s!"  b0({params}):"] ++ f.body.map SsaInstruction.render ++
    [s!"    return v{f.ret}", "}"]

end AcirLean
