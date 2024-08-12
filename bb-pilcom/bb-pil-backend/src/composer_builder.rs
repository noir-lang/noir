use crate::file_writer::BBFiles;
use handlebars::Handlebars;
use serde_json::json;

pub trait ComposerBuilder {
    fn create_composer_cpp(&mut self, name: &str);
    fn create_composer_hpp(&mut self, name: &str);
}

impl ComposerBuilder for BBFiles {
    fn create_composer_cpp(&mut self, name: &str) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
        });

        handlebars
            .register_template_string(
                "composer.cpp",
                std::str::from_utf8(include_bytes!("../templates/composer.cpp.hbs")).unwrap(),
            )
            .unwrap();

        let composer_cpp = handlebars.render("composer.cpp", data).unwrap();

        self.write_file(None, "composer.cpp", &composer_cpp);
    }

    fn create_composer_hpp(&mut self, name: &str) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
        });

        handlebars
            .register_template_string(
                "composer.hpp",
                std::str::from_utf8(include_bytes!("../templates/composer.hpp.hbs")).unwrap(),
            )
            .unwrap();

        let composer_hpp = handlebars.render("composer.hpp", data).unwrap();

        self.write_file(None, "composer.hpp", &composer_hpp);
    }
}
