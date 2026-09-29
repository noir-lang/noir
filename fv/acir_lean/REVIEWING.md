# Reviewing the Lean, line by line (no Lean knowledge needed)

This covers every file a human reviewer must read in `fv/acir_lean`: the six
files in `AcirLean/Spec/` and `Check.lean`. `scripts/check.sh` fails if a Lean
line quoted here no longer appears in those files, or if one of their
definitions is not mentioned here, so this guide stays in step with the code. For each one it explains:

- what the Lean says, in plain words;
- why it's there;
- what a reviewer should check.

Part 0 is a mini-glossary. Read it first, and the rest will make sense.

---

## Part 0 — just enough Lean to read these files

Lean looks a lot like a functional programming language (think Haskell, OCaml or Rust enums plus `match`). The Spec files only use a small part of it.

### Comments

| Lean | Meaning |
|---|---|
| `/-- text -/` | A doc comment for the definition right below it. **Read these. They say what each definition is supposed to mean, and your job as reviewer is to check that the code matches the comment.** |
| `/-! text -/` | A section or module comment. |
| `-- text` | A line comment. |

### Numbers and types

| Lean | Meaning |
|---|---|
| `ℕ` | Natural numbers: 0, 1, 2, … (unbounded, like `BigUint`) |
| `ℤ` | Integers, which may be negative (like `BigInt`) |
| `F` | Numbers mod `p` (the BN254 field). Defined in `Semantics.lean`. |
| `x.val` | For a field element `x`: its plain integer value, between 0 and p-1. `(σ 3).val < 2^8` means "witness 3, read as an ordinary number, is below 256". |
| `(n : F)` | Convert the natural number `n` into a field element (mod p). |
| `Bool` | `true` / `false`, like Rust's `bool`. |
| `Prop` | A mathematical *statement* that may be true or false. A proof is what establishes it. The claims are `Prop`s. |
| `List α` | A list, like `Vec<α>`. `[a, b, c]` is a literal. |
| `α × β` | A pair (tuple). `p.1` and `p.2` are its first and second parts. |
| `Option α` | `some x` or `none`, exactly like Rust's `Option`. The SSA semantics use `none` to mean "the program fails here". |
| `ℕ → F` | A function from ℕ to F. `σ : ℕ → F` is the name used everywhere for a *witness assignment*: `σ 7` is the value the prover put in witness 7. |

### Defining things

| Lean | Meaning |
|---|---|
| `def f (x : A) : B := body` | Define a function `f` taking `x` and returning `B`. |
| `def f : A → B` followed by lines `\| pattern => result` | Define `f` by pattern matching (like `match` in Rust). |
| `structure S where a : A; b : B` | A struct. `s.a` reads field `a`. |
| `inductive T where \| c1 (x : A) \| c2` | An enum. `.c1 x` builds a value, like `T::C1(x)`. |
| `abbrev` | Like `def`, for a short alias. |
| `deriving DecidableEq` | Auto-derive equality comparison, like `#[derive(PartialEq, Eq)]`. |
| `instance : …` | A trait impl. Only two small facts about `p` use it here. |
| `namespace AcirLean … end AcirLean` | A module scope. |
| `_root_.Int.tdiv` | The top-level `Int.tdiv`, never one defined inside `AcirLean`; like Rust's `::std::…`. |
| `{α : Type}` | A generic type parameter, like `<T>` in Rust. |
| `import X` | `use` another file. |

### Logic (only inside `Prop`s)

| Lean | Read it as |
|---|---|
| `∀ x, P x` | "for every x, P x holds" |
| `∀ x ∈ l, P x` | "for every x in list l, P x" |
| `∃ x, P x` | "there is some x such that P x" |
| `A → B` (between statements) | "if A then B" |
| `A ∧ B` | A and B |
| `A ∨ B` | A or B |
| `¬ A` | not A |
| `a ≠ b` | a is not equal to b |
| `decide (P)` | Turns a checkable statement into a `Bool` |

### Functional-programming idioms

| Lean | Meaning |
|---|---|
| `l.map f` | Apply `f` to every element (like `iter().map`). |
| `l.sum`, `l.prod` | Sum or product of a list. |
| `l.zip m` | Pair up elements. |
| `l.lookup k` | Find the value paired with key `k` in a list of pairs. |
| `fun x => e` | A lambda, like `\|x\| e`. |
| `l.foldlM f init` | A left fold that stops at the first `none`: run `f` step by step and fail if any step fails. |
| `do let x ← e; …` | Inside `Option`: "evaluate `e`; if it's `none`, the whole thing is `none`, otherwise call the result `x` and continue". Exactly Rust's `?` operator. |
| `s!"v{d} = {a}"` | String interpolation, like `format!`. |

One more thing about proofs. You will see `by norm_num` or `by decide` in two places inside Spec. Those are tiny proofs that Lean checks itself. You don't need to read them.

---

## Part 1 — `Semantics.lean`: what an ACIR opcode means (~75 lines)

This is the most fundamental file. Everything else is built on it.

