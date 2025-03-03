#![cfg(test)]

use crate::elaborator::UnstableFeature;

use crate::hir::comptime::InterpreterError;
use crate::hir::def_collector::{dc_crate::CompilationError, errors::DefCollectorErrorKind};
use crate::hir::resolution::{errors::ResolverError, import::PathResolutionError};

use crate::tests::{get_program_using_features, get_program_errors};

#[test]
fn cfg_attribute_on_function() {
    let src = r#"
        #[cfg(feature = "default")]
        fn foo() { }

        fn main() {
            foo();
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors, vec![]);
}

#[test]
fn cfg_disabled_attribute_on_function() {
    let src = r#"
        #[cfg(feature = "foo")]
        fn foo() {
            let result = unresolved_function(unresolved_variable)
                .unresolved_method::<UnresolvedType>();
            result
        }

        fn main() { }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors, vec![]);
}

#[test]
fn cfg_disabled_attribute_on_function_rejects_parse_error() {
    let src = r#"
        #[cfg(feature = "foo")]
        fn foo() {
            unmatched parentheses -> )
        }

        fn main() { }
    "#;
    let errors = get_program_errors(src);
    for error in errors {
        assert!(matches!(error, CompilationError::ParseError(_)));
    }
}

#[test]
fn cfg_disabled_attribute_on_global() {
    let src = r#"
        #[cfg(feature = "foo")]
        global FOO: bool = true;

        fn main() { }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors, vec![]);
}

#[test]
fn cfg_attribute_on_global() {
    let src = r#"
        #[cfg(feature = "default")]
        global FOO: bool = true;

        fn main() {
            let _ = FOO;
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors, vec![]);
}

#[test]
fn cfg_disabled_attribute_on_statement_block() {
    let src = r#"
        fn foo() -> Field {
            let mut result = 0;

            #[cfg(feature = "bar")]
            {
                result = 1;
            }

            result
        }

        fn main() {
            let _ = foo() == 0;
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors, vec![]);
}

#[test]
fn cfg_attribute_on_statement_block() {
    let src = r#"
        fn foo() -> Field {
            let mut result = 0;

            #[cfg(feature = "default")]
            {
                result = 1;
            }

            result
        }

        fn main() {
            let _ = foo() == 1;
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors, vec![]);
}

#[test]
fn cfg_attribute_on_module() {
    let src = r#"
        #[cfg(feature = "default")]
        mod foo_module {
            pub global FOO: bool = true;
        }

        fn main() {
            let _ = foo_module::FOO;
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors, vec![]);
}

#[test]
fn cfg_disabled_attribute_on_module() {
    let src = r#"
        #[cfg(feature = "foo")]
        mod foo_module {
            pub global FOO: bool = true;
        }

        fn main() {
            let _ = foo_module::FOO;
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        CompilationError::ResolverError(ResolverError::PathResolutionError(PathResolutionError::Unresolved(unresolved_path))) => {
            assert_eq!(unresolved_path, "foo_module");
        }
        other_error => panic!("expected a ResolverError::PathResolutionError, but found {other_error}"),
    }
}

#[test]
fn cfg_attribute_on_use() {
    let src = r#"
        mod foo_module {
            pub global FOO: bool = true;
        }

        #[cfg(feature = "default")]
        use foo_module::FOO;

        fn main() {
            let _ = FOO;
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors, vec![]);
}

#[test]
fn cfg_disabled_attribute_on_use() {
    let src = r#"
        mod foo_module {
            pub global FOO: bool = true;
        }

        #[cfg(feature = "foo")]
        use foo_module::FOO;

        fn main() {
            // this is out of scope:
            // let _ = FOO;
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors, vec![]);
}

#[test]
fn cfg_disabled_attribute_on_use_errors_on_use() {
    let src = r#"
        mod foo_module {
            pub global FOO: bool = true;
        }

        #[cfg(feature = "foo")]
        use foo_module::FOO;

        fn main() {
            // this is out of scope:
            let _ = FOO;
        }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    match &errors[0] {
        CompilationError::ResolverError(ResolverError::VariableNotDeclared { name, .. }) => {
            assert_eq!(name, "FOO");
        }
        other_error => panic!("expected a ResolverError::VariableNotDeclared, but found {other_error}"),
    }
}

