#[builtin(arraysum)]
fn sum(_input : [Field]) -> Field {}

#[builtin(arrayprod)]
fn prod(_input : [Field]) -> Field {}

#[builtin(arraylen)]
fn len<T>(_input : [T]) -> comptime Field {}

// insertion sort - n.b. it is a quadratic sort
fn sort<T>(mut a: [T]) -> [T] { 
    for i in 1..len(a) {
        for j in 0..i {
            if(a[i] < a[j]) {
                let c = a[j];
                a[j] = a[i];
                a[i]= c;
            }
        };
    };
    a
}
