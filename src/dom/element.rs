use std::collections::HashMap;

use crate::shared::{
    constants::{ALIASES, CHILD_PROPERTIES, SVG_ELEMENTS, VOID_ELEMENTS},
    structs::Template,
    transform::TransformInfo,
    utils::get_tag_name,
};
use swc_core::ecma::ast::{
    Expr, ExprStmt, Ident, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXElement,
    JSXExpr, Stmt,
};

pub fn transform_element_dom(node: &mut JSXElement, info: &TransformInfo) {
    let tag_name = get_tag_name(node);
    let wrap_svg = info.top_level && tag_name != "svg" && SVG_ELEMENTS.contains(&tag_name.as_str());
    let void_tag = VOID_ELEMENTS.contains(&tag_name.as_str());
    let is_custom_element = tag_name.contains("-");
    let mut results = Template {
        template: format!("<{}", tag_name),
        tag_name,
        decl: vec![],
        exprs: vec![],
        dynamics: vec![],
        tag_count: 0.0,
        is_svg: wrap_svg,
        is_void: void_tag,
        id: Ident::new("".into(), Default::default()),
    };
    if wrap_svg {
        results.template = format!("{}{}", "<svg>", results.template);
    }
    transform_attributes(node, &mut results);
}

fn set_attr(
    attr: &JSXAttr,
    elem: &Ident,
    name: &&str,
    value: &Box<Expr>,
    isSVG: bool,
    dynamic: bool,
    isCE: bool,
    prev_id: Option<&Ident>,
) -> Option<Expr> {
    return None;
}

fn transform_attributes(node: &mut JSXElement, results: &mut Template) {
    let elem = &results.id;
    let attributes = node.opening.attrs.clone();
    let is_svg = results.is_svg;
    let is_custom_element = results.tag_name.contains("-");
    let has_children = node.children.len() > 0;

    // preprocess spreads
    if attributes.iter().any(|attribute| match attribute {
        JSXAttrOrSpread::JSXAttr(_) => false,
        JSXAttrOrSpread::SpreadElement(_) => true,
    }) {}

    // preprocess styles

    // preprocess classList

    // combine class properties

    for attr in node.opening.attrs.clone() {
        let attr = match attr {
            JSXAttrOrSpread::JSXAttr(attr) => attr,
            JSXAttrOrSpread::SpreadElement(_) => panic!("Spread wasn't preprocessed"),
        };

        let value = &attr.value;

        let key = match &attr.name {
            JSXAttrName::JSXNamespacedName(name) => format!("{}:{}", name.ns.sym, name.name.sym),
            JSXAttrName::Ident(name) => name.sym.as_ref().to_string(),
        };

        let t: Box<Expr>;
        let value = match value {
            Some(value) => {
                let expr = match value {
                    JSXAttrValue::JSXExprContainer(value) => match &value.expr {
                        JSXExpr::JSXEmptyExpr(_) => panic!("Empty expression not allowed"),
                        JSXExpr::Expr(expr) => expr,
                    },
                    JSXAttrValue::JSXElement(value) => {
                        t = Box::new(Expr::JSXElement(value.clone()));
                        &t
                    }
                    JSXAttrValue::JSXFragment(value) => {
                        t = Box::new(Expr::JSXFragment(value.clone()));
                        &t
                    }
                    JSXAttrValue::Lit(value) => {
                        t = Box::new(Expr::Lit(value.clone()));
                        &t
                    }
                };
                Some(expr)
            }
            None => None,
        };

        let aliases: HashMap<&str, &str> = ALIASES.iter().cloned().collect();
        let key_str = key.as_str();
        let mut key = aliases.get(key.as_str()).unwrap_or(&key_str);

        if let Some(value) = value {
            if CHILD_PROPERTIES.contains(key) {
                let expr = set_attr(
                    &attr,
                    elem,
                    key,
                    value,
                    is_svg,
                    false,
                    is_custom_element,
                    None,
                );
                if let Some(expr) = expr {
                    let expr_statement = ExprStmt {
                        span: Default::default(),
                        expr: Box::new(expr),
                    };
                    results.exprs.push(Stmt::Expr(expr_statement));
                }
            }
        } else {
            let key_string: String;
            let key_str: &str;
            if !is_svg {
                key_string = key.to_lowercase();
                key_str = key_string.as_str();
                key = &key_str;
            }
            results.template += &format!(" {}", key);
            if let Some(value) = value {
                // results.template += &format!("=\"{}\"", escape_backticks(escape_html(value, true)));
            }
        }
    }
}
