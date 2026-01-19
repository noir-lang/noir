# High-Level Architecture of the Noir Compiler

Noir's compiler is set up as a pipeline of compiler passes where each pass produces output
that is fed as the input to the next pass. The compiler is split up into a few key crates, the
first of which is [compiler/noirc_frontend](/compiler/noirc_frontend) which holds the frontend of the compiler (all passes before
SSA in the list below) which checks a program is valid and performs various transforms on it.
This crate also contains the comptime interpreter which executes `comptime` code at compile-time.
Next, there is [compiler/noirc_evaluator](/compiler/noirc_evaluator) which contains the various SSA optimizations as well
as ACIR and Brillig translations for constrained and unconstrained code respectively. Finally,
the last key crate is [tooling/nargo_cli](/tooling/nargo_cli) which invokes the compiler through the `check_crate` method.

Here's a rough diagram of the compiler's architecture:

```mermaid
flowchart
  source[Source Program]
  ast_and_definitions[AST & Definitions]
  monomorphized_ast[Monomorphized AST]
  ownership_ast[Monomorphized AST with Clones & Drops]
  optimized_ssa[Optimized SSA]
  brillig[Brillig IR]

  source --Lexing--> Tokens --Parsing--> AST --Definition Collection--> ast_and_definitions --Elaboration--> HIR

  HIR --Monomorphization--> monomorphized_ast --Ownership Analysis--> ownership_ast --SSA-gen--> SSA

  SSA --SSA Optimizations--> optimized_ssa

  optimized_ssa --Brillig-gen--> brillig
  optimized_ssa --ACIR-gen--> ACIR
```

Below is a summary of each compiler pass and which Rust source files may be most useful in
understanding the pass.

## Lexing

Lexing is a straightforward pass which converts a single source file or stream of text into a vector
of tokens. A token can be a single word, number, or operator. This pass can be found in
[compiler/noirc_frontend/src/lexer/lexer.rs](/compiler/noirc_frontend/src/lexer/lexer.rs).

## Parsing

Parsing converts the vector of tokens from the lexer into an abstract syntax tree (the
`ParsedModule`). Noir uses a hand-written recursive descent parser for better error recovery and
error messages. The parser can be found in [compiler/noirc_frontend/src/parser/parser.rs](/compiler/noirc_frontend/src/parser/parser.rs).

Note that the resulting AST is generally expected to maintain most information about how the source
program is structured so that the formatter can use this to format files. The main exception to this
is comments which must be recovered with additional help from the lexer.

## Definition Collection

Definition Collection is a short step where any top-level definitions are registered for future use
during elaboration. This pass can be found in [compiler/noirc_frontend/src/hir/def_collector/dc_mod.rs](/compiler/noirc_frontend/src/hir/def_collector/dc_mod.rs)
and [compiler/noirc_frontend/src/hir/def_collector/dc_crate.rs](/compiler/noirc_frontend/src/hir/def_collector/dc_crate.rs).

## The Elaborator

The elaborator is the largest pass in the frontend because it performs multiple tasks simultaneously.
It traverses the AST performing name resolution and type checking of each node in tandem (e.g. we
perform both for one statement before moving on to the next). This is to support the third task the
elaborator performs which is executing comptime code. Comptime code must be resolved and type
checked before it is executed, but may also output more code which needs to be resolved and type
checked, so it is included as part of elaboration. The elaborator outputs `Hir` nodes which are
stored within the `NodeInterner`. The elaborator can be found in [compiler/noirc_frontend/src/elaborator/mod.rs](/compiler/noirc_frontend/src/elaborator/mod.rs).

Comptime code is executed by the comptime interpreter which notably can call back into the
elaborator to resolve more code to support comptime noir functions like `Expr::resolve`.
The comptime interpreter can be found in [compiler/noirc_frontend/src/hir/comptime/interpreter.rs](/compiler/noirc_frontend/src/hir/comptime/interpreter.rs).

## Monomorphization

After elaboration, `Hir` nodes are monomorphized to make code monomorphic - creating a copy of each
function for each used combination of its generic arguments. At this point, the program still may not
be completely error-free. Several error checks are still performed during monomorphization and during the SSA passes.
The monomorphizer outputs the monomorphized Ast and can be found in [compiler/noirc_frontend/src/monomorphization/mod.rs](/compiler/noirc_frontend/src/monomorphization/mod.rs).

## Ownership Analysis

