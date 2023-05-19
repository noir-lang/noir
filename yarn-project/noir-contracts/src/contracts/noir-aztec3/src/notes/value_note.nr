// global KEY_GENERATOR = Point { x: 1234, y: 5678 }; // would need to be some valid generator point.

use crate::types::point::Point;

// oracle
use crate::oracle::get_secret_key::get_secret_key;
use crate::oracle::rand::rand;

global NOTE_LEN = 6;

struct Note {
    value: Field,
    owner: Point,
    randomness: Field,
    nonce: Field,
    is_real: bool
}

impl Note {
    fn dummy() -> Note {
        Note {
            value: 0,
            owner: Point { x: 0, y: 0 },
            randomness: 0,
            nonce: 0,
            is_real: false,
        }
    }

    fn deserialize(
        fields: [Field; NOTE_LEN],
    ) -> Note {
        Note {
            is_real: fields[0] as bool,
            nonce: fields[1],
            owner: Point {
                x: fields[2],
                y: fields[3]
            },
            randomness: fields[4],
            value: fields[5],
        }
    }

    fn new(value: Field, owner: Point) -> Note {
        // let randomness = ORACLE::rand(); // ORACLE CALL
        // let nonce = ORACLE::rand(); // ORACLE CALL (nonce not secure: would need to be better in production).

        // rand_idx prevents repeated oracles + args from being optimised away.
        // The + 1 is a workaround for:
        //     The application panicked (crashed).
        //     Message:  unexpected constant expression
        //     Location: crates/noirc_evaluator/src/ssa/acir_gen/operations/intrinsics.rs:280
        let randomness = rand();
        // let randomness = 4; // Chosen by fair dice roll
        let nonce = 3;      // Guaranteed to be random

        Note {
            value,
            owner,
            randomness,
            nonce,
            is_real: true,
        }
    }

    fn compute_note_hash(self) -> Field {
        dep::std::hash::pedersen([
            crate::NoteHash,
            self.is_real as Field,
            self.nonce, 
            self.owner.x, 
            self.owner.y,
            self.randomness, 
            self.value, 
        ])[0]
    }

    fn compute_nullifier(self, note_hash: Field) -> Field {
        let mut secret_key = 0;

        if(self.is_real){
            secret_key = get_secret_key(self.owner);
            // TODO derive public key here
            let check_public_key = self.owner;
            constrain (check_public_key.x == self.owner.x) & (check_public_key.y == self.owner.y);
        }

        dep::std::hash::pedersen([crate::Nullifier, note_hash, secret_key])[0]
    }

    fn validate(self, sender: Point){
        if self.is_real {
            constrain sender.x == self.owner.x;
            constrain sender.y == self.owner.y;
        } else {
            constrain self.value == 0;
        }
    }
}
