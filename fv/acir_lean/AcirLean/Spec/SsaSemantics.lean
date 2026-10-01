/-
REVIEWED: this file is part of the trusted specification (`AcirLean/Spec/`).
Every definition here is taken on trust: read it against its comment. A change
to this directory needs careful review.
-/

import AcirLean.Spec.Semantics

/-!
# SSA functions and what they compute

The final SSA of a Noir `main` function made of one block of instructions:
arithmetic, comparisons, `not`, casts, truncation, range checks and assertions
over `Field` and fixed-width integers, and arrays of them. `Program.render`
prints it in the syntax `Ssa`'s `Display` uses, which is how the pin ties a
program here to the SSA `nargo compile` actually produced.

A scalar is a field element with a type. An integer of width `n` is a field
element whose integer value is below `2^n`; a signed integer is its
two's-complement bit pattern. An array is the list of its scalars, flattened:
an array of `n` tuples `(T1, …, Tk)` holds `n * k` scalars, and SSA indexes it
by flat position (element `i`'s field `j` is at `i * k + j`). In the circuit,
an array parameter or return value is one witness per scalar, in the same
order.
-/

namespace AcirLean

inductive ValueType where
  | field
  | uint (n : ℕ)
  | sint (n : ℕ)
  deriving DecidableEq

/-- A parameter's type: a scalar, or `[T; n]` / `[(T1, …, Tk); n]`. -/
inductive ParamType where
  | scalar (t : ValueType)
  | array (elems : List ValueType) (len : ℕ)
  deriving DecidableEq

/-- An SSA value, or a typed constant as SSA prints it (see `constVal`). -/
inductive Operand where
  | var (id : ℕ)
  | const (v : ℤ) (ty : ValueType)
  deriving DecidableEq

inductive BinaryOp where
  | add | sub | mul | div | mod | lt | eq | xor
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
  /-- `constrain <a> != <b>[, "<msg>"]` -/
  | constrainNe (a b : Operand) (msg : Option String)
  /-- `range_check <a> to <bits> bits[, "<msg>"]` -/
  | rangeCheck (a : Operand) (bits : ℕ) (msg : Option String)
  /-- `v<dst> = array_get <a>, index <i> -> <ty>` -/
  | arrayGet (dst : ℕ) (a i : Operand) (ty : ValueType)
  /-- `v<dst> = array_set [mut ]<a>, index <i>, value <v>` -/
  | arraySet (dst : ℕ) (isMut : Bool) (a i v : Operand)
  /-- `v<dst> = make_array [<elems>] : <ty>` -/
  | makeArray (dst : ℕ) (elems : List Operand) (ty : ParamType)
  /-- `enable_side_effects <c>` -/
  | enableSideEffects (c : Operand)
  deriving DecidableEq

/-- `<header>` / `b0(<params>):` / `<body>` / `return <rets>` / `}`. -/
structure Program where
  header : String
  params : List (ℕ × ParamType)
  body : List Instruction
  rets : List Operand
  deriving DecidableEq

/-! ## Printing -/

def ValueType.render : ValueType → String
  | .field => "Field"
  | .uint n => s!"u{n}"
  | .sint n => s!"i{n}"

def ParamType.render : ParamType → String
  | .scalar t => t.render
  | .array [t] n => s!"[{t.render}; {n}]"
  | .array ts n => s!"[({", ".intercalate (ts.map ValueType.render)}); {n}]"

def Operand.render : Operand → String
  | .var id => s!"v{id}"
  | .const v ty => s!"{ty.render} {v}"

def BinaryOp.name : BinaryOp → String
  | .add => "add" | .sub => "sub" | .mul => "mul" | .div => "div" | .mod => "mod"
  | .lt => "lt" | .eq => "eq" | .xor => "xor"

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
  | .constrainNe a b m => s!"    constrain {a.render} != {b.render}{msgSuffix m}"
  | .rangeCheck a k m => s!"    range_check {a.render} to {k} bits{msgSuffix m}"
  | .arrayGet d a i ty => s!"    v{d} = array_get {a.render}, index {i.render} -> {ty.render}"
  | .arraySet d m a i v =>
    s!"    v{d} = array_set {if m then "mut " else ""}{a.render}, index {i.render}, value {v.render}"
  | .makeArray d es ty =>
    s!"    v{d} = make_array [{", ".intercalate (es.map Operand.render)}] : {ty.render}"
  | .enableSideEffects c => s!"    enable_side_effects {c.render}"

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

/-- The types of a parameter's scalars, in order. -/
def ParamType.flat : ParamType → List ValueType
  | .scalar t => [t]
  | .array ts n => (List.replicate n ts).flatten

/-- An SSA value: a scalar, or an array's scalars in flat order. -/
inductive Value where
  | scalar (v : F × ValueType)
  | array (xs : List (F × ValueType))

/-- Values of the SSA variables so far. -/
abbrev Env := List (ℕ × Value)

/-- A constant's value: a signed integer is its two's-complement bit pattern
(`i8 -1` is `255`); any other constant is its value mod `p`. -/
def constVal : ValueType → ℤ → F
  | .sint n, v => ((v % 2 ^ n).toNat : F)
  | _, v => (v : F)

/-- A scalar operand's value. -/
def Operand.value (env : Env) : Operand → Option (F × ValueType)
  | .var id => match env.lookup id with
    | some (.scalar v) => some v
    | _ => none
  | .const v ty => some (constVal ty v, ty)

/-- An array operand's scalars. -/
def Operand.array (env : Env) : Operand → Option (List (F × ValueType))
  | .var id => match env.lookup id with
    | some (.array xs) => some xs
    | _ => none
  | .const _ _ => none

/-- An operand's scalars, in flat order: one for a scalar, all for an array. -/
def Operand.flat (env : Env) : Operand → Option (List F)
  | .var id => match env.lookup id with
    | some (.scalar v) => some [v.1]
    | some (.array xs) => some (xs.map Prod.fst)
    | none => none
  | .const v ty => some [constVal ty v]

/-- An array index: a `u32` below the array's length (the interpreter reads it
with `as_u32` and fails past the end). -/
def arrayIndex (i : F × ValueType) (len : ℕ) : Option ℕ :=
  if i.2 = .uint 32 ∧ i.1.val < len then some i.1.val else none

