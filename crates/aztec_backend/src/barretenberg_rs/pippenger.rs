pub struct Pippenger {
    pippenger_ptr: *mut std::os::raw::c_void,
}

impl Pippenger {
    pub fn new(crs_data: &[u8]) -> Pippenger {
        let pippenger_ptr = barretenberg_wrapper::pippenger::new(crs_data);
        Pippenger { pippenger_ptr }
    }

    pub fn pointer(&self) -> *mut std::os::raw::c_void {
        self.pippenger_ptr
    }
}
