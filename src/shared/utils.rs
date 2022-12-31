use swc_core::{
    common::comments::Comments,
    ecma::ast::{JSXElement, JSXElementName, JSXObject},
};

use crate::TransformVisitor;

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

// export function registerImportMethod(path, name, moduleName) {
//     if (!imports.has(`${moduleName}:${name}`)) {
//       let id = addNamed(path, name, moduleName, {
//         nameHint: `_$${name}`
//       });
//       imports.set(`${moduleName}:${name}`, id);
//       return id;
//     } else {
//       let iden = imports.get(`${moduleName}:${name}`);
//       // the cloning is required to play well with babel-preset-env which is
//       // transpiling import as we add them and using the same identifier causes
//       // problems with the multiple identifiers of the same thing
//       return t.cloneDeep(iden);
//     }
//   }

pub fn register_import_method<C>(
    visitor: &mut TransformVisitor<C>,
    node: &mut JSXElement,
    name: &String,
    module_name: &String,
) where
    C: Comments,
{
    let key = format!("{}:{}", module_name, name);
    if !visitor.imports.contains_key(&key) {
        // let id = add_named_import(path, name, module_name, {
        //     name_hint: format!("_${}", name),
        // });
        // imports.set(`${moduleName}:${name}`, id);
        // return id;
    } else {
        // let iden = imports.get(`${moduleName}:${name}`);
        // the cloning is required to play well with babel-preset-env which is
        // transpiling import as we add them and using the same identifier causes
        // problems with the multiple identifiers of the same thing
        // return t.cloneDeep(iden);
    }
}

fn add_named_import<C>(
    visitor: &mut TransformVisitor<C>,
    node: &mut JSXElement,
    name: &String,
    imported_source: &String,
) where
    C: Comments,
{
}