/-- `1` or `0`, as a `u1`. -/
def flag (b : Bool) : F × ValueType := (if b then 1 else 0, .uint 1)

/-- `add`, `sub` and `mul` in the field. -/
def fieldArith : BinaryOp → F → F → Option F
  | .add, x, y => some (x + y)
  | .sub, x, y => some (x - y)
  | .mul, x, y => some (x * y)
  | _, _, _ => none

/-- The low `n` bits of `x`. The interpreter reduces integer operands to their
type's width this way (`truncate_field`) before dividing or comparing them. -/
def lowBits (n : ℕ) (x : F) : ℕ := x.val % 2 ^ n

/-- A binary instruction on two `u1` values, as booleans
(`interpret_u1_binary_op`). Unchecked `add` is `xor`, checked `add` fails on
`1 + 1`, and `sub` fails on `0 - 1` whether checked or not. -/
def u1Apply (op : BinaryOp) (unchecked x y : Bool) : Option Bool :=
  match op with
  | .add => if !unchecked && x && y then none else some (x ^^ y)
  | .sub => if !x && y then none else some (x ^^ y)
  | .mul => some (x && y)
  | .div => if y then some x else none
  | .mod => if y then some false else none
  | .lt => some (!x && y)
  | .eq => some (x == y)
  | .xor => some (x ^^ y)

/-- A binary instruction on `x` and `y`, both of `x`'s type, as Noir's SSA
interpreter evaluates it in an ACIR function (`evaluate_binary`):
* `Field`: `add`, `sub`, `mul` and `div` in the field (`div` fails on a zero
  divisor), `lt` on the integer values, `eq`; `mod` is not defined;
* `u1`: both values must be `0` or `1` (the interpreter asserts it), then
  `u1Apply`;
* other `u<n>`: unchecked `add`, `sub` and `mul` are field arithmetic, and the
  result keeps the type even if it no longer fits (a later `truncate` brings it
  back). Checked `add`, `sub` and `mul` compute the same field result and fail
  unless it fits in `n` bits; a checked `u128` `mul` also fails when the
  product of the unreduced values reaches `2^128`. `div`, `mod`, `lt` and `eq`
  act on the operands' low `n` bits, and `div` and `mod` fail on a zero
  divisor;
* `i<n>`: unchecked `add`, `sub` and `mul` as for `u<n>`, and `eq` on the low
  `n` bits. The interpreter also defines their checked arithmetic, `div`,
  `mod` and `lt`, but `expand_signed_math` rewrites those before the SSA
  reaches ACIR, so they are left undefined here;
