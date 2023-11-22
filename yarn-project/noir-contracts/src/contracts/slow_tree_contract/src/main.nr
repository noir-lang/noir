mod capsule;
mod types;

// This contract allow us to "read" public storage in private through a delayed tree.
// More documentation need to be outlined for this properly, but there is some in 
// https://github.com/AztecProtocol/aztec-packages/issues/1291
// This is made as a separate contract for one thing mainly. Making it simpler to use.
contract SlowTree {
    use dep::std::option::Option;
    use dep::value_note::{
        balance_utils,
        utils::{increment, decrement},
        value_note::{VALUE_NOTE_LEN, ValueNote, ValueNoteMethods},
    };
    use dep::aztec::{
        context::{PrivateContext, PublicContext, Context},
        note::{
            note_header::NoteHeader,
            utils as note_utils,
        },
        selector::compute_selector,
        state_vars::{map::Map, public_state::PublicState, set::Set},
        types::type_serialization::field_serialization::{
            FieldSerializationMethods, FIELD_SERIALIZED_LEN,
        },
    };
    use dep::slow_updates_tree::slow_map::{
        SlowMap, Leaf, SlowUpdateProof, compute_merkle_root, deserialize_slow_update_proof
    };

    use crate::capsule::pop_capsule;
    use crate::types::{MembershipProof, deserialize_membership_proof};

    global TREE_HEIGHT: Field = 254;
    global MEMBERSHIP_SIZE: Field = 256; // TREE_HEIGHT + 2
    global UPDATE_SIZE: Field = 512; // TREE_HEIGHT * 2 + 4

    global EMPTY_ROOT: Field = 5785871043333994658400733180052743689641713274194136017445890613179954325976;

    struct Storage {
        trees: Map<SlowMap<TREE_HEIGHT, UPDATE_SIZE>>,
    }

    impl Storage {
        fn init(context: Context) -> pub Self {
            Storage {
                trees: Map::new(
                    context,
                    1,
                    |context, slot| {
                        SlowMap::new(
                            context,
                            slot,
                        )
                    }
                ),
            }
        }
    }

    #[aztec(private)]
    fn constructor() {}

    #[aztec(public)]
    fn initialize() {
        storage.trees.at(context.msg_sender()).initialize(EMPTY_ROOT);
    }

    #[aztec(public)]
    fn read_at_pub(key: Field) -> Field {
        storage.trees.at(context.msg_sender()).read_at(key)
    }

    #[aztec(public)]
    fn read_leaf_at_pub(key: Field) -> Leaf {
        storage.trees.at(context.msg_sender()).read_leaf_at(key)
    }

    #[aztec(private)]
    fn read_at(index: Field) -> Field {
        let fields = pop_capsule();
        let p: MembershipProof<TREE_HEIGHT, MEMBERSHIP_SIZE> = deserialize_membership_proof(fields);
        assert(index == p.index, "Index does not match expected");

        let expected_root = compute_merkle_root(p.value, p.index, p.sibling_path);
        let selector = compute_selector("_assert_current_root(Field,Field)");
        context.call_public_function(context.this_address(),
            selector,
            [context.msg_sender(), expected_root]);

        p.value
    }

    #[aztec(public)]
    internal fn _assert_current_root(caller: Field, expected: Field) {
        let root = storage.trees.at(caller).current_root();
        assert(root == expected, "Root does not match expected");
    }

    #[aztec(public)]
    fn update_at_public(p: SlowUpdateProof<TREE_HEIGHT, UPDATE_SIZE>) {
        storage.trees.at(context.msg_sender()).update_at(p);
    }

    #[aztec(private)]
    fn update_at_private(index: Field, new_value: Field) {
        let fields = pop_capsule();
        let p: SlowUpdateProof<TREE_HEIGHT, UPDATE_SIZE> = deserialize_slow_update_proof(fields);
        assert(index == p.index, "Index does not match expected");
        assert(new_value == p.new_value, "New value does not match expected");

        // We compute the root before.
        let before_root = compute_merkle_root(p.before.value, p.index, p.before.sibling_path);
        let after_root = compute_merkle_root(p.after.value, p.index, p.after.sibling_path);
        let new_after_root = compute_merkle_root(p.new_value, p.index, p.after.sibling_path);

        let selector = compute_selector("_update(Field,Field,Field,Field,Field,Field)");
        context.call_public_function(context.this_address(),
            selector,
            [
                context.msg_sender(),
                p.index,
                p.new_value,
                before_root,
                after_root,
                new_after_root
            ]);
    }

    #[aztec(public)]
    internal fn _update(caller: Field, index: Field, new_value: Field, before: Field, after: Field, new_root: Field) {
        let current_root = storage.trees.at(caller).current_root();
        let after_root = storage.trees.at(caller).read_root().after;

        assert(current_root == before, "Before root does not match expected");
        assert(after_root == after, "After root does not match expected");

        storage.trees.at(caller).update_unsafe_at(index, new_value, new_root);
    }

    unconstrained fn un_read_leaf_at(address: Field, key: Field) -> Leaf {
        storage.trees.at(address).read_leaf_at(key)
    }

    unconstrained fn un_read_root(address: Field) -> Leaf {
        storage.trees.at(address).read_root()
    }

    unconstrained fn compute_note_hash_and_nullifier(_contract_address: Field, _nonce: Field, _storage_slot: Field, _preimage: [Field; 4]) -> [Field; 4] {
        [0x0d, 0x0e, 0x0a, 0x0d]
    }
}
