(function() {
    var type_impls = Object.fromEntries([["noir_fuzzer",[]],["noir_greybox_fuzzer",[]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[18,27]}