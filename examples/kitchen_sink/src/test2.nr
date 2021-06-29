use dep::std;

fn run(x : Field, y : Field, answer : Field) {
    let arr = for i in 0..10 {
        (x + y) * i
    };
    
    constrain arr[9] == answer;
}