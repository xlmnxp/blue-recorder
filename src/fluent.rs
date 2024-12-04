use fluent_bundle::bundle::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use std::path::Path;

// Translate
pub fn get_bundle(message_id: &str, arg: Option<&FluentArgs>) -> String {
    let mut ftl_path = {
        let mut current_exec_dir = std::env::current_exe().unwrap();
        current_exec_dir.pop();
        current_exec_dir
    }.join(Path::new("locales"));
    if !ftl_path.exists() {
        let var = std::env::var("LC_DIR");
        ftl_path = std::fs::canonicalize(Path::new(
            &var.unwrap_or_else(|_| String::from("locales")),
        )).unwrap();
    }
    let supported_lang: Vec<String> = std::fs::read_dir(&ftl_path)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            path.file_stem().unwrap().to_string_lossy().to_string()
        }).collect();
    let mut locale = std::env::var("LANG").unwrap_or("en_US".to_string());
    if !supported_lang.contains(&locale) {
        locale = locale.split('_').next().unwrap().to_string();
        if !supported_lang.contains(&locale) {
            locale = String::from("en_US");
        }
    }
    let ftl_in_loacle = ftl_path.join(Path::new(&locale.split('.').next().unwrap()));
    let ftl_file = std::fs::read_to_string(
        format!("{}.ftl", ftl_in_loacle.to_str().unwrap())
    ).unwrap();
    let res = FluentResource::try_new(ftl_file).unwrap();
    let mut bundle = FluentBundle::default();
    bundle.add_resource(res).expect("Failed to add localization resources to the bundle.");
    bundle.format_pattern(bundle.get_message(message_id)
                          .unwrap().value().unwrap(), arg, &mut vec![]).to_string()
}
