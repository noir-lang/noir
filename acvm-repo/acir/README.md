# The Abstract Circuit Intermediate Representation (ACIR)

The purpose of ACIR is to make the link between a generic proving system, such
as Aztec's Barretenberg, and a frontend, such as Noir, which describes user
specific computations.

More precisely, Noir is a programming language for zero-knowledge proofs (ZKP)
which allows users to write programs in an intuitive way using a high-level
language close to Rust syntax. Noir is able to generate a proof of execution of
a Noir program, using an external proving system. However, proving systems use
specific low-level constrain-based languages. Similarly, frontends have their
own internal representation in order to represent user programs.

The goal of ACIR is to provide a generic open-source intermediate
representation close to proving system 'languages', but agnostic to a specific
proving system, that can be used both by proving system as well as a target for
frontends. So, at the end of the day, an ACIR program is just another
representation of a program, dedicated to proving systems.

## ACIR
ACIR stands for Abstract Circuit Intermediate Representation:
- **abstract circuit**: circuits are a simple computation model where basic
    computation units, named gates, are connected with wires. Data flows
    through the wires while gates compute output wires based on their input.
    More formally, they are directed acyclic graphs (DAG) where the vertices
    are the gates and the edges are the wires. Due to the immutability nature
    of the wires (their value does not change during an execution), they are
    well suited for describing computations for ZKPs. Furthermore, we do not
    lose any expressiveness when using a circuit as it is well known that any
    bounded computation can be translated into an arithmetic circuit (i.e a
    circuit with only addition and multiplication gates).
    The term abstract here simply means that we do not refer to an actual physical
    circuit (such as an electronic circuit). Furthermore, we will not exactly use
    the circuit model, but another model even better suited to ZKPs, the constraint
    model (see below).
- **intermediate representation**: The ACIR representation is intermediate
  because it lies between a frontend and its proving system. ACIR bytecode makes
  the link between noir compiler output and the proving system backend input.

## The constraint model

The first step for generating a proof that a specific program was executed, is
to execute this program. Since the proving system is going to handle ACIR
programs, we need in fact to execute an ACIR program, using the user-supplied
inputs.

In ACIR terminology, the gates are called opcodes and the wires are called
partial witnesses. However, instead of connecting the opcodes together through
wires, we create constraints: an opcode constraints together a set of wires.
This constraint model trivially supersedes the circuit model. For instance, an
addition gate `output_wire = input_wire_1 + input_wire_2` can be expressed with
the following arithmetic constraint:
`output_wire - (input_wire_1 + input_wire_2) = 0`

## Solving

Because of these constraints, executing an ACIR program is called solving the
witnesses. From the witnesses representing the inputs of the program, whose
values are supplied by the user, we find out what the other witnesses should be
by executing/solving the constraints one-by-one in the order they were defined.

For instance, if `input_wire_1` and `input_wire_2` values are supplied as `3` and
`8`, then we can solve the opcode
`output_wire - (input_wire_1 + input_wire_2) = 0` by saying that `output_wire` is
`11`.

In summary, the workflow is the following:
1. user program -> (compilation) ACIR, a list of opcodes which constrain
    (partial) witnesses
2. user inputs + ACIR -> (execution/solving) assign values to all the
    (partial) witnesses
3. witness assignment + ACIR -> (proving system) proof

Although the ordering of opcode does not matter in theory, since a system of
equations is not dependent on its ordering, in practice it matters a lot for the
solving (i.e the performance of the execution). ACIR opcodes **must be ordered**
so that each opcode can be resolved one after the other.

The values of the witnesses lie in the scalar field of the proving system. We
will refer to it as `FieldElement` or ACIR field. The proving system needs the
values of all the partial witnesses and all the constraints in order to generate
a proof.

_Note_: The value of a partial witness is unique and fixed throughout a program
    execution, although in some rare cases, multiple values are possible for a
    same execution and witness (when there are several valid solutions to the
    constraints). Having multiple possible values for a witness may indicate that
    the circuit is not safe.

_Note_: Why do we use the term partial witnesses? It is because the proving
    system may create other constraints and witnesses (especially with
    `BlackBoxFuncCall`, see below). A proof refers to a full witness assignment
    and its constraints. ACIR opcodes and their partial witnesses are still an
    intermediate representation before getting the full list of constraints and
    witnesses. For the sake of simplicity, we will refer to witness instead of
    partial witness from now on.

_Note_: Opcodes operate on witnesses, but we will see that some opcodes work on
    expressions of witnesses. We call an expression a linear combination of
    witnesses and/or products of two witnesses (also with a constant term). A
    single witness is a (simple) expression, and conversely, an expression can
    be turned into a single witness using an assert-zero opcode.
    So basically, using witnesses or expressions is equivalent,
    but the latter can avoid the creation of witness in some cases.

## Documentation

For detailed documentation, visit <https://noir-lang.github.io/noir/docs/acir/index.html>.