Ownership analysis is a pass performed on the monomorphized Ast (only modifying the same Ast) which
analyzes how variables are used to determine where to insert Clones and Drops. The pass itself as well
as the details on where these exactly clones and drops are inserted can be found in [compiler/noirc_frontend/src/ownership/mod.rs](/compiler/noirc_frontend/src/ownership/mod.rs).

### Last Use Analysis

Part of the ownership analysis is a sub-pass performed on each function before the rest of the ownership pass.
The purpose of this last use analysis pass is to find the last use of each variable so that the ownership
later knows where it can move values or where it needs to clone them. This pass can be found in
[compiler/noirc_frontend/src/ownership/last_uses.rs](/compiler/noirc_frontend/src/ownership/last_uses.rs).

## SSA

SSA marks the entry point of the separate evaluator crate. Noir's SSA (single-static assignment) IR
is the IR the majority of its optimizations are performed on during various SSA optimization passes.

The definition of the SSA data structures are spread across [compiler/noirc_evaluator/src/ssa/ir/](/compiler/noirc_evaluator/src/ssa/ir/)
but an important one to point out is the `DataFlowGraph` in [compiler/noirc_evaluator/src/ssa/ir/dfg.rs](/compiler/noirc_evaluator/src/ssa/ir/dfg.rs)
which is what holds the IR for a single function and is the most common structure operated on in each
optimization pass.

### SSA-gen

This pass generates an initial unoptimized SSA representation of the input program from the
monomorphized AST. The translation is largely straightforward but one notable difference between the
monomorphized AST and the SSA IR is that the later does not include tuples or any non-array aggregate
value. To remedy this, the SSA-gen pass translates any tuples into multiple function arguments or returns.
It can be found in [compiler/noirc_evaluator/src/ssa/ssa_gen/mod.rs](/compiler/noirc_evaluator/src/ssa/ssa_gen/mod.rs).

### SSA Optimization Passes

There are many optimization passes on Noir's SSA. The ordering of these passes is not stable and may
change over time although this should not be observable without debugging utilities like `--show-ssa`.
The various SSA passes can be found in [compiler/noirc_evaluator/src/ssa/opt/](/compiler/noirc_evaluator/src/ssa/opt/) and their ordering
can be found in the `optimize_all` function in [compiler/noirc_evaluator/src/ssa.rs](/compiler/noirc_evaluator/src/ssa.rs).

