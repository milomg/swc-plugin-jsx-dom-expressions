use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_module_name")]
    pub module_name: String,
    #[serde(default)]
    pub builtins: Vec<String>,
}

fn default_module_name() -> String {
    "dom".to_string()
}

impl Default for Config {
    fn default() -> Self {
        serde_json::from_value(json!({})).unwrap()
    }
}
