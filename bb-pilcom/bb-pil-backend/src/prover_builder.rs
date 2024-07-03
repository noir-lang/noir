use crate::{file_writer::BBFiles, utils::snake_case};
use handlebars::Handlebars;
use serde_json::json;

pub trait ProverBuilder {
    fn create_prover_hpp(&mut self, name: &str);

    fn create_prover_cpp(&mut self, name: &str, lookup_names: &[String]);
}

impl ProverBuilder for BBFiles {
    fn create_prover_hpp(&mut self, name: &str) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
        });

        handlebars
            .register_template_string(
                "prover.hpp",
                std::str::from_utf8(include_bytes!("../templates/prover.hpp.hbs")).unwrap(),
            )
            .unwrap();

        let prover_hpp = handlebars.render("prover.hpp", data).unwrap();

        self.write_file(
            &self.prover,
            &format!("{}_prover.hpp", snake_case(name)),
            &prover_hpp,
        );
    }

    fn create_prover_cpp(&mut self, name: &str, lookup_names: &[String]) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "lookups": lookup_names,
        });

        handlebars
            .register_template_string(
                "prover.cpp",
                std::str::from_utf8(include_bytes!("../templates/prover.cpp.hbs")).unwrap(),
            )
            .unwrap();

        let prover_cpp = handlebars.render("prover.cpp", data).unwrap();

        self.write_file(
            &self.prover,
            &format!("{}_prover.cpp", snake_case(name)),
            &prover_cpp,
        );
    }
}