Note that various SSA passes may have constraints on when they can be performed. For example the
dead instruction elimination pass (DIE) may only be performed after the flatten-cfg pass. Some
SSA passes are also non-optional. The most relevant example is the flatten-cfg pass which removes
any remaining control flow from the program since ACIR (Noir's target for constrained code) does
not have any control flow mechanisms.

## Brillig

Unconstrained SSA functions get converted into Brillig IR. This is a register based IR which
resembles a classical programming language IR much more than Noir's constrained IR for constrained
programs: ACIR. Note that brillig code can be embedded in ACIR code since Noir programs can contain
both constrained and unconstrained functions.
In addition to classical integer arithmetic, and because they are native to the Noir language, Brillig IR support also arithmetic operations in a given prime field (see ACIR-gen below).

### Brillig-gen

Brillig-gen generates brillig IR from SSA code. The input SSA code must have undergone defunctionalization
as brillig does not support first-class functions, but otherwise requires no other optimizations.
The bulk of this pass can be found in [compiler/noirc_evaluator/src/brillig/brillig_gen/brillig_block.rs](/compiler/noirc_evaluator/src/brillig/brillig_gen/brillig_block.rs).

### Brillig VM

Brillig code is executed by the brillig VM. It can be found in [acvm-repo/brillig_vm/src/lib.rs](/acvm-repo/brillig_vm/src/lib.rs).
The Brillig VM is a register-based VM, using infinite virtual registers, i.e registers are simply allocated
on the VM memory, which removes the need for register allocation and their associated bookkeeping across function calls.
Therefore function calls are handled by simply pushing the return address on the callstack, jumping to the function entry point and handling the stack pointer for memory isolation.
The Brillig VM can also support foreign function calls which are handled externally, allowing to connect to any external service such as a database or a web application.

## ACIR-gen

ACIR-gen generates Noir's backend-agnostic output format for circuits: ACIR. This has a number of requirements
for the input SSA. Namely, all functions must be inlined, all loops must be unrolled, all remaining control-flow
(if, match) must be removed, and all references must be removed. As such it is expected by the end of all
SSA optimizations we end up with a single large block of SSA containing the entire program, and the ACIR cannot
be generated any earlier.

The main goal of the ACIR generation is to transform operations written with standard integer arithmetic (e.g. u32, i8, etc..) into operations within a specific prime field. The prime field is defined in `acvm-repo/acir_field/` and is expected to be the scalar field of the proving system. Most of the remaining SSA instructions can be directly converted to ACIR opcodes using mainly arithmetic and range-check opcodes.
Some dedicated operations, called black box functions, are deferred to the proving system for more efficient proving.
Similarly, unresolved array operations are deferred to the proving system using specific `Memory` opcodes.
ACIR specification can be found in `acvm-repo/acir/`

Further transformations of the ACIR bytecode are performed in `acvm-repo/acvm/src/compiler/mod.rs`.
The transformations are mainly optimizations, except for the `csat` transformation which helps to convert
the ACIR bytecode to satisfy the `width` required by the proving system.

## Execution
The compilation outputs a json file under the `target` directory in the project directory. It contains:
- the ABI, i.e the inputs and outputs of the program and their types
- the ACIR bytecode, including any Brillig bytecode for unconstrained functions
- various additional debug information

The program must be executed before a proof can be generated. The execution requires a `prover.toml` in the project directory which contains the program input values, matching the ABI.
Execution will generate a compressed `.gz` file containing the witness values of all the ACIR opcodes.
It takes the input values and feed them to the ACIR opcodes, one by one, deducing the witness values of each opcode outputs.


## Backend: Proving, and Verifying
Through the `target` artifacts, Noir provides a generic low-level constraint system (the ACIR bytecode in json format) and the corresponding witness assignments (the `.gz` file) so that a compatible backend can generate a proof.
Proof generation and verification is the responsibility of the backend.

# Tooling

## Nargo CLI

The [tooling/nargo_cli](/tooling/nargo_cli) crate provides the `nargo` binary and performs any command-line argument handling
necessary before calling into the [compiler/noirc_driver](/compiler/noirc_driver) crate to invoke the compiler.

## Noir Language Server

The [tooling/lsp](/tooling/lsp) crate provides the implementation of the `nargo lsp` command. This command implements
the server side of the [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
(LSP) and is invoked by the client side of the protocol
(for example by the [vscode-noir](https://github.com/noir-lang/vscode-noir) client).
LSP provides many features like completion, go-to-definition, hover information and inlay hints. Each
of these is implemented in a file inside [tooling/lsp/src/requests](/tooling/lsp/src/requests). Most of these features rely on
running the compiler frontend to get type information like available types and methods. Other features
just rely on parsing (for example listing all symbols in the current file).

## Formatter

The [tooling/nargo_fmt](/tooling/nargo_fmt) crate provides the implementation of the `nargo fmt` command. The formatter
takes an input string, parses it into an AST and, if there are no syntax errors, traverses the
AST together with the input string tokens (needed to be aware of whitespace and comments) to produce
a prettier output string.

## Debugger

The debugger for Noir code can be found in [tooling/debugger](/tooling/debugger). The debugger works by inserting extra
instrumentation in the form of function calls (see `compile_program_with_debug_instrumenter`) then
executing the resulting ACIR & Brillig code itself.

## Fuzzer

There are multiple fuzzers in the repo, focusing on different aspects of Noir.

### Input Fuzzer

The [tooling/fuzzer](/tooling/fuzzer) crate contains utilities to generate random `InputMap` according to a circuit ABI
and bytecode. It is used by `nargo test` to generate input for `#[test]` methods that have parameters.

### Greybox Fuzzer

The [tooling/greybox_fuzzer](/tooling/greybox_fuzzer) crate implements the `noir fuzz` machinery, which is an end-to-end solution
for fuzzing a Noir program, including generating random inputs and carefully mutating them to provide
optimal coverage of the circuit by discovering its control flow based on the changes in the output.

### SSA Fuzzer

The [tooling/ssa_fuzzer](/tooling/ssa_fuzzer) crate focuses on crafting SSA with arithmetic and logical operations,
asserting the equivalence of their execution through ACIR and Brillig. Unlike the _Greybox Fuzzer_,
it relies on `cargo fuzz` to drive the process.

### AST Fuzzer

The [tooling/ast_fuzzer](/tooling/ast_fuzzer) crate generates random monomorphized AST programs and performs comparative
testing by comparing execution results between:
1. ACIR and Brillig
2. different points in the SSA processing pipeline
3. the generated program and one that has a number of equivalence mutations applied to it

Like the _SSA Fuzzer_ it requires `cargo fuzz` to run.
