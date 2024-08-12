use crate::file_writer::BBFiles;
use handlebars::Handlebars;
use serde_json::json;

pub trait CircuitBuilder {
    fn create_circuit_builder_hpp(&mut self, name: &str);
    fn create_circuit_builder_cpp(&mut self, name: &str, all_cols_without_inverses: &[String]);

    fn create_full_row_hpp(&mut self, name: &str, all_cols: &[String]);
    fn create_full_row_cpp(&mut self, name: &str, all_cols: &[String]);
}

impl CircuitBuilder for BBFiles {
    fn create_circuit_builder_hpp(&mut self, name: &str) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
        });

        handlebars
            .register_template_string(
                "circuit_builder.hpp",
                std::str::from_utf8(include_bytes!("../templates/circuit_builder.hpp.hbs"))
                    .unwrap(),
            )
            .unwrap();

        let circuit_hpp = handlebars.render("circuit_builder.hpp", data).unwrap();

        self.write_file(None, "circuit_builder.hpp", &circuit_hpp);
    }

    fn create_circuit_builder_cpp(&mut self, name: &str, all_cols_without_inverses: &[String]) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "all_cols_without_inverses": all_cols_without_inverses,
        });

        handlebars
            .register_template_string(
                "circuit_builder.cpp",
                std::str::from_utf8(include_bytes!("../templates/circuit_builder.cpp.hbs"))
                    .unwrap(),
            )
            .unwrap();

        let circuit_cpp = handlebars.render("circuit_builder.cpp", data).unwrap();

        self.write_file(None, "circuit_builder.cpp", &circuit_cpp);
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

        self.write_file(None, "full_row.hpp", &hpp);
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

        self.write_file(None, "full_row.cpp", &cpp);
    }
}
