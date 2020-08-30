///  Import the barretenberg WASM file
pub static WASM: &'static [u8] = include_bytes!("barretenberg.wasm");

pub mod composer;
mod crs;
mod fft;
mod pippenger;
mod prover;

use std::str;
use wasmer_runtime::cache::{Cache, FileSystemCache, WasmHash};
use wasmer_runtime::types::MemoryDescriptor;
use wasmer_runtime::units::Pages;
use wasmer_runtime::{compile, func, imports, memory::Memory, Ctx, Instance, Module, Value};
use wasmer_wasi::generate_import_object_for_version;

/// Barretenberg is the low level struct which calls the WASM file
/// This is the bridge between Rust and the WASM which itself is a bridge to the C++ codebase.
pub struct Barretenberg {
    instance: Instance,
}

/// A wrapper around the return value from a WASM call
/// Notice, Option<> is used because not every call returns a value
/// Some calls are simply made to free a pointer for example
/// Or manipulate the heap
pub struct WASMValue(Option<Value>);

impl WASMValue {
    pub fn value(self) -> Value {
        self.0.unwrap()
    }
}

impl Barretenberg {
    /// Transfer bytes to WASM heap
    pub fn transfer_to_heap(&mut self, arr: &[u8], offset: usize) {
        let memory = self.instance.context().memory(0);
        for (byte_id, cell) in memory.view::<u8>()[offset as usize..(offset + arr.len())]
            .iter()
            .enumerate()
        {
            cell.set(arr[byte_id]);
        }
    }
    // change to read_mem
    pub fn slice_memory(&mut self, start: usize, end: usize) -> Vec<u8> {
        let memory = self.instance.context().memory(0);

        let mut result = Vec::new();

        for cell in memory.view()[start as usize..end].iter() {
            result.push(cell.get());
        }

        result
    }

    pub fn call(&self, name: &str, param: &Value) -> WASMValue {
        self.call_multiple(name, vec![param])
    }
    pub fn call_multiple(&self, name: &str, params: Vec<&Value>) -> WASMValue {
        // We take in a reference to values, since they do not implement Copy.
        // We then clone them inside of this function, so that the API does not have a bunch of Clones everywhere
        // We take in a reference to values, since they do not implement Copy.
        // We then clone them inside of this function, so that the API does not have a bunch of Clones everywhere

        let params: Vec<_> = params.into_iter().map(|p| p.clone()).collect();
        let option_value = self.instance.call(name, &params).unwrap().first().cloned();

        WASMValue(option_value)
    }

    /// Creates a pointer and allocates the bytes that the pointer references to, to the heap
    pub fn allocate(&mut self, bytes: &[u8]) -> Value {
        let ptr = self
            .call("bbmalloc", &Value::I32(bytes.len() as i32))
            .value();
        self.transfer_to_heap(bytes, ptr.to_u128() as usize);
        ptr
    }

    /// Frees a pointer.
    /// Notice we consume the Value, if you clone the value before passing it to free
    /// It most likely is a bug
    pub fn free(&mut self, pointer: Value) {
        self.call("bbfree", &pointer);
    }
}

fn load_module() -> Module {
    let cache_key = WasmHash::generate(WASM);

    // Load module from the cache if we already compiled it
    let mut fs_cache = unsafe { FileSystemCache::new("mod_cache/").unwrap() };
    let module = match fs_cache.load(cache_key) {
        Ok(module) => module,
        Err(_) => {
            println!("Compiling WASM... This will take ~3 minutes for the first time, and cached on subsequent runs.");
            let module = compile(&WASM).expect("wasm compilation");
            // Store a module into the cache given a key
            fs_cache.store(cache_key, module.clone()).unwrap();
            module
        }
    };
    module
}
impl Barretenberg {
    pub fn new() -> Barretenberg {
        let module = load_module();

        // get the version of the WASI module in a non-strict way, meaning we're
        // allowed to have extra imports
        let wasi_version = wasmer_wasi::get_wasi_version(&module, false)
            .expect("WASI version detected from Wasm module");

        // WASI imports
        let mut base_imports =
            generate_import_object_for_version(wasi_version, vec![], vec![], vec![], vec![]);

        // env is the default namespace for extern functions
        let descriptor = MemoryDescriptor::new(Pages(129), None, false).unwrap();
        let memory = Memory::new(descriptor).unwrap();
        let custom_imports = imports! {
            "env" => {
                "logstr" => func!(logstr),
                "memory" => memory.clone(),
            },
        };
        // The WASI imports object contains all required import functions for a WASI module to run. This includes wasi_unstable
        base_imports.extend(custom_imports);
        let instance = module
            .instantiate(&base_imports)
            .expect("failed to instantiate wasm module");

        Barretenberg { instance: instance }
    }
}

fn logstr(ctx: &mut Ctx, ptr: u32) {
    // Get a slice that maps to the memory currently used by the webassembly
    // instance.
    //
    // Webassembly only supports a single memory for now,
    // but in the near future, it'll support multiple.
    //
    // Therefore, we don't assume you always just want to access first
    // memory and force you to specify the first memory.
    let memory = ctx.memory(0);
    let len = 10;

    // Get a subslice that corresponds to the memory used by the string.
    let str_vec: Vec<_> = memory.view()[ptr as usize..(ptr + len) as usize]
        .iter()
        .map(|cell| cell.get())
        .collect();

    // Convert the subslice to a `&str`.
    let string = str::from_utf8(&str_vec).unwrap();

    // Print it!
    dbg!("{}", string);
}
