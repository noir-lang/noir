pub unconstrained fn main(arr: [Field; 300]) {
    let mut serialized_args: [Field] = &[];
    let arr_serialized: [[Field; 1]; 300] = arr.map(|x| [x]);

    for i in 0..arr.len() {
        serialized_args = serialized_args.append(arr_serialized[i].as_slice());
    }
    let calldata = serialized_args.push_front(611740813);
    std::println(calldata);
}
