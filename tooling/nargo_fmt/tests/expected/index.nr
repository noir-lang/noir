fn foo() {
    let arr = [10, 20, 30, 40];
    arr[2];
    arr[2];
    arr[/*test*/ 2];
    arr[2/*test*/];
    arr[
        // test
        2
    ];
}
