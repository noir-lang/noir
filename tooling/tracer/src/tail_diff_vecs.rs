/// Produce the "tail difference" of two vectors.
///
/// Here, this means that the common prefix of the two vectors is computed. The length of that
/// prefix is returned, together with the suffix of the first vector that follows the common
/// prefix, as well as the suffix of the second vector that follows the common prefix.
///
/// Example:
/// ```
/// use noir_tracer::tail_diff_vecs::tail_diff_vecs;
/// let a = 1;
/// let b = 2;
/// let c = 3;
/// let d = 4;
/// let e = 5;
/// let f = 6;
///
/// assert_eq!(tail_diff_vecs(&vec!(a, b, c), &vec!(a, b, e, f)), (2, vec!(&c), vec!(&e, &f)));
//
/// assert_eq!(tail_diff_vecs(&vec!(a, b, c), &vec!(e, f)), (0, vec!(&a, &b, &c), vec!(&e, &f)));
///
/// assert_eq!(tail_diff_vecs(&vec!(a, b, c), &vec!(a, b)), (2, vec!(&c), vec!()));
///
/// assert_eq!(tail_diff_vecs(&vec!(a, b, c, d), &vec!(a, b, e)), (2, vec!(&c, &d), vec!(&e)));
///
/// // Corner cases:
/// assert_eq!(tail_diff_vecs(&vec!(a, b), &vec!(a, b, c)), (2, vec!(), vec!(&c)));
/// assert_eq!(tail_diff_vecs::<usize>(&vec!(), &vec!()), (0, vec!(), vec!()));
/// ```
pub fn tail_diff_vecs<'a, T>(xs: &'a [T], ys: &'a [T]) -> (usize, Vec<&'a T>, Vec<&'a T>)
where
    T: PartialEq + Clone,
{
    let min_len = std::cmp::min(xs.len(), ys.len());
    let first_nomatch = xs.iter().zip(ys.iter()).position(|(x, y)| x != y).unwrap_or(min_len);

    let xs_tail: Vec<&T> = xs[first_nomatch..].iter().collect();
    let ys_tail: Vec<&T> = ys[first_nomatch..].iter().collect();

    (first_nomatch, xs_tail, ys_tail)
}
