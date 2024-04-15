# Bytecode Validation Circuit

Goal: Validate that a polynomial commitment to AVM program opcodes maps to the bytecode representation of an AVM program of maximum size $n$.

# Definitions - Curves and Fields

The bytecode validation circuit is implemented over the BN254 elliptic curve with group elements defined via $\mathbb{G}_{bn254}$.

The field $\mathbb{F}$ represents the finite field whose characteristic equals the number of points on the BN254 curve.

# Bytecode representation

Each opcode in the AVM can be described by an integer in the range $[0, \ldots, 256^{31}]$ (i.e. 31 bytes of data). All opcodes excluding `SET` require much less data than 31 bytes.

In the AVM circuit architecture, multiple columns are used to define a VM operation. These columns describe the following quantities:

1. the opcode to be executed (1 byte of data)
2. three parameter columns that define either literal values or memory indexes
3. three "flag" columns that define metadata associated with each parameter (e.g. whether the parameter a should be interpreted as a literal value or a memory index, or an indirect memory index)

To both minimize the amount of information used to _define_ a given AVM program, the AVM posesses an additional column that describes the _packed_ opcode. i.e. the integer concatenation of all 6 of the above column values. We define this column via the vector of field elements $\mathbf{op} \in \mathbb{F}^n$ (where $n$ is the number of opcodes in the program). $\mathbf{op}$ is defined as the _column representation_ of an AVM program.

## Packed bytecode representation

When _broadcasting_ the data for AVM programs, we desire an encoding that minimizes the raw number of bytes broadcast, we call this the _packed representation_ of the program.

The number of bytes required to represent an element in $\mathbf{op}$ in the AVM can be derived from the value of the 1st byte (e.g `ADD` requires 7 bytes of data - the ADD opcode (1 byte) and three memory indices (each of size 2 bytes)).

See (ref: TODO!) for a table that describes the amount of data required for each opcode.

Each field element in a BN254 circuit can represent _31_ bytes of bytecode data. The packed representation of an AVM program $\mathbf{b} \in \mathbb{F}^n$ is defined as the concatenation of $\mathbf{op}$ into 31-byte chunks, represented as field elements.

There exists a mapping function $g$ that, given the packed representation $\mathbf{b}$, will produce the column representation $\mathbf{op}$.

$$
g(\mathbf{b}) = \mathbf{op}
$$

