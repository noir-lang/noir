use crate::bindings::ecc_mult;

pub fn new(crs_data: &[u8]) -> *mut std::os::raw::c_void {
    let num_points = (crs_data.len() / 64) as u64;
    let result: *mut std::os::raw::c_void;
    unsafe {
        result = ecc_mult::new_pippenger(crs_data.as_ptr(), num_points);
    }
    result
}
