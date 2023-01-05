use crate::shared::{
    constants::{ALIASES, CHILD_PROPERTIES, SVG_ELEMENTS, VOID_ELEMENTS},
    structs::TemplateInstantiation,
    transform::TransformInfo,
    utils::get_tag_name,
};
use std::collections::HashMap;
use swc_core::{
    common::DUMMY_SP,
    ecma::{
        ast::{
            Expr, ExprStmt, Ident, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXElement,
            JSXExpr, Lit, Stmt, VarDecl, VarDeclKind,
        },
        utils::private_ident,
    },
};

pub fn transform_element_dom(node: &mut JSXElement, info: &TransformInfo) -> TemplateInstantiation {
    let tag_name = get_tag_name(node);
    let wrap_svg = info.top_level && tag_name != "svg" && SVG_ELEMENTS.contains(&tag_name.as_str());
    let void_tag = VOID_ELEMENTS.contains(&tag_name.as_str());
    let is_custom_element = tag_name.contains('-');
    let mut results = TemplateInstantiation {
        template: format!("<{}", tag_name),
        id: None,
        tag_name: tag_name.clone(),
        decl: VarDecl {
            span: DUMMY_SP,
            kind: VarDeclKind::Const,
            declare: true,
            decls: vec![],
        },
        exprs: vec![],
        dynamics: vec![],
        post_exprs: vec![],
        is_svg: wrap_svg,
        is_void: void_tag,
        has_custom_element: false,
        dynamic: false,
    };
    if wrap_svg {
        results.template = "<svg>".to_string() + &results.template;
    }
    if !info.skip_id {
        results.id = Some(private_ident!("_el$"));
    }
    transform_attributes(node, &mut results);
    results.template += ">";
    if !void_tag {
        transform_children(node, &mut results);
        results.template += &format!("</{}>", tag_name);
    }
    results
}

pub struct AttrOptions {
    pub is_svg: bool,
    pub dynamic: bool,
    pub is_custom_element: bool,
    pub prev_id: Option<Ident>,
}
pub fn set_attr(
    attr: &JSXElement,
    elem: Option<&Ident>,
    name: &str,
    value: &Expr,
    options: &AttrOptions,
) -> Option<Expr> {
    None
}

fn transform_attributes(node: &mut JSXElement, results: &mut TemplateInstantiation) {
    let elem = &results.id;
    let attributes = node.opening.attrs.clone();
    let is_svg = results.is_svg;
    let is_custom_element = results.tag_name.contains('-');
    let has_children = !node.children.is_empty();

    // preprocess spreads
    if attributes.iter().any(|attribute| match attribute {
        JSXAttrOrSpread::JSXAttr(_) => false,
        JSXAttrOrSpread::SpreadElement(_) => true,
    }) {}

    // preprocess styles

    // preprocess classList

    // combine class properties

    for attr in node.opening.attrs.clone() {
        println!("attribute");
        let attr = match attr {
            JSXAttrOrSpread::JSXAttr(attr) => attr,
            JSXAttrOrSpread::SpreadElement(_) => panic!("Spread wasn't preprocessed"),
        };

        let value = &attr.value;

        let key = match &attr.name {
            JSXAttrName::JSXNamespacedName(name) => format!("{}:{}", name.ns.sym, name.name.sym),
            JSXAttrName::Ident(name) => name.sym.as_ref().to_string(),
        };

        let value_is_lit_or_none = if let Some(value) = value {
            if let JSXAttrValue::JSXExprContainer(value) = value {
                match &value.expr {
                    JSXExpr::JSXEmptyExpr(_) => panic!("Empty expressions are not supported."),
                    JSXExpr::Expr(expr) => match expr.as_ref() {
                        Expr::Lit(_) => true,
                        _ => false,
                    },
                }
            } else {
                true
            }
        } else {
            true
        };

        println!("value_is_lit_or_none: {}", value_is_lit_or_none);

        if !value_is_lit_or_none {
        } else {
            let value = match &value {
                Some(value) => {
                    let expr = match value {
                        JSXAttrValue::JSXExprContainer(value) => match &value.expr {
                            JSXExpr::JSXEmptyExpr(_) => panic!("Empty expression not allowed"),
                            JSXExpr::Expr(expr) => match expr.as_ref() {
                                Expr::Lit(value) => value,
                                _ => panic!(),
                            },
                        },
                        JSXAttrValue::JSXElement(_) => panic!(),
                        JSXAttrValue::JSXFragment(_) => panic!(),
                        JSXAttrValue::Lit(value) => value,
                    };
                    Some(expr)
                }
                None => None,
            };

            let aliases: HashMap<&str, &str> = ALIASES.iter().cloned().collect();
            let key_str = key.as_str();
            let mut key = aliases.get(key.as_str()).unwrap_or(&key_str);

            let mut value_is_child_property = false;
            if let Some(value) = value {
                if CHILD_PROPERTIES.contains(key) {
                    value_is_child_property = true;
                    let expr = set_attr(
                        node,
                        elem.as_ref(),
                        key,
                        &Expr::Lit(value.clone()),
                        &AttrOptions {
                            is_svg,
                            dynamic: false,
                            is_custom_element,
                            prev_id: None,
                        },
                    );
                    if let Some(expr) = expr {
                        results.exprs.push(expr);
                    }
                }
            }
            if !value_is_child_property {
                let key_string: String;
                let key_str: &str;
                if !is_svg {
                    key_string = key.to_lowercase();
                    key_str = key_string.as_str();
                    key = &key_str;
                }
                results.template += &format!(" {}", key);
                if let Some(value) = value {
                    let value_as_string = match value {
                        Lit::Str(value) => value.value.to_string(),
                        Lit::Bool(value) => value.value.to_string(),
                        Lit::Null(_) => "null".to_string(),
                        Lit::Num(value) => value.value.to_string(),
                        Lit::BigInt(value) => value.value.to_string(),
                        Lit::Regex(value) => value.exp.to_string(),
                        Lit::JSXText(value) => value.raw.to_string(),
                    };
                    // results.template += &format!("=\"{}\"", escape_backticks(escape_html(value, true)));
                    results.template += &format!("=\"{}\"", value_as_string);
                }
            }
        }
    }
}

fn transform_children(node: &mut JSXElement, results: &mut TemplateInstantiation) {}
