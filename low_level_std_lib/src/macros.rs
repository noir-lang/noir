// We want to be able to iterate an enum, so this macro will be used
// Code taken from: https://github.com/rust-lang/rfcs/issues/284#issuecomment-277871931

macro_rules! iterable_enum {
    ($name:ident { $($variant:ident),* })   => (
        #[derive(Debug, Clone)]
        pub enum $name { $($variant),* }
        
        impl $name {
            pub fn iter() -> Iter {
                Iter(None)
            }
        }
        
        pub struct Iter(Option<$name>);
        
        impl Iterator for Iter {
            type Item = $name;
            
            fn next(&mut self) -> Option<Self::Item> {
                match self.0 {
                    None                    => $( { self.0 = Some($name::$variant); Some($name::$variant) },
                    Some($name::$variant)   => )* None,
                }
            }
        }
    );
}
