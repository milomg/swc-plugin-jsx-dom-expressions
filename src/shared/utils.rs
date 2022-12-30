pub fn is_component(tag_name: &String) -> bool {
    let first_char = tag_name.chars().next().unwrap();
    let first_char_lower = first_char.to_lowercase().to_string();
    let has_dot = tag_name.contains(".");
    let has_non_alpha = !first_char.is_alphabetic();
    first_char_lower != first_char.to_string() || has_dot || has_non_alpha
}