#[test]
fn cfg_attribute_on_trait_and_impl() {
    let feature_options = ["default", "disabled_feature"];
    let is_disabled = |feature: &str| { feature == "disabled_feature" };
    let src_templates: Vec<_> = feature_options.iter().flat_map(|trait_feature| {
        feature_options.iter().flat_map(|impl_feature| {
            let expect_error =
                is_disabled(trait_feature) && !is_disabled(impl_feature);
            std::iter::once((trait_feature.to_string(), impl_feature.to_string(), expect_error))
        })
    }).collect();
    let src_template_fn = |trait_feature: &str, impl_feature: &str| { format!(r#"
        #[cfg(feature = "{trait_feature}")]
        pub trait Foo {{
            fn bar(self) -> Field;
        }}

        #[cfg(feature = "{impl_feature}")]
        impl Foo for Field {{
            fn bar(self) -> Field {{
                self
            }}
        }}

        fn main() {{ }}
    "#) };
    for src_template in src_templates {
        let (trait_feature, impl_feature, expect_error) = src_template;
        let src = src_template_fn(&trait_feature, &impl_feature);
        let errors = get_program_errors(&src);
        if expect_error {
            assert_eq!(errors.len(), 1);
            assert!(matches!(errors[0], CompilationError::DefinitionError(DefCollectorErrorKind::TraitNotFound { .. })));
        } else {
            assert_eq!(errors, vec![]);
        }
    }
}

#[test]
fn cfg_attribute_on_struct() {
    let feature_options = ["default", "disabled_feature"];
    let is_disabled = |feature: &str| { feature == "disabled_feature" };
    let src_templates: Vec<_> = feature_options.iter().flat_map(|struct_feature| {
        feature_options.iter().flat_map(|bar_feature| {
            let expect_error =
                is_disabled(struct_feature) && !is_disabled(bar_feature);
            std::iter::once((struct_feature.to_string(), bar_feature.to_string(), expect_error))
        })
    }).collect();
    let src_template_fn = |struct_feature: &str, bar_feature: &str| { format!(r#"
        #[cfg(feature = "{struct_feature}")]
        pub struct Foo {{
            bar: Field,
        }}
        
        #[cfg(feature = "{bar_feature}")]
        pub fn foo(x: Foo) -> Field {{
            x.bar
        }}
        
        fn main() {{ }}
    "#) };
    for src_template in src_templates {
        let (struct_feature, bar_feature, expect_error) = src_template;
        let src = src_template_fn(&struct_feature, &bar_feature);
        let errors = get_program_errors(&src);
        if expect_error {
            assert_eq!(errors.len(), 1);
            match &errors[0] {
                CompilationError::ResolverError(ResolverError::PathResolutionError(PathResolutionError::Unresolved(unresolved_path))) => {
                    assert_eq!(unresolved_path, "Foo");
                }
                other_error => panic!("expected a ResolverError::PathResolutionError, but found {other_error}"),
            }
        } else {
            assert_eq!(errors, vec![]);
        }
    }
}

#[test]
fn cfg_attribute_on_enum() {
    let feature_options = ["default", "disabled_feature"];
    let is_disabled = |feature: &str| { feature == "disabled_feature" };
    let src_templates: Vec<_> = feature_options.iter().flat_map(|enum_feature| {
        feature_options.iter().flat_map(|bar_feature| {
            let expect_error =
                is_disabled(enum_feature) && !is_disabled(bar_feature);
            std::iter::once((enum_feature.to_string(), bar_feature.to_string(), expect_error))
        })
    }).collect();
    let src_template_fn = |enum_feature: &str, bar_feature: &str| { format!(r#"
        #[cfg(feature = "{enum_feature}")]
        pub enum Foo {{
            Bar(Field),
        }}
        
        #[cfg(feature = "{bar_feature}")]
        pub fn foo(x: Foo) -> Field {{
            match x {{
                Foo::Bar(y) => y,
            }}
        }}
        
        fn main() {{ }}
    "#) };
    for src_template in src_templates {
        let (enum_feature, bar_feature, expect_error) = src_template;
        let src = src_template_fn(&enum_feature, &bar_feature);
        let features = &[UnstableFeature::Enums];
        let errors = get_program_using_features(&src, features).2;
        if expect_error {
            assert_eq!(errors.len(), 3);
            // Foo not declared (1st occurrence)
            assert!(matches!(&errors[0], CompilationError::ResolverError(ResolverError::PathResolutionError(PathResolutionError::Unresolved(_)))));
            // Foo not declared (2nd occurrence)
            assert!(matches!(&errors[1], CompilationError::ResolverError(ResolverError::PathResolutionError(PathResolutionError::Unresolved(_)))));
            // 'y' not declared
            assert!(matches!(&errors[2], CompilationError::ResolverError(ResolverError::VariableNotDeclared { .. })));
        } else {
            assert_eq!(errors, vec![]);
        }
    }
}

#[test]
fn cfg_attribute_on_alias() {
    let feature_options = ["default", "disabled_feature"];
    let is_disabled = |feature: &str| { feature == "disabled_feature" };
    let src_templates: Vec<_> = feature_options.iter().flat_map(|alias_feature| {
        feature_options.iter().flat_map(|bar_feature| {
            let expect_error =
                is_disabled(alias_feature) && !is_disabled(bar_feature);
            std::iter::once((alias_feature.to_string(), bar_feature.to_string(), expect_error))
        })
    }).collect();
    let src_template_fn = |alias_feature: &str, bar_feature: &str| { format!(r#"
        #[cfg(feature = "{alias_feature}")]
        pub type Foo = Field;
        
        #[cfg(feature = "{bar_feature}")]
        pub fn bar(x: Foo) -> Field {{
            x
        }}
        
        fn main() {{ }}
    "#) };
    for src_template in src_templates {
        let (alias_feature, bar_feature, expect_error) = src_template;
        let src = src_template_fn(&alias_feature, &bar_feature);
        let errors = get_program_errors(&src);
        if expect_error {
            assert_eq!(errors.len(), 1);
            match &errors[0] {
                CompilationError::ResolverError(ResolverError::PathResolutionError(PathResolutionError::Unresolved(unresolved_path))) => {
                    assert_eq!(unresolved_path, "Foo");
                }
                other_error => panic!("expected a ResolverError::PathResolutionError, but found {other_error}"),
            }
        } else {
            assert_eq!(errors, vec![]);
        }
    }
}

#[test]
fn cfg_attribute_on_comptime() {
    let feature_options = ["default", "disabled_feature"];
    let is_disabled = |feature: &str| { feature == "disabled_feature" };
    let src_templates: Vec<_> = feature_options.iter().flat_map(|comptime_feature| {
        feature_options.iter().flat_map(|bar_feature| {
            // TODO: make follow-up issue to prevent error when:
            // - comptime_feature is disabled
            // - bar_feature is enabled
            let expect_error =
                is_disabled(comptime_feature);
                // && !is_disabled(bar_feature);
            std::iter::once((comptime_feature.to_string(), bar_feature.to_string(), expect_error))
        })
    }).collect();
    let src_template_fn = |comptime_feature: &str, bar_feature: &str| { format!(r#"
        #[cfg(feature = "{comptime_feature}")]
        comptime fn foo_generator() -> Field {{
            2
        }}
        
        fn main() {{
            #[cfg(feature = "{bar_feature}")]
            comptime let foo: Field = foo_generator();
            
            #[cfg(feature = "{bar_feature}")]
            assert_eq(foo, 2);
        }}
    "#) };
    for src_template in src_templates {
        let (comptime_feature, bar_feature, expect_error) = src_template;
        let src = src_template_fn(&comptime_feature, &bar_feature);
        let errors = get_program_errors(&src);
        if expect_error {
            assert_eq!(errors.len(), 2);
            assert!(matches!(&errors[0], CompilationError::ResolverError(ResolverError::VariableNotDeclared { .. })));
            assert!(matches!(&errors[1], CompilationError::InterpreterError(InterpreterError::VariableNotInScope { .. })));
        } else {
            assert_eq!(errors, vec![]);
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// END cfg tests
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////
