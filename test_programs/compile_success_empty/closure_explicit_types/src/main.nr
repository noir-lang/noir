fn ret_normal_lambda1() -> fn() -> Field {
    || 10
}
// return lamda that captures a thing
fn ret_closure1() -> fn[(Field,)]() -> Field {
    let x = 20;
    || x + 10
}
// return lamda that captures two things
fn ret_closure2() -> fn[(Field, Field)]() -> Field {
    let x = 20;
    let y = 10;
    || x + y + 10
}
// return lamda that captures two things with different types
fn ret_closure3() -> fn[(u32, u64)]() -> u64 {
    let x: u32 = 20;
    let y: u64 = 10;
    || x as u64 + y + 10
}
// accepts closure that has 1 thing in its env, calls it and returns the result
fn accepts_closure1(f: fn[(Field,)]() -> Field) -> Field {
    f()
}
// accepts closure that has 1 thing in its env and returns it
fn accepts_closure2(f: fn[(Field,)]() -> Field) -> fn[(Field,)]() -> Field {
    f
}
// accepts closure with different types in the capture group
fn accepts_closure3(f: fn[(u32, u64)]() -> u64) -> u64 {
    f()
}
// generic over closure environments
fn add_results<Env1, Env2>(f1: fn[Env1]() -> Field, f2: fn[Env2]() -> Field) -> Field {
    f1() + f2()
}
// a *really* generic function
fn map<T, U, let N: u32, Env>(arr: [T; N], f: fn[Env](T) -> U) -> [U; N] {
    let first_elem = f(arr[0]);
    let mut ret = [first_elem; N];

    for i in 1..N {
        ret[i] = f(arr[i]);
    }

    ret
}

fn main() {
    assert(ret_normal_lambda1()() == 10);
    assert(ret_closure1()() == 30);
    assert(ret_closure2()() == 40);
    assert(ret_closure3()() == 40);

    let x = 50;
    assert(accepts_closure1(|| x) == 50);
    assert(accepts_closure2(|| x + 10)() == 60);

    let y: u32 = 30;
    let z: u64 = 40;
    assert(accepts_closure3(|| y as u64 + z) == 70);

    let w = 50;
    assert(add_results(|| 100, || x) == 150);
    assert(add_results(|| x + 100, || w + x) == 250);

    let arr = [1, 2, 3, 4];

    assert(map(arr, |n| n + 1) == [2, 3, 4, 5]);
    assert(map(arr, |n| n + x) == [51, 52, 53, 54]);
}
