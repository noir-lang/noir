use crate::{file_writer::BBFiles, utils::snake_case};
use handlebars::Handlebars;
use serde_json::json;

pub trait VerifierBuilder {
    fn create_verifier_cpp(
        &mut self,
        name: &str,
        inverses: &[String],
        public_cols: &[(String, usize)],
    );

    fn create_verifier_hpp(&mut self, name: &str, public_cols: &[(String, usize)]);
}

impl VerifierBuilder for BBFiles {
    fn create_verifier_cpp(
        &mut self,
        name: &str,
        inverses: &[String],
        public_cols: &[(String, usize)],
    ) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "inverses": inverses,
            "public_cols": public_cols.iter().map(|(name, idx)| {
                json!({
                    "col": name,
                    "idx": idx,
                })
            }).collect::<Vec<_>>()
        });

        handlebars
            .register_template_string(
                "verifier.cpp",
                std::str::from_utf8(include_bytes!("../templates/verifier.cpp.hbs")).unwrap(),
            )
            .unwrap();

        let verifier_cpp = handlebars.render("verifier.cpp", data).unwrap();

        self.write_file(
            &self.prover,
            &format!("{}_verifier.cpp", snake_case(name)),
            &verifier_cpp,
        );
    }

    fn create_verifier_hpp(&mut self, name: &str, public_cols: &[(String, usize)]) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "public_cols": public_cols,
        });

        handlebars
            .register_template_string(
                "verifier.hpp",
                std::str::from_utf8(include_bytes!("../templates/verifier.hpp.hbs")).unwrap(),
            )
            .unwrap();

        let verifier_hpp = handlebars.render("verifier.hpp", data).unwrap();

        self.write_file(
            &self.prover,
            &format!("{}_verifier.hpp", snake_case(name)),
            &verifier_hpp,
        );
    }
}