* `xor` is defined on `u1` only (through `u1Apply`). The interpreter also
  defines it bitwise on wider integers, which ACIR computes with a black-box
  function this spec does not model. -/
def BinaryOp.apply (op : BinaryOp) (unchecked : Bool) (x y : F) :
    ValueType → Option (F × ValueType)
  | .field =>
    match op with
    | .add => some (x + y, .field)
    | .sub => some (x - y, .field)
    | .mul => some (x * y, .field)
    | .div => if y = 0 then none else some (x * y⁻¹, .field)
    | .lt => some (flag (x.val < y.val))
    | .eq => some (flag (x = y))
    | .mod | .xor => none
  | .uint 1 =>
    if x.val < 2 ∧ y.val < 2 then (u1Apply op unchecked (x.val = 1) (y.val = 1)).map flag
    else none
  | .uint n =>
    match op, fieldArith op x y with
    | .div, _ =>
      if lowBits n y = 0 then none else some (((lowBits n x / lowBits n y : ℕ) : F), .uint n)
    | .mod, _ =>
      if lowBits n y = 0 then none else some (((lowBits n x % lowBits n y : ℕ) : F), .uint n)
    | .lt, _ => some (flag (lowBits n x < lowBits n y))
    | .eq, _ => some (flag (lowBits n x = lowBits n y))
    | _, some r =>
      if unchecked then some (r, .uint n)
      else if r.val < 2 ^ n ∧ (op = .mul → n = 128 → x.val * y.val < 2 ^ 128) then
        some (r, .uint n)
      else none
    | _, none => none
  | .sint n =>
    match op, fieldArith op x y with
    | .eq, _ => some (flag (lowBits n x = lowBits n y))
    | _, some r => if unchecked then some (r, .sint n) else none
    | _, none => none

/-- Run one instruction with side effects enabled, as Noir's SSA interpreter
does in an ACIR function (`interpret_instruction`); `Instruction.step` handles
`enable_side_effects` and disabled side effects:
* `not` on an `n`-bit integer reduces the value to its low `n` bits and flips
  them: `2^n - 1 - (x mod 2^n)`. A `u1` above `1` fails (the interpreter
  asserts a `u1` is `0` or `1`);
* `cast` keeps the value and changes its type, without checking that it fits
  (the interpreter relabels, and a later `truncate` makes it fit);
* `truncate` keeps the low `bits` bits, so truncating to `0` bits gives `0`.
  Otherwise it fails for a `u1` above `1`, as the interpreter does;
* `constrain` fails unless its operands are equal, and `constrain !=` fails if
  they are equal;
* `array_get` reads the scalar at a flat position, and `array_set` returns a
  copy of the array with that position replaced (arrays are values in ACIR
  functions, so `mut` does not change the result). Both fail unless the index
  is a `u32` below the array's length;
* `make_array` collects its scalar operands;
* `range_check` fails unless the value is below `2^bits`. It also fails for
  `0` bits and for a `u1` above `1`, as the interpreter does. -/
def Instruction.run (env : Env) : Instruction → Option Env
  | .bin d op u a b => do
    let (x, tx) ← a.value env
    let (y, _) ← b.value env
    let r ← op.apply u x y tx
    some ((d, .scalar r) :: env)
  | .not d a => do
    let (x, tx) ← a.value env
    match tx with
    | .uint n | .sint n =>
      if tx = .uint 1 ∧ 2 ≤ x.val then none
      else some ((d, .scalar (((2 ^ n - 1 - x.val % 2 ^ n : ℕ) : F), tx)) :: env)
    | .field => none
  | .cast d a ty => do
    let (x, _) ← a.value env
    some ((d, .scalar (x, ty)) :: env)
  | .truncate d a k _ => do
    let (x, tx) ← a.value env
    if k = 0 ∨ (tx = .uint 1 → x.val < 2) then
      some ((d, .scalar (((x.val % 2 ^ k : ℕ) : F), tx)) :: env)
    else none
  | .constrain a b _ => do
    let (x, _) ← a.value env
    let (y, _) ← b.value env
    if x = y then some env else none
  | .constrainNe a b _ => do
    let (x, _) ← a.value env
    let (y, _) ← b.value env
    if x = y then none else some env
  | .rangeCheck a k _ => do
    let (x, tx) ← a.value env
    if 0 < k ∧ x.val < 2 ^ k ∧ (tx = .uint 1 → x.val < 2) then some env else none
  | .arrayGet d a i _ => do
    let xs ← a.array env
    let idx ← i.value env
    let j ← arrayIndex idx xs.length
    let v ← xs[j]?
    some ((d, .scalar v) :: env)
  | .arraySet d _ a i v => do
    let xs ← a.array env
    let idx ← i.value env
    let j ← arrayIndex idx xs.length
    let x ← v.value env
    some ((d, .array (xs.set j x)) :: env)
  | .makeArray d es _ => do
    let xs ← es.mapM (·.value env)
    some ((d, .array xs) :: env)
  | .enableSideEffects _ => some env

