mod test {
    global arr_0: [u32; 1] = [0];

    #[test]
    fn test_sort() {
        let mut arr: [u32; 1] = [1];
        let sorted = arr.sort_via(|a, b| a <= b);
        assert(sorted == arr);
    }

    #[test]
    fn test_sort_comptime() {
        let sorted = arr_0.sort_via(|a, b| a <= b);
        assert(sorted == arr_0);
    }
}

fn main() { }