```lean
def p : ℕ := 21888242871839275222246405745257275088548364400416034343698204186575808495617
```

This is the BN254 scalar field modulus, the same number as Rust's `FieldElement::modulus()`.
**Check:** that it's the right prime. A typo here would silently change the whole field.

```lean
instance : NeZero p := ⟨by norm_num [p]⟩
instance : Fact (1 < p) := ⟨by norm_num [p]⟩
```

These two lines are boilerplate: "p isn't zero" and "p is bigger than 1", each with a one-word proof that Lean checks. Lean's library needs them to do arithmetic mod p. Nothing to review.

```lean
abbrev F := ZMod p
```

`F` means "integers mod p": the field ACIR works in. `ZMod p` comes from Mathlib, Lean's standard math library, which is trusted like a standard library. `x.val` gives the representative in `[0, p)`.

```lean
def Range (x : F) (k : ℕ) : Prop := x.val < 2 ^ k
```

The meaning of ACIR's `RANGE { num_bits: k }` on a witness with value `x`: "x, as an ordinary number, is below 2^k".
**Check:** that this is what the proving backend (bb) enforces for a range constraint. It is.

```lean
structure Term where
  coef : ℤ
  witnesses : List ℕ
```

One term of a polynomial: a coefficient times a product of witnesses. For example, `3 · w1 · w4` is `{ coef := 3, witnesses := [1, 4] }`. A term with `witnesses = []` is a constant.

```lean
abbrev Expression := List Term
```

An `Expression` is a polynomial: the sum of its terms. It's the same thing as Rust's `Expression`, which stores the same polynomial split into `mul_terms`, `linear_combinations` and the constant `q_c`. The Lean version is simply a flat list of terms.

```lean
inductive Opcode where
  | assertZero (expr : Expression)
  | range (witness numBits : ℕ)
```

