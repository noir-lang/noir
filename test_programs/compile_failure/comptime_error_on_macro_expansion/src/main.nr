mod other;
use other::expr;

#[foo]
comptime fn foo(_: FunctionDefinition) -> Quoted {
    quote {
        pub fn generated_by_foo() {
            1 + "a";
        }
    }
}

#[bar]
comptime fn bar(_: FunctionDefinition) -> Quoted {
    let expr = expr();
    quote {
        pub fn generated_by_bar() {
            $expr;
        }
    }
}

#[derive_bn254_impl]
pub struct BN254 {}

comptime fn derive_bn254_impl(s: TypeDefinition) -> Quoted {
    let typ = s.as_type();
    quote {
        impl BN254 {
            fn one() {}

            fn mul<let NScalarSlices: u32>(self, scalar: ScalarField<NScalarSlices>) -> Self {
                unconstrained_function();
                crate::mul_with_hint($typ, scalar, transcript)
            }
        }
    }
}

pub unconstrained fn unconstrained_function() {}

fn main() {}
