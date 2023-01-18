use acvm::acir::circuit::Circuit;
use std::path::PathBuf;
use std::sync::Once;

wit_bindgen_guest_rust::generate!({
    path: "./wit/noir_wasm.wit",
});

struct NoirWasm;

export_noir_wasm!(NoirWasm);

impl noir_wasm::NoirWasm for NoirWasm {
    // Returns a compiled program which is the ACIR circuit along with the ABI
    fn compile(src: String) -> String {
        init();
        let language = acvm::Language::PLONKCSat { width: 3 };
        let path = PathBuf::from(src);
        let compiled_program = noirc_driver::Driver::compile_file(path, language);
        serde_json::to_string(&compiled_program).unwrap()
    }

    // Deserialises bytes into ACIR structure
    fn acir_from_bytes(bytes: Vec<u8>) -> String {
        init();
        let circuit = Circuit::from_bytes(&bytes);
        serde_json::to_string(&circuit).unwrap()
    }

    fn acir_to_bytes(acir: String) -> Vec<u8> {
        init();
        let circuit: Circuit = serde_json::from_str(&acir).unwrap();
        circuit.to_bytes()
    }
}

fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            console::error(&info.to_string());
            prev_hook(info);
        }));
    });
}
