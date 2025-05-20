use crate::{
    constants::{BLOBS_PER_BLOCK, FIELDS_PER_BLOB, SPONGE_BLOB_LENGTH, TWO_POW_64},
    hash::poseidon2_absorb_chunks_existing_sponge,
    poseidon2::Poseidon2Sponge,
    traits::{Deserialize, Empty, Serialize},
};

// A Poseidon2 sponge used to accumulate data that will be added to blob(s)
// (More accurately called BlobSponge, but that's not as fun)

// Each base rollup circuit has a start and end SpongeBlob instance, like a PartialStateReference.
// Tx data is accumulated by calling absorb() on each field, and incrementing the number of fields.
// To save gates, we do not check after every absorb whether we have filled a blob (we do this in block root).

// Each merge rollup circuit checks that the left rollup input's end SpongeBlob == the right rollup input's start SpongeBlob.

// Finally, the block_root is given the list of all fields to be included in the blob.
// We hash these ^ and check that the hash matches the one given by the block_root's right input's end SpongeBlob.squeeze().
// The hash is used as part of the blob challenge, as we've proven it encompasses all elts of the blob(s).

// Init is given by input len * 2^64 (see noir/noir-repo/noir_stdlib/src/hash/poseidon2.nr -> hash_internal)
global IV: Field = (FIELDS_PER_BLOB * BLOBS_PER_BLOCK) as Field * TWO_POW_64;

pub struct SpongeBlob {
    pub sponge: Poseidon2Sponge,
    pub fields: u32,
    pub expected_fields: u32, // The hinted number of tx effects this will absorb
}

impl SpongeBlob {
    pub fn new_full_blobs() -> Self {
        Self { sponge: Poseidon2Sponge::new(IV), fields: 0, expected_fields: 0 }
    }

    pub fn new(expected_fields_hint: u32) -> Self {
        Self {
            sponge: Poseidon2Sponge::new((expected_fields_hint as Field) * TWO_POW_64),
            fields: 0,
            expected_fields: expected_fields_hint,
        }
    }

    // Add fields to the sponge
    pub fn absorb<let N: u32>(&mut self, input: [Field; N], in_len: u32) {
        // We skip the 0 check below, as most use cases (e.g. base rollup) constrain that the input array
        // is constructed from i=0->in_len from an empty array, so no need to check.
        self.sponge = poseidon2_absorb_chunks_existing_sponge(self.sponge, input, in_len, true);
        self.fields += in_len;
    }

    // Finalise the sponge and output poseidon2 hash of all fields absorbed
    pub fn squeeze(&mut self) -> Field {
        // If the blob sponge is not 'full', we append 1 to match Poseidon2::hash_internal()
        // NB: There is currently no use case in which we don't 'fill' a blob sponge, but adding for completeness
        if self.fields != self.expected_fields {
            self.sponge.absorb(1);
        }
        self.sponge.squeeze()
    }
}

impl Eq for SpongeBlob {
    // TODO: should Poseidon2 struct have a .eq() impl?
    fn eq(self, other: Self) -> bool {
        (self.fields == other.fields)
            & (self.sponge.cache == other.sponge.cache)
            & (self.sponge.state == other.sponge.state)
            & (self.sponge.cache_size == other.sponge.cache_size)
            & (self.sponge.squeeze_mode == other.sponge.squeeze_mode)
    }
}

impl Serialize<SPONGE_BLOB_LENGTH> for SpongeBlob {
    fn serialize(self) -> [Field; SPONGE_BLOB_LENGTH] {
        let mut fields: BoundedVec<Field, SPONGE_BLOB_LENGTH> = BoundedVec::new();
        fields.extend_from_array(self.sponge.cache);
        fields.extend_from_array(self.sponge.state);
        fields.push(self.sponge.cache_size as Field);
        fields.push(self.sponge.squeeze_mode as Field);
        fields.push(self.fields as Field);
        fields.push(self.expected_fields as Field);

        fields.storage()
    }
}

impl Deserialize<SPONGE_BLOB_LENGTH> for SpongeBlob {
    fn deserialize(fields: [Field; SPONGE_BLOB_LENGTH]) -> Self {
        Self {
            sponge: Poseidon2Sponge {
                cache: [fields[0], fields[1], fields[2]],
                state: [fields[3], fields[4], fields[5], fields[6]],
                cache_size: fields[7] as u32,
                squeeze_mode: fields[8] as bool,
            },
            fields: fields[9] as u32,
            expected_fields: fields[10] as u32,
        }
    }
}

impl Empty for SpongeBlob {
    fn empty() -> Self {
        Self { sponge: Poseidon2Sponge::new(0), fields: 0, expected_fields: 0 }
    }
}

#[test]
fn serialization_of_empty() {
    let item = SpongeBlob::empty();
    let serialized = item.serialize();
    let deserialized = SpongeBlob::deserialize(serialized);
    assert(item.eq(deserialized));
}

use crate::utils::arrays::array_concat;

#[test]
unconstrained fn absorb() {
    // This tests that absorbing two arrays separately then squeezing matches an ordinary hash
    let mut spongeblob = SpongeBlob::new(7);
    let input_3 = [1, 2, 3];
    spongeblob.absorb(input_3, input_3.len());
    // Assert that we have correctly absorbed the first 3 inputs
    assert(spongeblob.sponge.cache.eq(input_3));
    assert(spongeblob.fields == input_3.len());
    // Absorb the next 4 in a new call...
    let input_4 = [4, 5, 6, 7];
    spongeblob.absorb(input_4, input_4.len());
    // ...and create a normal poseidon2 hash of the same input
    let input: [Field; 7] = array_concat(input_3, input_4);
    let expected = Poseidon2Sponge::hash(input, input.len());
    assert(spongeblob.squeeze() == expected);
}

#[test(should_fail_with = "Given in_len to absorb is larger than the input array len")]
unconstrained fn absorb_incorrect_in_len() {
    let mut spongeblob = SpongeBlob::new(10);
    let input_3 = [1, 2, 3];
    // The below should fail, as we try to absorb 10 inputs but only provide 3
    spongeblob.absorb(input_3, 10);
}
