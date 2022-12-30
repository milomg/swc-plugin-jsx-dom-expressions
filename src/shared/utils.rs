use swc_core::ecma::ast::{JSXElement, JSXElementName, JSXObject};

pub fn is_component(tag_name: &String) -> bool {
    let first_char = tag_name.chars().next().unwrap();
    let first_char_lower = first_char.to_lowercase().to_string();
    let has_dot = tag_name.contains(".");
    let has_non_alpha = !first_char.is_alphabetic();
    first_char_lower != first_char.to_string() || has_dot || has_non_alpha
}

pub fn get_tag_name(element: &mut JSXElement) -> String {
    let jsx_name = &element.opening.name;
    match jsx_name {
        JSXElementName::Ident(ident) => ident.sym.to_string(),
        JSXElementName::JSXMemberExpr(member) => {
            let mut name = member.prop.sym.to_string();
            let mut obj = &member.obj;
            while let JSXObject::JSXMemberExpr(member) = obj {
                name = format!("{}.{}", member.prop.sym.to_string(), name);
                obj = &member.obj;
            }
            name = format!("{}.{}", member.prop.sym.to_string(), name);
            name
        }
        JSXElementName::JSXNamespacedName(name) => {
            format!("{}:{}", name.ns.sym.to_string(), name.name.sym.to_string())
        }
    }
}
