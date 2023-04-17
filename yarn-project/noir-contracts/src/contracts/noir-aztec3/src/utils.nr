fn arr_copy_slice<T, N, M>(
    src: [T; N],
    mut dst: [T; M],
    offset: comptime Field,
) -> [T; M] {
    for i in 0..dst.len() {
        dst[i] = src[i + offset];
    }
    dst
}