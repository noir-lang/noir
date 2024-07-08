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
    );

    fn create_full_row_hpp(&mut self, name: &str, all_cols: &[String]);
    fn create_full_row_cpp(&mut self, name: &str, all_cols: &[String]);
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
    ) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "relations": relations,
            "permutations": permutations,
            "all_cols_without_inverses": all_cols_without_inverses,
            "all_cols": all_cols,
            "to_be_shifted": to_be_shifted,
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

    fn create_full_row_hpp(&mut self, name: &str, all_cols: &[String]) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "all_cols": all_cols,
        });

        handlebars
            .register_template_string(
                "full_row.hpp",
                std::str::from_utf8(include_bytes!("../templates/full_row.hpp.hbs")).unwrap(),
            )
            .unwrap();

        let hpp = handlebars.render("full_row.hpp", data).unwrap();

        self.write_file(
            &self.circuit,
            &format!("{}_full_row.hpp", snake_case(name)),
            &hpp,
        );
    }

    fn create_full_row_cpp(&mut self, name: &str, all_cols: &[String]) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "all_cols": all_cols,
        });

        handlebars
            .register_template_string(
                "full_row.cpp",
                std::str::from_utf8(include_bytes!("../templates/full_row.cpp.hbs")).unwrap(),
            )
            .unwrap();

        let cpp = handlebars.render("full_row.cpp", data).unwrap();

        self.write_file(
            &self.circuit,
            &format!("{}_full_row.cpp", snake_case(name)),
            &cpp,
        );
    }
}
