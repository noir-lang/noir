use crate::array;
use crate::pow_32;
mod bn254; // Instantiations of Poseidon for prime field of the same order as BN254

// General Poseidon permutation on elements of type Field
fn permute<N,O,P>(rf: u8, // Number of full rounds; should be even
                  rp: u8, // Number of partial rounds
                  alpha: Field, // S-box power, depending on the underlying field
                  ark: [Field; O], // Additive round keys
                  mds_matrix: [Field; P], // MDS Matrix in row-major order
                  state: [Field; N] // State as an array of length t = c + r
) -> [Field; N]
{
    let t = array::len(state);

    // Input checks
    constrain t as u8 * (rf + rp) == array::len(ark) as u8;
    constrain t * t == array::len(mds_matrix);
    constrain alpha > 0;
    
    let mut count = 0;

    let mut out_perm = state;

    for r in 0..(array::len(ark)/array::len(state)) // i.e. r in 0..rf + rp
    {
        for i in 0..array::len(state)
        {
            out_perm[i] = out_perm[i] + ark[count + i];
        } // Shift by round constants
        
        out_perm[0] = pow_32(out_perm[0], alpha);
        
        if (r as u8 < rf/2) | (r as u8 >= rf/2 + rp) // Check whether we are in a full round
        {
            for i in 1..array::len(state)
            {
                out_perm[i] = pow_32(out_perm[i], alpha);
            }
        }
        
        out_perm = apply_matrix(mds_matrix, out_perm); // Apply MDS matrix
        count = count + t;
    }

    out_perm
}

// Absorption. Fully absorbs input message.
fn absorb<N,O,P,Q>(rf: u8,
                   rp: u8,
                   alpha: Field,
                   ark: [Field; O],
                   mds_matrix: [Field; P],
                   state: [Field; N], // Initial state; usually [0; N]
                   rate: Field, // Rate
                   capacity: comptime Field, // Capacity; usually 1
                   msg: [Field; Q] // Arbitrary length message
) -> [Field; N]
{
    let mut block = state;
    
    let mut i = 0;
    
    for k in 0..array::len(msg)
    {
        // Populate block
        block[capacity + i] += msg[k];
        i = i+1;
        if i == rate // Enough to absorb
        {
            block = permute(rf, rp, alpha, ark, mds_matrix, block);
            i = 0;
        }
    };
    
    if i > 0 // If we have one more block to permute
    {
        block = permute(rf, rp, alpha, ark, mds_matrix, block);
    }
    
    block
}

// A*x where A is an n x n matrix in row-major order and x an n-vector
fn apply_matrix<N>(a: [Field], x: [Field; N]) -> [Field; N]
{
    let mut y = x;

    for i in 0..array::len(x)
    {
        y[i] = 0;
        for j in 0..array::len(x)
        {
            y[i] = y[i] + a[array::len(x)*i + j]* x[j];            
        };
    };
    
    y
}