/-- The binary instructions Noir's interpreter skips while side effects are
disabled (`requires_acir_gen_predicate`), given the first operand's type:
checked `add`, `sub` and `mul` on integers, and `div` and `mod`. The
interpreter reads the type from the second operand; the two agree in every
program it accepts. -/
def BinaryOp.predicated (op : BinaryOp) (unchecked : Bool) (ty : ValueType) : Bool :=
  match op with
  | .add | .sub | .mul => !unchecked && ty != .field
  | .div | .mod => true
  | _ => false

/-- Run one instruction with the side-effects flag `s.2`, as the interpreter
does (`side_effects_enabled`):
* `enable_side_effects c` sets the flag to `c`, which must be a `u1` holding
  `0` or `1`;
* while the flag is off, a `predicated` binary instruction gives `0` of its
  type, `constrain !=` does nothing, and `array_set` returns the array
  unchanged;
* everything else, including `constrain ==` and `range_check`, runs as
  `Instruction.run` says whatever the flag. -/
def Instruction.step (s : Env × Bool) (i : Instruction) : Option (Env × Bool) :=
  match i, s.2 with
  | .enableSideEffects c, _ => do
    let (x, tx) ← c.value s.1
    if tx = .uint 1 ∧ x.val < 2 then some (s.1, decide (x.val = 1)) else none
  | .bin d op u a b, false => do
    let (_, tx) ← a.value s.1
    let _ ← b.value s.1
    if op.predicated u tx then some ((d, .scalar (0, tx)) :: s.1, false)
    else (i.run s.1).map (·, false)
  | .constrainNe a b _, false => do
    let _ ← a.value s.1
    let _ ← b.value s.1
    some (s.1, false)
  | .arraySet d _ a _ _, false => do
    let xs ← a.array s.1
    some ((d, .array xs) :: s.1, false)
  | i, en => (i.run s.1).map (·, en)

/-- Bind each parameter to its scalars, taken in order from `ins`. -/
def bindParams : List (ℕ × ParamType) → List F → Env
  | [], _ => []
  | (id, t) :: ps, ins =>
    let xs := (ins.take t.flat.length).zip t.flat
    (id, match t with
      | .scalar _ => .scalar (xs.headD (0, .field))
      | .array _ _ => .array xs) :: bindParams ps (ins.drop t.flat.length)

/-- The scalar types of all the parameters, in order. -/
def Program.inputTypes (P : Program) : List ValueType := P.params.flatMap (·.2.flat)

/-- Bind the parameters, run the body with side effects enabled at the start,
and read the return values' scalars. -/
def Program.eval (P : Program) (ins : List F) : Option (List F) := do
  let (env, _) ← P.body.foldlM Instruction.step (bindParams P.params ins, true)
  let outs ← P.rets.mapM (·.flat env)
  some outs.flatten

/-- A circuit implements the function: it takes one input per parameter
scalar, enforces each scalar's type, and returns exactly what the function
returns (so it rejects every input on which the function fails). -/
def ProgramSpec (P : Program) : List ℕ → List ℕ → Prop := fun ins outs =>
  ins.length = P.inputTypes.length ∧
    (∀ e ∈ P.inputTypes.zip ins, e.1.fits (e.2 : F) = true) ∧
    ∃ vs, P.eval (ins.map fun x => (x : F)) = some vs ∧ outs = vs.map ZMod.val

/-- A test program: its final SSA, the circuit `nargo compile` shipped, and
the witness `nargo execute` solved for it from its `Prover.toml`, as
`(witness, value)` pairs. -/
structure TestProgram where
  name : String
  prog : Program
  fn : Circuit
  witness : List (ℕ × ℕ)

/-- The solved witness as an assignment (unlisted witnesses are `0`). -/
def TestProgram.assignment (e : TestProgram) (i : ℕ) : F :=
  ((e.witness.lookup i).getD 0 : ℕ)

end AcirLean
