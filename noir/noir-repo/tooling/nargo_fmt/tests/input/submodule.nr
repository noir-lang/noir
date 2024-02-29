mod a {
// hello
}

mod a {
// hello
    mod b {
// hello
    }
}

mod a {mod b {mod c {}}}

mod a {
// 1
// 2
// 3
/*test*/
}

mod a {
    /**/ mod b {
        mod c {}
    }
}
