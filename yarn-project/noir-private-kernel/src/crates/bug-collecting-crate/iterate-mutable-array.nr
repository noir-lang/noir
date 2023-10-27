fn array_push_field<T>(mut array : &mut [Field; T], value_to_push : Field) {
    let mut found_empty_slot = false;
    for elem in array {
        if (elem == 0) && (found_empty_slot == false) {
            found_empty_slot = true;
            *elem = value_to_push;
        }
    }
    assert(found_empty_slot);
}

error: Variable elem must be mutable to be assigned to