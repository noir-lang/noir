mod capsule;
mod types;

// This contract allow us to "read" public storage in private through a delayed tree.
// More documentation need to be outlined for this properly, but there is some in 
// https://github.com/AztecProtocol/aztec-packages/issues/1291
// This is made as a separate contract for one thing mainly. Making it simpler to use.
// TODO(#4760): Rename slow updates to shared mutable and ideally move the impl to state-vars in aztec-nr.
contract SlowTree {
    use dep::aztec::prelude::{
        AztecAddress, FunctionSelector, NoteHeader, NoteInterface, NoteViewerOptions, PrivateContext,
        Map, PublicMutable, PrivateSet
    };

    use dep::value_note::{balance_utils, utils::{increment, decrement}, value_note::ValueNote};
    use dep::aztec::{context::{PublicContext, Context}, protocol_types::type_serialization::FIELD_SERIALIZED_LEN};
    use dep::slow_updates_tree::{SlowMap, Leaf, SlowUpdateProof, compute_merkle_root, deserialize_slow_update_proof};

    // docs:start:import_pop_capsule
    use crate::capsule::pop_capsule;
    // docs:end:import_pop_capsule
    use crate::types::{MembershipProof, deserialize_membership_proof};

    // docs:start:constants_and_storage
    global TREE_HEIGHT: u64 = 254;
    global MEMBERSHIP_SIZE: Field = 256; // TREE_HEIGHT + 2
    global UPDATE_SIZE: Field = 512; // TREE_HEIGHT * 2 + 4

    global EMPTY_ROOT: Field = 5785871043333994658400733180052743689641713274194136017445890613179954325976;

    #[aztec(storage)]
    struct Storage {
        trees: Map<Field, SlowMap<TREE_HEIGHT, UPDATE_SIZE>>,
    }
    // docs:end:constants_and_storage

    // docs:start:initialize
    #[aztec(public)]
    fn initialize() {
        storage.trees.at(context.msg_sender().to_field()).initialize(EMPTY_ROOT);
    }
    // docs:end:initialize
    // docs:start:read_at_pub
    #[aztec(public)]
    fn read_at_pub(key: Field) -> Field {
        storage.trees.at(context.msg_sender().to_field()).read_at(key)
    }
    // docs:end:read_at_pub
    #[aztec(public)]
    fn read_leaf_at_pub(key: Field) -> Leaf {
        storage.trees.at(context.msg_sender().to_field()).read_leaf_at(key)
    }
    // docs:start:read_at_private
    #[aztec(private)]
    fn read_at(index: Field) -> Field {
        // docs:start:pop_capsule
        let fields = pop_capsule();
        // docs:end:pop_capsule
        let p: MembershipProof<TREE_HEIGHT, MEMBERSHIP_SIZE> = deserialize_membership_proof(fields);
        assert(index == p.index, "Index does not match expected");

        let expected_root = compute_merkle_root(p.value, p.index, p.sibling_path);
        let selector = FunctionSelector::from_signature("_assert_current_root(Field,Field)");
        context.call_public_function(
            context.this_address(),
            selector,
            [context.msg_sender().to_field(), expected_root]
        );

        p.value
    }
    // docs:end:read_at_private
    // docs:start:assert_current_root
    #[aztec(public)]
    #[aztec(internal)]
    fn _assert_current_root(caller: Field, expected: Field) {
        let root = storage.trees.at(caller).current_root();
        assert(root == expected, "Root does not match expected");
    }
    // docs:end:assert_current_root

    // docs:start:update_at_pub
    #[aztec(public)]
    fn update_at_public(p: SlowUpdateProof<TREE_HEIGHT, UPDATE_SIZE>) {
        storage.trees.at(context.msg_sender().to_field()).update_at(p);
    }
    // docs:end:update_at_pub
    // docs:start:update_at_private
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

        let selector = FunctionSelector::from_signature("_update(Field,Field,Field,Field,Field,Field)");
        context.call_public_function(
            context.this_address(),
            selector,
            [
            context.msg_sender().to_field(),
            p.index,
            p.new_value,
            before_root,
            after_root,
            new_after_root
        ]
        );
    }
    // docs:end:update_at_private
    // docs:start:_update
    #[aztec(public)]
    #[aztec(internal)]
    fn _update(
        caller: Field,
        index: Field,
        new_value: Field,
        before: Field,
        after: Field,
        new_root: Field
    ) {
        let current_root = storage.trees.at(caller).current_root();
        let after_root = storage.trees.at(caller).read_root().after;

        assert(current_root == before, "Before root does not match expected");
        assert(after_root == after, "After root does not match expected");

        storage.trees.at(caller).update_unsafe_at(index, new_value, new_root);
    }
    // docs:end:_update
    unconstrained fn un_read_leaf_at(address: AztecAddress, key: Field) -> pub Leaf {
        storage.trees.at(address.to_field()).read_leaf_at(key)
    }

    unconstrained fn un_read_root(address: AztecAddress) -> pub Leaf {
        storage.trees.at(address.to_field()).read_root()
    }
}
