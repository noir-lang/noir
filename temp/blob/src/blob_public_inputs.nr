use bigint::{BigNum, BLS12_381_Fr as F};
use std::ops::Add;
use types::{
    constants::{BLOB_PUBLIC_INPUTS, BLOBS_PER_BLOCK},
    traits::{Deserialize, Empty, Serialize},
    utils::reader::Reader,
};

// TODO(MW): delete

// NB: This only exists because a nested array of [[Field; 2]; N] did not build with ci2.5, but was fine otherwise
// For blobs, we use the compressed 48 byte BLS12 commitment to compute the challenge. We never need to operate on it,
// so it's encoded as 2 fields. The first is the first 31 bytes, and the second is the next 17 bytes.
pub struct BlobCommitment {
    pub inner: [Field; 2],
}

impl Eq for BlobCommitment {
    fn eq(self, other: Self) -> bool {
        self.inner.eq(other.inner)
    }
}

impl Empty for BlobCommitment {
    fn empty() -> Self {
        Self { inner: [0; 2] }
    }
}

pub struct BlobPublicInputs {
    pub z: Field,
    pub y: F,
    pub kzg_commitment: BlobCommitment,
}

impl BlobPublicInputs {
    pub fn accumulate(self, other: Self) -> Self {
        // TODO: When we verify root, rather than block root, on L1 we need to accumulate many blob openings
        // @Mike this may be where we calculate z_acc, y_acc, C_acc, etc.
        // WARNING: unimplemented, below is nonsense to get noir to compile
        Self { z: self.z + other.z, y: self.y.add(other.y), kzg_commitment: self.kzg_commitment }
    }

    // This checks whether the blob public inputs represent data of all 0s
    // This is not equivalent to being empty, since the challenge point z is a hash and won't have 0 value.
    pub fn is_zero(self) -> bool {
        // Note: there is no constrained is_zero in bignum
        (self.y == F::zero()) & (self.kzg_commitment.inner == [0, 0])
    }
}

impl Empty for BlobPublicInputs {
    fn empty() -> Self {
        Self { z: 0, y: BigNum::zero(), kzg_commitment: BlobCommitment::empty() }
    }
}

impl Serialize<BLOB_PUBLIC_INPUTS> for BlobPublicInputs {
    fn serialize(self) -> [Field; BLOB_PUBLIC_INPUTS] {
        [
            self.z,
            self.y.get_limb(0) as Field,
            self.y.get_limb(1) as Field,
            self.y.get_limb(2) as Field,
            self.kzg_commitment.inner[0],
            self.kzg_commitment.inner[1],
        ]
    }
}

impl Deserialize<BLOB_PUBLIC_INPUTS> for BlobPublicInputs {
    fn deserialize(fields: [Field; BLOB_PUBLIC_INPUTS]) -> Self {
        Self {
            z: fields[0],
            y: F::from_limbs([fields[1] as u128, fields[2] as u128, fields[3] as u128]),
            kzg_commitment: BlobCommitment { inner: [fields[4], fields[5]] },
        }
    }
}

impl Eq for BlobPublicInputs {
    fn eq(self, other: Self) -> bool {
        (self.z == other.z) & (self.y.eq(other.y)) & (self.kzg_commitment.eq(other.kzg_commitment))
    }
}

// NB: it is much cleaner throughout the protocol circuits to define this struct rather than use a nested array.
// Once we accumulate blob inputs, it should be removed, and we just use BlobPublicInputs::accumulate everywhere.
pub struct BlockBlobPublicInputs {
    pub inner: [BlobPublicInputs; BLOBS_PER_BLOCK],
}

impl Empty for BlockBlobPublicInputs {
    fn empty() -> Self {
        Self { inner: [BlobPublicInputs::empty(); BLOBS_PER_BLOCK] }
    }
}

impl Serialize<BLOB_PUBLIC_INPUTS * BLOBS_PER_BLOCK> for BlockBlobPublicInputs {
    fn serialize(self) -> [Field; BLOB_PUBLIC_INPUTS * BLOBS_PER_BLOCK] {
        let mut fields: BoundedVec<Field, BLOB_PUBLIC_INPUTS * BLOBS_PER_BLOCK> = BoundedVec::new();
        for i in 0..BLOBS_PER_BLOCK {
            fields.extend_from_array(self.inner[i].serialize());
        }
        fields.storage()
    }
}

impl Deserialize<BLOB_PUBLIC_INPUTS * BLOBS_PER_BLOCK> for BlockBlobPublicInputs {
    fn deserialize(fields: [Field; BLOB_PUBLIC_INPUTS * BLOBS_PER_BLOCK]) -> Self {
        let mut reader = Reader::new(fields);
        let item = Self {
            inner: reader.read_struct_array(
                BlobPublicInputs::deserialize,
                [BlobPublicInputs::empty(); BLOBS_PER_BLOCK],
            ),
        };
        reader.finish();
        item
    }
}

impl Eq for BlockBlobPublicInputs {
    fn eq(self, other: Self) -> bool {
        self.inner.eq(other.inner)
    }
}

#[test]
fn serialization_of_empty() {
    let item = BlockBlobPublicInputs::empty();
    let serialized = item.serialize();
    let deserialized = BlockBlobPublicInputs::deserialize(serialized);
    assert(item.eq(deserialized));
}
