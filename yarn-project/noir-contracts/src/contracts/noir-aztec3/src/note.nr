// global KEY_GENERATOR = Point { x: 1234, y: 5678 }; // would need to be some valid generator point.

#[builtin(rand)]
fn rand() -> Field {}

#[builtin(getSecretKey)]
fn get_secret_key(_owner: Point) -> Field {}

struct Note {
    value: Field,
    owner: Point,
    randomness: Field,
    nonce: Field,
    is_dummy: bool
}

struct Point {
    x: Field,
    y: Field,
}

impl Note {
    fn dummy() -> Note {
        Note {
            value: 0,
            owner: Point { x: 0, y: 0 },
            randomness: 0,
            nonce: 0,
            is_dummy: true,
        }
    }

    fn new(value: Field, owner: Point) -> Note {
        // let randomness = ORACLE::rand(); // ORACLE CALL
        // let nonce = ORACLE::rand(); // ORACLE CALL (nonce not secure: would need to be better in production).

        let randomness = rand();//4; // Chosen by fair dice roll
        // let randomness = 4; // Chosen by fair dice roll
        let nonce = 3;      // Guaranteed to be random

        Note {
            value,
            owner,
            randomness,
            nonce,
            is_dummy: false,
        }
    }

    fn compute_note_hash(self) -> Field {
        dep::std::hash::pedersen([
            crate::NoteHash,
            self.is_dummy as Field,
            self.nonce, 
            self.owner.x, 
            self.owner.y,
            self.randomness, 
            self.value, 
        ])[0]
    }

    fn compute_nullifier(self, note_hash: Field) -> Field {
        // let secret_key = ORACLE::get_secret_key(self.owner); // ORACLE CALL - retrieves the secret key for a given public key.
        let secret_key = get_secret_key(self.owner);

        // let key_generator = Point { x: 1234, y: 5678 };
        // let check_public_key = scalar_mul(secret_key, key_generator); // (function not show in this doc: just assumed to exist in noir's stdlib)
        let check_public_key = self.owner;

        constrain (check_public_key.x == self.owner.x) & (check_public_key.y == self.owner.y);

        dep::std::hash::pedersen([crate::Nullifier, note_hash, secret_key])[0]
    }
}

impl Point {
    fn new(x: Field, y: Field) -> Self {
        Point { x, y }
    } 
}
