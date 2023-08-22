
fn ret_normal_lambda1() -> fn() -> Field {
    || 10
}

// explicitly specified empty capture group
fn ret_normal_lambda2() -> fn[]() -> Field {
    || 20
}

// return lamda that captures a thing
fn ret_closure1() -> fn[Field]() -> Field {
    let x = 20;
    || x + 10
}

// return lamda that captures two things
fn ret_closure2() -> fn[Field,Field]() -> Field {
    let x = 20;
    let y = 10;
    || x + y + 10
}

// return lamda that captures two things with different types
fn ret_closure3() -> fn[u32,u64]() -> u64 {
    let x: u32 = 20;
    let y: u64 = 10;
    || x as u64 + y + 10
}

// accepts closure that has 1 thing in its env, calls it and returns the result
fn accepts_closure1(f: fn[Field]() -> Field) -> Field {
    f()
}

// accepts closure that has 1 thing in its env and returns it
fn accepts_closure2(f: fn[Field]() -> Field) -> fn[Field]() -> Field {
    f
}

// accepts closure with different types in the capture group
fn accepts_closure3(f: fn[u32, u64]() -> u64) -> u64 {
    f()
}

fn main() {
    assert(ret_normal_lambda1()() == 10);
    assert(ret_normal_lambda2()() == 20);
    assert(ret_closure1()() == 30);
    assert(ret_closure2()() == 40);
    assert(ret_closure3()() == 40);

    let x = 50;
    assert(accepts_closure1(|| x) == 50);
    assert(accepts_closure2(|| x + 10)() == 60);

    let y: u32 = 30;
    let z: u64 = 40;
    assert(accepts_closure3(|| y as u64 + z) == 70);
}