An `Opcode` (Rust's `Opcode`) is one of two kinds:

- `assertZero expr`: ACIR's `AssertZero`. The expression must equal 0.
- `range witness numBits`: ACIR's `RANGE` black box. The witness must fit in `numBits` bits.

These are the only two ACIR opcodes this PR models. A circuit with any other opcode (memory, other black boxes, calls) can't be written in this form, so it isn't covered.

```lean
def Term.eval (σ : ℕ → F) (t : Term) : F := (t.coef : F) * (t.witnesses.map σ).prod
```

The value of a term under the witness assignment `σ` is the coefficient times the product of the witness values, all in F. `σ i` is the value the prover put in witness `i`.

```lean
def Opcode.Holds (σ : ℕ → F) : Opcode → Prop
  | .assertZero expr => (expr.map (Term.eval σ)).sum = 0
  | .range witness numBits => Range (σ witness) numBits
```

**This is the heart of the whole thing: when an opcode holds.**

- An `AssertZero` holds when its terms add up to 0 mod p.
- A `RANGE` holds when the witness fits in `numBits` bits.

**Check:** that this is exactly what ACIR means and what bb enforces. It is. The companion bb work (PR 651) proves bb's side against this same meaning.

```lean
def AllHold (σ : ℕ → F) (opcodes : List Opcode) : Prop :=
  ∀ c ∈ opcodes, c.Holds σ
```

The prover's values satisfy the whole circuit when every opcode holds.

```lean
structure Circuit where
  opcodes : List Opcode
  parameters : List ℕ
  returnValues : List ℕ
```

A `Circuit` (Rust's `Circuit`) has three parts:

- its opcodes;
- the witnesses that hold its parameters. Rust keeps private and public parameters in two lists; here they're merged, in witness order;
- the witnesses that hold its return values.

---

## Part 2 — `Ssa.lean`: a tiny SSA language (~93 lines)

This file is used by exactly one claim: that the SSA `expand_signed_math` produces for signed `<` is correct. It defines a mini-SSA with just `cast`, `div`, `lt` and `xor`.

```lean
inductive IntType where
  | u (n : ℕ)
  | i (n : ℕ)
```

A type: `u8`, `i32` and so on.

```lean
inductive SsaOperand where
  | var (id : ℕ)
  | const (ty : IntType) (v : ℕ)
```

An operand is either `v12` or a constant like `u8 128`.

```lean
inductive SsaInstruction where
  | cast (dst src : ℕ) (ty : IntType)
  | bin (dst : ℕ) (op : SsaBinOp) (a b : SsaOperand)
```

An instruction is `v{dst} = cast v{src} as ty` or `v{dst} = op a, b`.

```lean
def SsaBinOp.eval : SsaBinOp → ℕ → ℕ → ℕ
  | .div, a, b => a / b
  | .lt, a, b => if a < b then 1 else 0
  | .xor, a, b => a ^^^ b
```

What each operation computes, on plain numbers:

- `div` is integer division;
- `lt` is 1 or 0;
- `xor` is bitwise xor.

Values here are bit patterns, so an `i8` holding -1 is the number 255.
**Check:** that these match Noir's SSA on the unsigned values it sees after `expand_signed_math`.

```lean
def SsaInstruction.step (env : ℕ → ℕ) : SsaInstruction → (ℕ → ℕ)
  | .cast dst src _ => fun i => if i = dst then env src else env i
  | .bin dst op a b => fun i => if i = dst then op.eval (a.eval env) (b.eval env) else env i
```

Running one instruction updates the variable `dst` and leaves the others alone (`SsaOperand.eval` reads an operand: a variable's value, or the constant). `cast` keeps the bit pattern unchanged, which is how Noir treats it.

```lean
def SsaFunction.run (f : SsaFunction) (args : List ℕ) : ℕ := …
```

Put the arguments in the parameters, run every instruction in order, and return `v{ret}`.

The rest of the file (`IntType.render`, `SsaOperand.render`, `SsaBinOp.render`, `SsaInstruction.render`, `SsaFunction.render`) prints this mini-SSA as text in Noir's own SSA syntax. That's how it gets compared with what the compiler really emits (see `Pin.lean`).
**Check:** that the printed format matches Noir's `Display`. If it didn't, the comparison with the compiler would fail, so a mistake here shows up as a CI failure rather than a silent error.

---

## Part 3 — `Corpus.lean`: small `div`/`lt` programs (~70 lines)

This file is for the 66-program "corpus": straight-line programs of `u<n>` division and comparison.

```lean
inductive CorpusOp where
  | div
  | lt

structure CorpusInstruction where
  op : CorpusOp
  a : ℕ
  b : ℕ

structure CorpusProgram where
  width : ℕ
  nparams : ℕ
  body : List CorpusInstruction
  ret : ℕ
```

A program has:

- a bit width (8, 64 or 128);
- a number of parameters;
- a list of instructions, each saying "apply `op` to value `a` and value `b`";
- the index of the value to return.

Values are numbered in order: parameters first, then each instruction's result.

```lean
def CorpusProgram.eval (P : CorpusProgram) (ins : List ℕ) : Option ℕ :=
  …
      | .div => if y = 0 then none else some (vs ++ [x / y])
      | .lt => some (vs ++ [if x < y then 1 else 0])
```

Run the program. `div` by zero **fails** (`none`); otherwise it's integer division. `lt` gives 1 or 0.

```lean
def CorpusSpec (P : CorpusProgram) : List ℕ → List ℕ → Prop := fun ins outs =>
  ins.length = P.nparams ∧ (∀ x ∈ ins, x < 2 ^ P.width) ∧
    ∃ v, P.eval ins = some v ∧ outs = [v]
```

**This is the promise for these programs.** Given the circuit's input values `ins` and output values `outs`, all of these must hold:

1. there is one input per parameter;
2. every input fits in `width` bits (so the circuit enforces the parameter types);
3. the program runs without failing on those inputs, and the output equals its result.

Point 3 also means that if the program would fail (for example, divide by zero), no set of values can satisfy the circuit. The circuit must reject that input.

```lean
structure CorpusEntry where
  prog : CorpusProgram
  fn : Circuit
  witness : List (ℕ × ℕ)
```

Each corpus entry is a program, the circuit the compiler shipped for it, and the witness values ACVM actually computed. `CorpusEntry.assignment` turns that list into a witness assignment (unlisted witnesses are 0); it's used to show the circuit isn't contradictory.

---

## Part 4 — `SsaSemantics.lean`: what real SSA programs mean (~190 lines)

**This is the file most worth a careful read.** It defines the meaning of the SSA for the real test programs. If an instruction's meaning here is wrong, the proof proves the wrong thing.

### The syntax

```lean
inductive ValueType where
  | field
  | uint (n : ℕ)
  | sint (n : ℕ)
```

A scalar's type: `Field`, `u<n>` or `i<n>`.

```lean
inductive ParamType where
  | scalar (t : ValueType)
  | array (elems : List ValueType) (len : ℕ)
```

A parameter's type: a scalar, or an array. `array [u8] 5` is `[u8; 5]`, and `array [Field, u8] 3` is an array of three tuples, `[(Field, u8); 3]`. Arrays of arrays aren't covered; the data generator refuses them.

```lean
inductive Operand where
  | var (id : ℕ)
  | const (v : ℕ) (ty : ValueType)
```

An operand: a variable `v12`, or a constant like `u32 7`. Constants are natural numbers only; the data generator refuses negative constants.

```lean
inductive BinaryOp where
  | add | sub | mul | div | mod | lt | eq
```

The binary operations covered.

```lean
inductive Instruction where
  | bin (dst : ℕ) (op : BinaryOp) (unchecked : Bool) (a b : Operand)
  | not (dst : ℕ) (a : Operand)
  | cast (dst : ℕ) (a : Operand) (ty : ValueType)
  | truncate (dst : ℕ) (a : Operand) (bits maxBits : ℕ)
  | constrain (a b : Operand) (msg : Option String)
  | rangeCheck (a : Operand) (bits : ℕ) (msg : Option String)
  | arrayGet (dst : ℕ) (a i : Operand) (ty : ValueType)
  | arraySet (dst : ℕ) (isMut : Bool) (a i v : Operand)
  | makeArray (dst : ℕ) (elems : List Operand) (ty : ParamType)
```

The nine kinds of instruction. Each has a doc comment showing its SSA text, for example ``v3 = unchecked_add v1, v2``.

```lean
structure Program where
  header : String
  params : List (ℕ × ParamType)
  body : List Instruction
  rets : List Operand
```

A function has a header line, typed parameters, a body, and the returned operands.

### The printer (lines ~60–96)

`ValueType.render`, `ParamType.render`, `Operand.render`, `BinaryOp.name`, `msgSuffix`, `Instruction.render` and `Program.render` print the program back as SSA text, for example `"    v{d} = truncate {a} to {k} bits, max_bit_size: {m}"`. CI compares that text with what `nargo compile` actually printed, character by character.
**Check:** only that the printer is faithful. If a field were printed but ignored by the meaning below, a real difference could slip past. (Every field here is used.)

### The meaning

```lean
def ValueType.fits : ValueType → F → Bool
  | .field, _ => true
  | .uint n, x => decide (x.val < 2 ^ n)
  | .sint n, x => decide (x.val < 2 ^ n)
```

When a value "fits" its type:

- any field element fits `Field`;
- a `u<n>` or `i<n>` value must be below 2^n. Signed values are stored as bit patterns, so -1 in `i8` is 255.

```lean
def ParamType.flat : ParamType → List ValueType
  | .scalar t => [t]
  | .array ts n => (List.replicate n ts).flatten
```

The types of a parameter's scalars, in order. `[(Field, u8); 2]` flattens to `Field, u8, Field, u8`. This is also how SSA numbers an array's positions: element `i`'s field `j` of a `k`-field tuple is at `i * k + j`.

```lean
inductive Value where
  | scalar (v : F × ValueType)
  | array (xs : List (F × ValueType))

abbrev Env := List (ℕ × Value)
```

A value is a scalar (a field element and its type) or an array (its scalars in that flat order). The program's state is a list of `(variable id, value)`.

```lean
def Operand.value (env : Env) : Operand → Option (F × ValueType)
```

Reading a scalar operand: look the variable up (it must hold a scalar), or use the constant. `Operand.array` reads an array operand the same way, and `Operand.flat` reads either kind as a list of field elements, which is what a returned value becomes in the circuit.

```lean
def arrayIndex (i : F × ValueType) (len : ℕ) : Option ℕ :=
  if i.2 = .uint 32 ∧ i.1.val < len then some i.1.val else none
```

An array index must be a `u32` below the array's length, as in Noir's interpreter (which reads it with `as_u32` and fails past the end).

```lean
def flag (b : Bool) : F × ValueType := (if b then 1 else 0, .uint 1)
```

A boolean result becomes a `u1` that is 1 or 0.

```lean
def fieldArith : BinaryOp → F → F → Option F
```

`add`, `sub` and `mul` done in the field (mod p); `none` for any other operation.

```lean
def lowBits (n : ℕ) (x : F) : ℕ := x.val % 2 ^ n
```

The low n bits of a value. Noir's interpreter reduces integer operands this way (`truncate_field`) before dividing or comparing them, so a value that has grown past its type's width (after an unchecked operation) is read by its low bits.

```lean
def u1Apply (op : BinaryOp) (unchecked x y : Bool) : Option Bool :=
  | .add => if !unchecked && x && y then none else some (x ^^ y)
  | .sub => if !x && y then none else some (x ^^ y)
  | .mul => some (x && y)
  | .div => if y then some x else none
  | .mod => if y then some false else none
  | .lt => some (!x && y)
  | .eq => some (x == y)
```

Operations on `u1` values, treated as booleans, exactly as Noir's `interpret_u1_binary_op` does. `^^` is xor, `&&` and, `!` not. Note that unchecked `add` is xor (so `1 + 1` gives 0), checked `add` fails on `1 + 1`, and `sub` fails on `0 - 1` whether checked or not.

```lean
def BinaryOp.apply (op : BinaryOp) (unchecked : Bool) (x y : F) :
    ValueType → Option (F × ValueType)
```

**This is the key definition: what each binary operation does**, following Noir's SSA interpreter for ACIR functions (`evaluate_binary`). It is organised by the operand's type.

- **`Field`:**
  - `add`, `sub` and `mul` are field arithmetic (wrap mod p);
  - `div` multiplies by the inverse (`x * y⁻¹`) and fails if `y` is 0;
  - `lt` compares the integer values, and `eq` tests equality (1 or 0);
  - `mod` isn't defined for `Field` in Noir either, so it fails.
- **`u1`:** both values must be 0 or 1 (Noir's interpreter asserts it), then `u1Apply` above.
- **Other `u<n>`:**
  - **Unchecked `add`, `sub`, `mul`** (`unchecked_add` and friends) are field arithmetic: they never fail, and the result keeps the type even if it no longer fits (a later `truncate` brings it back).
  - **Checked `add`, `sub`, `mul`** compute the same field result and fail unless it fits in n bits. So `sub` fails when `y > x`, and `add`/`mul` fail on overflow. A checked `u128` `mul` also fails when the product of the two values reaches `2^128`, a check Noir adds because that product could otherwise wrap around p and land back in range.
  - **`div` and `mod`** use the operands' low n bits (`lowBits`) and fail on a zero divisor.
  - **`lt` and `eq`** compare the low n bits and give 1 or 0.
- **`i<n>`:** unchecked `add`/`sub`/`mul` (field arithmetic, as above) and `eq` on the low n bits. Noir's interpreter also defines signed checked arithmetic, `div`, `mod` and `lt`, but the `expand_signed_math` pass rewrites all of them into unsigned operations before the SSA reaches ACIR, so this definition leaves them out: a program using them would fail here, and CI would report it.

`fieldArith` above is just the `add`/`sub`/`mul` part, shared by the integer cases.

**Check this definition against Noir's interpreter** (`evaluate_binary`, `interpret_u1_binary_op`, `evaluate_integer_binary` and `eval_constant_binary_op` in `compiler/noirc_evaluator/src/ssa/`). It should match case by case.

```lean
def Instruction.run (env : Env) : Instruction → Option Env
```

Running one instruction:

- **`bin`:** read both operands, apply the operation, and store the result as `v{dst}`. It fails if the operation fails.
- **`not`:** on an n-bit integer (`u<n>` or `i<n>`), reduce the value to its low n bits and flip them: `2^n − 1 − (x mod 2^n)`. On `Field` it fails, and so does a `u1` above 1, as in Noir (whose interpreter asserts a `u1` is 0 or 1).
- **`cast`:** keep the value and change the type, without checking that it fits, exactly like Noir. A value that doesn't fit its new type is later brought into range by a `truncate`.
- **`truncate`:** keep the low `bits` bits: `x mod 2^bits`, so truncating to 0 bits gives 0. Otherwise, like Noir, it fails for a `u1` above 1.
- **`constrain a == b`:** fail unless `a = b`.
- **`range_check a to k bits`:** fail unless `a < 2^k`. Like Noir, it also fails for 0 bits and for a `u1` above 1.
- **`array_get a, index i`:** read the scalar at position `i`; fail if `arrayIndex` does.
- **`array_set a, index i, value v`:** a copy of `a` with position `i` replaced by `v`; fail if `arrayIndex` does. In an ACIR function arrays are values, so `mut` doesn't change the result.
- **`make_array [..]`:** the array of the listed scalars.

**Check each against Noir's SSA interpreter.** In particular, `not`, `truncate` and the overflow rules. You don't have to do this alone: `EmitSemantics.lean` runs `Instruction.run` on a grid of edge-case values for every instruction and type, and the Rust test `fv_semantics.rs` fails unless Noir's interpreter gives the same result on every one (see "How the SSA meaning stays attached to Noir" in `README.md`). What the test cannot tell you is whether the grid is wide enough, so glance at `values` in `EmitSemantics.lean` too.

```lean
def bindParams : List (ℕ × ParamType) → List F → Env
def Program.inputTypes (P : Program) : List ValueType := P.params.flatMap (·.2.flat)
def Program.eval (P : Program) (ins : List F) : Option (List F) := do
  let env ← P.body.foldlM Instruction.run (bindParams P.params ins)
  let outs ← P.rets.mapM (·.flat env)
  some outs.flatten
```

Running a whole program:

1. bind the parameters to the inputs (`bindParams`): each parameter takes as many inputs as it has scalars, in order, so an array parameter `[u8; 3]` takes the next three;
2. run the instructions in order, stopping as soon as one fails;
3. read the return values, flattened the same way.

`inputTypes` lists the scalar type of every input, in the same order.

```lean
def ProgramSpec (P : Program) : List ℕ → List ℕ → Prop := fun ins outs =>
  ins.length = P.inputTypes.length ∧
    (∀ e ∈ P.inputTypes.zip ins, e.1.fits (e.2 : F) = true) ∧
    ∃ vs, P.eval (ins.map fun x => (x : F)) = some vs ∧ outs = vs.map ZMod.val
```

**The promise for the real test programs.** For the circuit's input values and output values:

1. there is one input per parameter scalar (an array parameter has one input per element);
2. every input fits its scalar's type;
3. the program runs to the end without failing (no overflow, no zero divisor, no failed `constrain`), and the circuit's outputs equal the program's return values.

This is the same shape as `CorpusSpec`, just for richer programs.

```lean
structure TestProgram where
  name : String
  prog : Program
  fn : Circuit
  witness : List (ℕ × ℕ)

def TestProgram.assignment (e : TestProgram) (i : ℕ) : F :=
  ((e.witness.lookup i).getD 0 : ℕ)
```

A test program: its name, its SSA, its shipped circuit, and the witness `nargo execute` solved for it from its `Prover.toml`, as `(witness, value)` pairs. `TestProgram.assignment` turns that list into a value for every witness (`0` for any not listed), the same way `CorpusEntry.assignment` does for the corpus.

---

## Part 5 — `Pin.lean`: the printer that ties Lean to the compiler (~121 lines)

`Pin.lean` proves nothing. It prints things, so that CI can compare Lean's copy of the circuits with the compiler's.

```lean
def pinnedWidths : List ℕ := [8, 16, 32, 64, 128]
def signedWidths : List ℕ := [8, 16, 32, 64]
```

The bit widths the gadget claims cover. **Check:** that you're happy with this list, because the claims say "for every n in pinnedWidths", not "for every n".

```lean
def witnessListLe : List ℕ → List ℕ → Bool
```

Compares two lists of witness indices the way Rust sorts `Vec<u32>`. Used only to print terms in a fixed order.

```lean
def modP (c : ℤ) : ℤ := c % (p : ℤ)
def insertBy {α : Type} (le : α → α → Bool) (x : α) : List α → List α
def isort {α : Type} (le : α → α → Bool) : List α → List α
```

`modP` reduces a coefficient to `[0, p)`. `insertBy` and `isort` are a plain insertion sort.

```lean
def Opcode.canon : Opcode → Opcode
```

Puts one constraint in canonical form:

1. reduce each coefficient mod p and sort each term's witnesses;
2. drop zero terms;
3. sort the terms;
4. flip the overall sign if the first coefficient is in the upper half (an equation `= 0` means the same with all signs flipped).

You don't need to check by eye that this keeps the equation's meaning: `Opcode.canon_sat` in `Proofs/Canon.lean` proves that a constraint holds exactly when its canonical form does. Terms over the same witnesses are not merged; Rust merges them, so such a constraint would print differently on the two sides and fail the pin rather than pass wrongly.

```lean
def Opcode.render (c : Opcode) : String :=
```

Prints `c.canon`: `zero 1*[0] + 21888…616*[3]` for an `AssertZero`, or `range 5 8`. The Rust test prints the compiler's constraints the same way, so the two can be compared as text.
**Check:** that printing a canonical constraint is faithful: each coefficient and witness list is printed as is.

```lean
def Circuit.render … CorpusProgram.render … CorpusEntry.render … TestProgram.render
```

The same idea for whole circuits: one line per opcode, then `inputs [..]` (the parameter witnesses) and `returns [..]` (the return-value witnesses). The last ones print a program followed by its circuit.

```lean
def renderAll : String :=
def renderTestPrograms : String :=
```

These build the full text of the two golden files, `templates.golden` and `test_programs.golden`. The chain works like this:

- `check.sh` fails unless Lean's printout equals those files;
- a Rust test and the regeneration job fail unless the *compiler's* printout equals them;
- so Lean's copy and the compiler's output must match exactly.

---

## Part 6 — `Claims.lean`: the promise itself (~172 lines)

### The building blocks

```lean
def InputsFit (σ : ℕ → F) (inputs : List (ℕ × ℕ)) : Prop :=
  ∀ iw ∈ inputs, (σ iw.1).val < 2 ^ iw.2
```

"Each listed input witness fits its bit width". For example, `[(0, 8), (1, 8)]` means witnesses 0 and 1 are 8-bit.

```lean
def Sound (T : List Opcode) (inputs : List (ℕ × ℕ)) (spec : (ℕ → F) → Prop) : Prop :=
  ∀ σ : ℕ → F, AllHold σ T → InputsFit σ inputs → spec σ
```

**The definition of "not underconstrained", for a single gadget.** Read it as: for *every* possible set of prover values σ, if σ satisfies every constraint in `T`, and the inputs fit their types, then `spec` holds.

"For every σ" is what covers every cheating prover. There's no "assume the prover is honest".

Note that gadget claims *assume* their inputs fit, because inside a bigger circuit someone else range-checks them. Whole-function claims (`SoundFunction`, below) do not assume it; they prove it.

```lean
def Satisfiable (T : List Opcode) (inputs : List (ℕ × ℕ)) : Prop :=
  ∃ σ : ℕ → F, AllHold σ T ∧ InputsFit σ inputs
```

"*Some* σ satisfies everything". This guards against a vacuous claim: a contradictory circuit (say, one containing `1 = 0`) would make `Sound` trivially true. Proving `Satisfiable` shows that isn't the case. It is *not* completeness: it says one witness works, not that every valid input has one. A circuit that wrongly rejects some honest inputs can still satisfy every claim here.

```lean
def divSpec (σ : ℕ → F) : Prop :=
  (σ 3).val = (σ 0).val / (σ 1).val ∧ (σ 4).val = (σ 0).val % (σ 1).val

def divPredSpec (σ : ℕ → F) : Prop :=
  σ 2 = 1 → (σ 5).val = (σ 0).val / (σ 1).val ∧ (σ 6).val = (σ 0).val % (σ 1).val

def truncSpec (k : ℕ) (σ : ℕ → F) : Prop :=
  (σ 2).val = (σ 0).val % 2 ^ k

def geSpec (σ : ℕ → F) : Prop :=
  (σ 2).val = if (σ 1).val ≤ (σ 0).val then 1 else 0
```

What each gadget must compute, in terms of *witness numbers* inside that gadget. The doc comment above each says which witness is which, for example "0 = a, 1 = b, 3 = q, 4 = r". So `divSpec` says "q = a / b and r = a % b". `divPredSpec` says the same, but only when the predicate witness is 1.
**Check:** that the witness numbers in the comments match the gadget. The pin guarantees the constraint list is the real one, but which witness means "q" is a human reading.

```lean
def toSigned (n v : ℕ) : ℤ := if v < 2 ^ (n - 1) then v else (v : ℤ) - 2 ^ n
```

Reads an n-bit pattern as a signed number (two's complement). For example, `toSigned 8 255 = -1`.

```lean
def ComputesSignedLt (f : SsaFunction) (n : ℕ) : Prop :=
  ∀ a b : ℕ, a < 2 ^ n → b < 2 ^ n →
    f.run [a, b] = if toSigned n a < toSigned n b then 1 else 0
```

"For all n-bit patterns a and b, the SSA program returns 1 exactly when a < b as signed numbers". This is the claim about `expand_signed_math`.

```lean
def SoundFunction (f : Circuit) (spec : List ℕ → List ℕ → Prop) : Prop :=
  ∀ σ : ℕ → F, AllHold σ f.opcodes →
    spec (f.parameters.map fun i => (σ i).val) (f.returnValues.map fun i => (σ i).val)
```

**"Not underconstrained" for a whole compiled function.** For every σ that satisfies all its constraints, `spec` holds for the input values and the output values.

There is no input-type assumption here. The input types must be enforced by the circuit itself, and the specs check them.

```lean
def SatisfiableFunction (f : Circuit) : Prop := ∃ σ : ℕ → F, AllHold σ f.opcodes
```

The whole-function version of the non-vacuity check.

```lean
def Computes2 (n : ℕ) (g : ℕ → ℕ → ℕ) : List ℕ → List ℕ → Prop
  | [a, b], [r] => a < 2 ^ n ∧ b < 2 ^ n ∧ r = g a b
  | _, _ => False

def Computes1 (g : ℕ → ℕ) : List ℕ → List ℕ → Prop
  | [a], [r] => r = g a
  | _, _ => False
```

The spec for a two-input function: exactly two inputs, both n-bit, and exactly one output, equal to `g a b`. The `| _, _ => False` line means any other number of inputs or outputs fails the spec, so the circuit can't sneak in an extra input.

```lean
def DivOp (n : ℕ) : List ℕ → List ℕ → Prop
  | [a, b], [r] => a < 2 ^ n ∧ b < 2 ^ n ∧ b ≠ 0 ∧ r = a / b
  | _, _ => False
```

The spec for unsigned `/` on `u<n>`: two n-bit inputs, a divisor that isn't zero, and an output equal to the quotient. Noir fails on a zero divisor, so `b ≠ 0` here means a circuit that accepted `b = 0` (with any output) would break the claim. **Check:** that `b ≠ 0` is there.

```lean
def toBitPattern (n : ℕ) (x : ℤ) : ℕ := (x % 2 ^ n).toNat
def SignedOp (n : ℕ) (op : ℤ → ℤ → ℤ) : List ℕ → List ℕ → Prop
  | [a, b], [r] =>
    a < 2 ^ n ∧ b < 2 ^ n ∧ toSigned n b ≠ 0 ∧ ¬ (toSigned n a = -2 ^ (n - 1) ∧ toSigned n b = -1) ∧
      r = toBitPattern n (op (toSigned n a) (toSigned n b))
```

The spec for signed `/` and `%`. It requires:

- both inputs are n-bit;
- the divisor isn't zero;
- it isn't the overflowing `MIN / -1`;
- the output is the true signed result, written back as a bit pattern.

`op` will be `Int.tdiv` or `Int.tmod`: division that rounds toward zero, as Noir does. `AllClaims` writes them `_root_.Int.tdiv` and `_root_.Int.tmod`: `_root_.` means "the one at the top level", so no definition elsewhere in the project can stand in for Lean's.

```lean
def uncoveredPrograms : List String :=
  ["arithmetic_binary_operations", "array_eq", "global_consts", "regression_8519"]
```

The test programs deliberately left out of the claim, each with its reason in the comment above. **Check:** that the reasons are acceptable, and that the list doesn't grow silently in future PRs.

```lean
def testProgramNames : List String :=
```

This one lives in `Spec/Coverage.lean`: the names of the test programs in `testPrograms`, sorted, one per line. The program data itself is generated and unreviewed, so this list is what pins down *which* programs the claim is about. `AllClaims` requires the generated programs to be exactly these, so a program can only drop out of the claim by being deleted here, in a reviewed diff. The regeneration script reads this list too, and refuses to write anything if one of the listed programs no longer compiles or no longer fits the supported subset. **Check:** in a PR, that any name removed from this list was removed on purpose.

### `AllClaims`: the entire promise, one conjunction

Every line below is joined with `∧` ("and"). Read each one as a sentence.

| Lean | Plain English |
|---|---|
| `∀ n ∈ pinnedWidths, Sound (divVarGadget n) [(0, n), (1, n)] divSpec ∧ Satisfiable …` | For every width, the Euclidean-division constraints, with n-bit `a` and `b`, force `q = a/b` and `r = a%b`, and they're satisfiable. |
| `… Sound (divPredGadget n) … divPredSpec …` | The same for division under a predicate, when the predicate is on. |
| `∀ k ∈ pinnedWidths, Sound (truncateGadget k) [] (truncSpec k) …` | Truncation forces `r = x mod 2^k` for *any* field element `x`, with no input assumption. This is the gadget bug #7895 was in. |
| `… Sound (moreThanEqGadget m) … geSpec …` | The comparison gadget forces the result to be `[a ≥ b]`. |
| `∀ n ∈ pinnedWidths, ComputesSignedLt (signedLtSsa n) n` | The SSA `expand_signed_math` makes for signed `<` is correct. |
| `SoundFunction (acirGenDiv n) (DivOp n) ∧ SatisfiableFunction …` | The whole function `fn(a: u<n>, b: u<n>) -> a / b`, as ACIR generation compiles it, is correct, enforces the input types, and rejects a zero divisor. |
| `… acirGenLt … acirGenTruncate … acirGenSignedLt …` | The same for `lt`, truncation and signed `lt`. |
| `… shippedDiv … shippedLt … shippedTruncate … shippedSignedLt …` | The same four, **after the ACVM optimizer**, as `nargo compile` actually ships them. |
| `∀ n ∈ signedWidths, SoundFunction (shippedSignedDiv n) (SignedOp n _root_.Int.tdiv) …` and `…shippedSignedMod… _root_.Int.tmod` | Signed `/` and `%`, as shipped, are correct and reject a zero divisor and `MIN / -1`. |
| `∀ e ∈ corpus, SoundFunction e.fn (CorpusSpec e.prog) ∧ AllHold e.assignment e.fn.opcodes` | Every corpus program's shipped circuit implements it, and ACVM's real witness satisfies that circuit. |
| `testPrograms.map TestProgram.name = testProgramNames` | The generated test programs are exactly the ones the reviewed list names. |
| `∀ e ∈ testPrograms, e.name ∉ uncoveredPrograms → SoundFunction e.fn (ProgramSpec e.prog) ∧ AllHold e.assignment e.fn.opcodes` | Every real test program in `testPrograms` (except those in `uncoveredPrograms`) is implemented by its shipped circuit, and the witness `nargo execute` solved for it satisfies that circuit, so the first half can't hold just because the circuit is contradictory. |

Names like `divVarGadget n` and `shippedDiv n` refer to constraint lists in `Templates/`. Those aren't reviewed, because the pin makes them equal to the compiler's real output.

---

## Part 7 — `Check.lean`: the final gate (4 lines of code)

```lean
example : AcirLean.AllClaims := AcirLean.allClaims
```

"Here is a proof of `AllClaims`". `allClaims` is the big proof in `Proofs/`. Lean refuses to accept this line unless that proof really proves *exactly* the statement in `Claims.lean`. That's what stops the proofs from quietly proving something weaker.

```lean
/-- info: 'AcirLean.allClaims' depends on axioms: [propext, Classical.choice, Quot.sound] -/
#guard_msgs in
#print axioms AcirLean.allClaims
```

Lean prints every axiom the proof relies on, and `#guard_msgs` fails the build unless the list is exactly these three. They are Lean's standard axioms, used by essentially all of Mathlib. If anyone added a new axiom (a "trust me"), or used a shortcut like `sorry`, this line would fail.

---

## Review checklist: what to actually look for

1. **Semantics.** Are `p`, `Range`, `Opcode.Holds` and `AllHold` the true meaning of ACIR `AssertZero` and `RANGE`?
2. **SSA meaning** (`SsaSemantics.lean`, `Programs.lean`, `Ssa.lean`). Does each instruction mean what Noir means?
   - The only place Lean leaves something undefined that Noir defines is signed checked arithmetic, `div`, `mod` and `lt`, which never reach ACIR. Failing there is safe: it can only make a program unprovable, and CI says so.
   - Where it's *looser* or *different*, that's a bug to flag.
3. **Specs** (`Claims.lean`). Does each spec say what you'd want it to? Look for:
   - a missing condition;
   - an input assumption that shouldn't be there (only the gadget `Sound` claims assume input types);
   - wrong witness numbers.
4. **Scope.** Are `pinnedWidths`, `signedWidths` and `uncoveredPrograms` acceptable? Did any name leave `testProgramNames`?
5. **Printer** (`Pin.lean` and the `render` functions). Is it faithful? If it prints the same text for two different things, the pin could be fooled.

Everything outside this list is either checked by Lean (`Proofs/`) or compared byte-for-byte with the compiler (`Templates/`).