A full description of $g$ is provided [further down in this document](#Definition-of-mapping-function-g).

## Committed representation

The committed representation of an AVM program is an elliptic curve polynomial commitment $[P] \in \mathbb{G}_{bn254}$, created via the KZG polynomial commitment scheme (ref).

$[P]$ is a commitment to $P(X) \in \mathbb{F}[X]^n$ where $P(X) = \sum_{i=0}^{n-1} op_i X^i$

# Bytecode validation logic

Given inputs $\mathbf{b} \in \mathbb{F}^n$ and $[P] \in \mathbb{G}_{bn254}$, we must validate that $[P] = \text{commit}_{KZG}(g(\mathbf{b}))$.

This requires the following _high level_ steps:

1. For all $i \in [0, \ldots, n - 1]$, validate that $b_i < 256^{31} - 1$
2. Compute $\mathbf{op} = g(\mathbf{b})$
3. Perform a _polynomial consistency check_ between $\mathbb{op}$ and $[P]$

# Polynomial Consistency Check

> The most straightforward way of validating $\mathbb{op}, [P]$ would be to directly construct $[P]$ from $\mathbb{op}$.
> We do not do this, as this would require a large multiscalar multiplication over the BN254 curve. This could only be performed efficiently over a Grumpkin SNARK circuit, which would add downstream complexity to the Aztec architecture (currently the only Grumpkin proofs being accumulated are elliptic-curve-virtual-machine circuits). The rollup circuit architecture already supports efficient recursive aggregation of BN254 proofs - the desire is for the bytecode validation circuit to be a canonical Honk SNARK over the BN254 field.

To perform a polynomial consistency check between $\mathbb{op}$ and $[P]$, we perform the following:

1. Generate a challenge $z \in \mathbb{F}$ by computing the Poseidon hash of $H(op_0, \ldots, op_{n-1}, [P])$
2. Compute $\sum_{i=0}^{n-1} op_i z^i = r \in \mathbb{F}$
3. Validate via a KZG opening proof that $[P]$ commits to a polynomial $P(X)$ such that $P(z) = r$

In the same manner that Honk pairings can be deferred via aggregating pairing inputs into an accumulator, the pairing required to validate the KZG opening proof can also be deferred.

## Evaluating the polynomial consistency check within a circuit

The direct computation of $r = \sum_{i=0}^{n-1} op_i z^i$ is trivial as the field is native to a BN254 SNARK circuit, and will require approx. 2 constraints per opcode.

Validating a KZG opening proof will require approx. 3 non-native elliptic curve scalar multiplications, which will have a cost of approx. 30,000 constraints if using `stdlib::biggroup` from the PLONK standard library.

The major cost of the consistency check is the Poseidon hash of the packed bytecode vector $\mathbb{b}$ and the commitment $[P]$ - this will incur approx. 22 constraints per element in $\mathbb{b}$

# Definition of mapping function $g$

The following is a pseudocode description of $g$, which can efficiently be described in a Honk circuit (i.e. no branches).

We define a function `slice(element, idx, length)`. `element` is a field element interpreted as a length-31 byte array. `slice` computes the byte array `element[idx] : element[idx + length]`, converts into a field element and returns it.

We define a size-256 lookup table `c` that maps an avm instruction byte to the byte length required to represent its respective opcode.

```
g(b) {
    let i := 0; // index into bytecode array `b`
    let j := 0; // byte offset of current bytecode element
    let op := []; // vector of opcode values we need to populate
    for k in [0, n]:
    {
        let f := b[i];
        let instruction_byte := f.slice(j, 1);
        let opcode_length := c[instruction_byte];
        let bytes_remaining_in_f := 30 - j;
        let op_split := opcode_length > bytes_remaining_in_f;
        let bytes_from_f := op_split ? bytes_remaining_in_f : opcode_length;
        let op_hi := f.slice(j, bytes_from_f);

        let f' := b[i+1];
        let bytes_from_f' := opcode_length - bytes_from_f;
        let op_lo := f'.slice(0, bytes_in_f');

        op[k] := op_lo + (op_hi << (bytes_in_f' * 8));
        i := i + op_split;
        j := op_split ? bytes_in_f' : j + opcode_length;
    }
    return op;
}
```

Pseudocode definition of `slice` function constraints:

We define `pow(x)` to be a size-31 lookup table that maps an input $x \in [0, \ldots, 31]$ into the value $2^{8x}$

We require the Prover has computed witness field elements `f_lo`, `f_hi`, `result` that satisfy the following constraints:

```
slice(f, index, length)
{
    assert(f_hi < pow(index));
    assert(f_lo < pow(31 - index - length));
    assert(result < pow(length));
    assert(f == f_lo + result * pow(31 - index - length) + f_hi * pow(31 - index));
    return result;
}
```

## Evaluating `g` within a Honk circuit

The `g` function requires the contents of $\mathbb{b}$ be present via a lookup table. We can achieve this by instantiating elements of $\mathbb{b}$ via the ROM abstraction present in the Plonk standard library (table initialisation costs 2 constraints per element, table reads cost 2 constraints per element)

We can instantiate tables `c` , `pow` as lookup tables via the same mechanism.

The `slice` function requires 3 variable-length range checks. In Honk circuits we only can support fixed-length range checks.

The following pseudocode defines how a variable-length range check can be composed of fixed-length range checks. Here we assume we have previously constrained all inputs to be less than $2^{248} - 1$

```
less_than(a, b) {
    // this block is not constrained and defines witness gneeration
    let a_lo := a & (2^{124} - 1)
    let b_lo := b & (2^{124} - 1)
    let a_hi := (a >> 124)
    let b_hi := (b >> 124)
    let borrow := b_lo < a_lo
    let r_lo := b_lo - a_lo + borrow*2^124
    let r_hi := b_hi - a_hi - borrow

    // this block defines constraints
    assert(a_lo < 2^124)
    assert(a_hi < 2^124)
    assert(b_lo < 2^124)
    assert(b_hi < 2^124)
    assert(r_lo < 2^124)
    assert(r_hi < 2^124)
    assert(borrow*borrow - borrow = 0) // bool check
    assert(a_lo + 2^{124}a_hi = a)
    assert(b_lo + 2^{124}b_hi = b)
    assert(r_lo = b_lo - a_lo + borrow*2^124)
    assert(r_hi = b_hi - a_hi - borrow)
}
```

Each `slice` call requires three `less_than` calls, and each iteration of `g` requires 3 `slice` calls. In total this produces 36 size-124 range checks per iteration of `g`. Each size-124 range check requires approx. 5 constraints, producing 180 constraints of range checks per opcode processed.

A rough estimate of the total constraints per opcode processed by the `g` function would be 200 constraints per opcdoe.

# Bytecode Validation Circuit Summary

The bytecode validation circuit takes, as public inputs, the packed bytecode array $\mathbf{b} \in \mathbb{F}$ and the bytecode commitment $[P] \in \mathbb{G}_{bn254}$ (represented via field elements).

The circuit evaluates the following:

1. For all $i \in [0, \ldots, n - 1]$, validate that $b_i < 256^{31} - 1$
2. Compute $\mathbf{op} = g(\mathbf{b})$
3. Perform a _polynomial consistency check_ between $\mathbf{op}$ and $[P]$

### Summary of main circuit costs

The polynomial consistency check requires a Poseidon hash that includes the packed bytecode array $\mathbb{b}$. This requires approx. 22 Honk constraints per 31 bytes of bytecode.

The `g` function will cost approx. 200 constraints per opcode.

For a given length `n` , the approx. number of constraints required will be approx `222n`.

A circuit of size 2^21 (2 million constraints) will be able to process a program containing approximately $n = 9,400$ steps. In contrast, a Soldity program can contain a maximum of 24kb of bytecode.

Note: unless the efficiency of the validation circuit can be improved by a factor of ~4x, it will not be possible to construct bytecode validation proofs client-side in a web browser. Delegating proof construction to a 3rd party would be acceptable in this context because the 3rd party is untrusted and no secret information is leaked.
