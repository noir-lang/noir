use crate::file_writer::BBFiles;
use handlebars::{handlebars_helper, Handlebars};
use itertools::Itertools;
use serde_json::json;

pub trait FlavorBuilder {
    #[allow(clippy::too_many_arguments)]
    fn create_flavor_hpp(
        &mut self,
        name: &str,
        relation_file_names: &[String],
        inverses: &[String],
        lookups: &[String],
        fixed: &[String],
        witness: &[String],
        witness_without_inverses: &[String],
        all_cols: &[String],
        to_be_shifted: &[String],
        shifted: &[String],
        all_cols_and_shifts: &[String],
    );

    fn create_flavor_cpp(
        &mut self,
        name: &str,
        relation_file_names: &[String],
        inverses: &[String],
        fixed: &[String],
        witness: &[String],
        witness_without_inverses: &[String],
        all_cols: &[String],
        to_be_shifted: &[String],
        shifted: &[String],
        all_cols_and_shifts: &[String],
    );

    fn create_flavor_settings_hpp(&mut self, name: &str);
}

/// Build the boilerplate for the flavor file
impl FlavorBuilder for BBFiles {
    fn create_flavor_hpp(
        &mut self,
        name: &str,
        relation_file_names: &[String],
        lookups: &[String],
        inverses: &[String],
        fixed: &[String],
        witness: &[String],
        witness_without_inverses: &[String],
        all_cols: &[String],
        to_be_shifted: &[String],
        shifted: &[String],
        all_cols_and_shifts: &[String],
    ) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "relation_file_names": relation_file_names,
            "lookups": lookups,
            "inverses": inverses,
            "fixed": fixed,
            "witness": witness,
            "all_cols": all_cols,
            "to_be_shifted": to_be_shifted,
            "shifted": shifted,
            "all_cols_and_shifts": all_cols_and_shifts,
            "witness_without_inverses": witness_without_inverses,
        });

        handlebars_helper!(join: |*args|
            args.iter().map(|v| v.as_array().unwrap().to_owned()).collect_vec().concat()
        );
        handlebars.register_helper("join", Box::new(join));

        handlebars
            .register_template_string(
                "flavor.hpp",
                std::str::from_utf8(include_bytes!("../templates/flavor.hpp.hbs")).unwrap(),
            )
            .unwrap();

        let flavor_hpp = handlebars.render("flavor.hpp", data).unwrap();

        self.write_file(None, "flavor.hpp", &flavor_hpp);
    }

    fn create_flavor_cpp(
        &mut self,
        name: &str,
        relation_file_names: &[String],
        inverses: &[String],
        fixed: &[String],
        witness: &[String],
        witness_without_inverses: &[String],
        all_cols: &[String],
        to_be_shifted: &[String],
        shifted: &[String],
        all_cols_and_shifts: &[String],
    ) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
            "relation_file_names": relation_file_names,
            "inverses": inverses,
            "fixed": fixed,
            "witness": witness,
            "all_cols": all_cols,
            "to_be_shifted": to_be_shifted,
            "shifted": shifted,
            "all_cols_and_shifts": all_cols_and_shifts,
            "witness_without_inverses": witness_without_inverses,
        });

        handlebars_helper!(join: |*args|
            args.iter().map(|v| v.as_array().unwrap().to_owned()).collect_vec().concat()
        );
        handlebars.register_helper("join", Box::new(join));

        handlebars
            .register_template_string(
                "flavor.cpp",
                std::str::from_utf8(include_bytes!("../templates/flavor.cpp.hbs")).unwrap(),
            )
            .unwrap();

        let flavor_cpp = handlebars.render("flavor.cpp", data).unwrap();

        self.write_file(None, "flavor.cpp", &flavor_cpp);
    }

    fn create_flavor_settings_hpp(&mut self, name: &str) {
        let mut handlebars = Handlebars::new();

        let data = &json!({
            "name": name,
        });

        handlebars
            .register_template_string(
                "flavor_settings.hpp",
                std::str::from_utf8(include_bytes!("../templates/flavor_settings.hpp.hbs"))
                    .unwrap(),
            )
            .unwrap();

        let flavor_hpp = handlebars.render("flavor_settings.hpp", data).unwrap();

        self.write_file(None, "flavor_settings.hpp", &flavor_hpp);
    }
}
