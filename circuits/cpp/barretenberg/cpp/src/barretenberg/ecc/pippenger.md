# Implement cahe-optimised, parallelisable pippenger implementation 

## The problem 

Pippenger's algorithm for batched scalar multiplications is the fastest known algorithm for computing a large number of scalar multiplications, with a run time of `O(n / logn)`, where `n` is the number of points being multiplied. 

We currently have pippenger implemented in barretenberg, which can process a scalar multiplication in ~4 micro-seconds (for large batches).

However, the algorithm, as it stands, is not parallelisable, due to the highly non-sequential memory access patterns.

## The algorithm 

Say you have `n` `g1::affine_element` points, and `n` `fr` scalars you want to multiply each point by. 

Pippenger's algorithm proceeds, but taking each scalar and slicing it into sequences of bits. The size of each slice depends on the number of points being multiplied.

The number of rounds of the algorithm depends on the number of slices it takes to slice up a scalar.

Basically - the more bits per slice, the fewer rounds, but the more time-consuming each round is. The extra time added into each round is purely a function of the slice size. The more points you have, the larger you want your slice size to be (as the amount of work per round increases with the number of points, so there will be a sweet spot where reducing the number of rounds will not be worth increasing the slice size).

For each round, every point being multiplied will be added into a 'bucket'. Which bucket will depend on the value of the scalar bit slice that corresponds to the round.

For example, let's say that our bit slice is 6 bits. The first round will take the most significant 6 bits of every scalar. The value of this slice corresponds to a bucket index. So, for a point P, we have buckets that map to P, 2P, 3P, ..., 2^{6}P. Each bucket contains an 'accumulator' point. At the start of each round, each bucket's accumulator point is set to the point at infinity (basically 0). We then iterate over every scalar, and add the scalar's corresponding point into one of the buckets.

So, for example, if the most significant 6 bits of a scalar are `011001` (25), we add the scalar's point into the 25th bucket. 

At the end of each round, we then 'concatenate' all of the buckets into a sum. Let's represent each bucket accumulator in an array `A[num_buckets]`. The concatenation phase will compute `A[0] + 2A[1] + 3A[2] + 4A[3] + 5A[4] + ... = Sum`.

Finally, we add each `Sum` point into an overall accumulator. For example, for a set of 254 bit scalars, if we evaluate the most 6 significant bits of each scalar and accumulate the resulting point into `Sum`, we actually need `(2^{248}).Sum` to accomodate for the bit shift. 

This final step is similar to the 'double and add' algorithm in a traditional scalar multiplication algorithm - we start at the most significant bit slice and work our way down to minimize the number of doublings. At each round, we multiply the overall accumulator by 2^{bit slice size} - by the time we iterate over every round, we will have performed the total required number of doublings for every 'bit slice' that we add into our accumulator.

## The utility of Pippenger's algorithm 

Let's say you have 2^{18} points and an 254 bit scalar (e.g. the BN curve). Using a naive algorithm, you would need to perform 127 additions for every point. With pippenger, the optimal bucket width is (after optimizations) 15 bits. We can (after optimizations) iterate over all 254 bits in 16 rounds of 15-bit slices. At the end of each round, we need to perform 2^{15} additions to concatenate our buckets. For 16 rounds, that equals 2^{19} additions.

Total run time = 16 * 2^{18} + 16 * 2^{15} = 18 * 2^{18}. So the aggregate number of additions required per point is only 18. i.e. 10 times more efficient than the naive algorithm.

## The problem with Pippenger's algorithm 

As it is currently implemented, each round will iterate over the points to be added, and add each point into one of the round's buckets. Whilst point access is sequential in memory, bucket access is very much not. In fact, if the points being multiplied are from a zero-knowlege proof, bucket access is literally uniformly randomly distributed and therefore presents the worst-case scenario.

This makes it difficult to parallelize. It is not possible to simply assign threads a section of points to iterate over, because of race conditions when two threads access the same bucket.

Splitting threads over the number of pippenger rounds is possible, but there is no gaurantee that the number of rounds will nicely divide the number of threads.

Our currently implementation will simply split up one multiple-scalar multipliation into smaller multiple-scalar multiplications (one per thread). While this works, it means each thread works over a larger number of rounds (as the run time is O(n / logn) and we've just shrunk n by the number of threads). It is also completely impractical for massively parallelizable architectures like GPUs.

## The solution - ordering points in bucket order

We can add a preprocessing step that organises our point data into 'bucket order'. Effectively, each bucket is assigned a block of memory, and the points we want to add into the bucket are copied into this memory block. This is essentially a radix sort, with the following procedure:

1. Iterate over every scalar, computing the 'windowed non-adjacent form' bit slices required for each round 
2. For each round, iterate over the wnaf slices, counting the number of entries assigned to each bucket 
3. Use the bucket counts to allocate memory for each bucket (let's call these bucket blocks)
4. For each round, iterate over the wnaf slices, using the wnaf slice value to copy the associated point into the relevant bucket block

Once this has been achieved, we can then, for each bucket, iterate over the bucket's associated points, and add the point into the bucket accumulator.

The benefits of this approach are:

1. Easily parallelizable, threads / gpu cores can be assigned a range of buckets to iterate over 
2. Reduced control flow in main loop - we currently have to check whether a bucket is empty for iteration of our main loop - with this version, we can skip this step and initialize each bucket to contain the first bucket point
3. Concretely fewer field multiplications: when adding the second bucket point into a bucket accumulator, both points will have a Z-coordinate of 1. This allows us to use a more efficient point addition algorithm for this special case

Drawbacks of this approach: 

1. Memory latency will be a problem for the radix sort - each bucket will have a size that is greater than the L1 cache for all but the smallest of PLONK circuits. Until we have a working implementation further optimization is premature, but if this becomes a problem we can use a cache-optimized radix sort (more sorting rounds, but each round works on data that's in the L1 cache)

## Summary 

By restructuring the memory heirarchy of our pippenger algorithm, we can create a parallelizable version of pippenger. This will significantly simplify the logic of our PLONK prover (instead of allocating threads for batches of multi-exponentations, we can multi-thread individual multi-exponentiations, simplifying our thread logic). 

This will concretely reduce the number of pippenger rounds of our multi-exponentiations by approximately 1, giving a theoretical 15% speed-up. Some of this will be eaten by the run-time of the radix sort. 

Longer term, this parallelizable algorithm will be significantly easier to adapt for GPUs, using OpenCL.