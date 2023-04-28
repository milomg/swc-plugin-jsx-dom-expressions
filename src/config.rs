use serde::{Deserialize, Serialize};
// use serde_json::json;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_module_name")]
    pub module_name: String,
    pub generate: String,
    pub hydratable: bool,
    pub delegate_events: bool,
    pub delegated_events: Vec<String>,
    #[serde(default)]
    pub built_ins: Vec<String>,
    pub require_import_source: bool,
    pub wrap_conditionals: bool,
    pub omit_nested_closing_tags: bool,
    pub context_to_custom_elements: bool,
    pub static_marker: String,
    pub effect_wrapper: String,
    pub memo_wrapper: String,
    pub validate: bool,
}

fn default_module_name() -> String {
    "dom".to_string()
}

impl Default for Config {
    fn default() -> Self {
        // todo!("change default");
        Config {
            module_name: "dom".to_owned(),
            generate: "dom".to_owned(),
            hydratable: false,
            delegate_events: true,
            delegated_events: vec![],
            built_ins: vec![],
            require_import_source: false,
            wrap_conditionals: true,
            omit_nested_closing_tags: false,
            context_to_custom_elements: false,
            static_marker: "@once".to_owned(),
            effect_wrapper: "effect".to_owned(),
            memo_wrapper: "memo".to_owned(),
            validate: true
        }
    }
}
