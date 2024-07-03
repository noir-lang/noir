use crate::{file_writer::BBFiles, utils::snake_case};
use handlebars::Handlebars;
use serde_json::json;

pub trait CircuitBuilder {
    fn create_circuit_builder_hpp(
        &mut self,
        name: &str,
        relations: &[String],
        permutations: &[String],
        all_cols_without_inverses: &[String],
        all_cols: &[String],
        to_be_shifted: &[String],
        all_cols_with_shifts: &[String],
    );

    fn create_circuit_builder_cpp(&mut self, name: &str, all_cols: &[String]);
}

impl CircuitBuilder for BBFiles {
    fn create_circuit_builder_hpp(
        &mut self,
        name: &str,
        relations: &[String],
        permutations: &[String],
        all_cols_without_inverses: &[String],
        all_cols: &[String],
        to_be_shifted: &[String],
        all_cols_with_shifts: &[String],
    ) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "relations": relations,
            "permutations": permutations,
            "all_cols_without_inverses": all_cols_without_inverses,
            "all_cols": all_cols,
            "to_be_shifted": to_be_shifted,
            "all_cols_with_shifts": all_cols_with_shifts,
        });

        handlebars
            .register_template_string(
                "circuit_builder.hpp",
                std::str::from_utf8(include_bytes!("../templates/circuit_builder.hpp.hbs"))
                    .unwrap(),
            )
            .unwrap();

        let circuit_hpp = handlebars.render("circuit_builder.hpp", data).unwrap();

        self.write_file(
            &self.circuit,
            &format!("{}_circuit_builder.hpp", snake_case(name)),
            &circuit_hpp,
        );
    }

    fn create_circuit_builder_cpp(&mut self, name: &str, all_cols: &[String]) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "all_cols": all_cols,
        });

        handlebars
            .register_template_string(
                "circuit_builder.cpp",
                std::str::from_utf8(include_bytes!("../templates/circuit_builder.cpp.hbs"))
                    .unwrap(),
            )
            .unwrap();

        let circuit_cpp = handlebars.render("circuit_builder.cpp", data).unwrap();

        self.write_file(
            &self.circuit,
            &format!("{}_circuit_builder.cpp", snake_case(name)),
            &circuit_cpp,
        );
    }
}
