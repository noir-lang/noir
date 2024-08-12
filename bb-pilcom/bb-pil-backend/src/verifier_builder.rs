use crate::file_writer::BBFiles;
use handlebars::Handlebars;
use serde_json::json;

pub trait VerifierBuilder {
    fn create_verifier_cpp(&mut self, name: &str, public_cols: &[(usize, String)]);

    fn create_verifier_hpp(&mut self, name: &str);
}

impl VerifierBuilder for BBFiles {
    fn create_verifier_cpp(&mut self, name: &str, public_cols: &[(usize, String)]) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "public_cols": public_cols.iter().map(|(idx, name)| {
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

        self.write_file(None, "verifier.cpp", &verifier_cpp);
    }

    fn create_verifier_hpp(&mut self, name: &str) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
        });

        handlebars
            .register_template_string(
                "verifier.hpp",
                std::str::from_utf8(include_bytes!("../templates/verifier.hpp.hbs")).unwrap(),
            )
            .unwrap();

        let verifier_hpp = handlebars.render("verifier.hpp", data).unwrap();

        self.write_file(None, "verifier.hpp", &verifier_hpp);
    }
}
