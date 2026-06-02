fn main(x: [u8; 64], x_len: u32, y: [u8; 64], y_len: u32, z: [u8; 64], z_len: u32) {
    let mut vec: BoundedVec<u8, 192> = BoundedVec::new();
    let vec_x = BoundedVec::from_parts_unchecked(x, x_len);
    let vec_y = BoundedVec::from_parts_unchecked(y, y_len);
    let vec_z = BoundedVec::from_parts_unchecked(z, z_len);
    vec.extend_from_bounded_vec(vec_x);
    vec.extend_from_bounded_vec(vec_y);
    vec.extend_from_bounded_vec(vec_z);

    println(vec);
}
