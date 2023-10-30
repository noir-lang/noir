//@error_on_lost_comment=false
fn foo() {
    40 + 2;
    !40+2;
    40 +  2 == 42;

40/*test*/ + 2 == 42;
                40 + 2/*test*/ == 42;
}
