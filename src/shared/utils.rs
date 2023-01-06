use crate::TransformVisitor;
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::{
        ast::*,
        utils::{prepend_stmt, private_ident},
        visit::{Visit, VisitWith},
    },
};

use super::structs::{ImmutableChildTemplateInstantiation};

pub fn is_component(tag_name: &str) -> bool {
    let first_char = tag_name.chars().next().unwrap();
    let first_char_lower = first_char.to_lowercase().to_string();
    let has_dot = tag_name.contains('.');
    let has_non_alpha = !first_char.is_alphabetic();
    first_char_lower != first_char.to_string() || has_dot || has_non_alpha
}

pub fn get_tag_name(element: &JSXElement) -> String {
    let jsx_name = &element.opening.name;
    match jsx_name {
        JSXElementName::Ident(ident) => ident.sym.to_string(),
        JSXElementName::JSXMemberExpr(member) => {
            let mut name = member.prop.sym.to_string();
            let mut obj = &member.obj;
            while let JSXObject::JSXMemberExpr(member) = obj {
                name = format!("{}.{}", member.prop.sym, name);
                obj = &member.obj;
            }
            name = format!("{}.{}", member.prop.sym, name);
            name
        }
        JSXElementName::JSXNamespacedName(name) => {
            format!("{}:{}", name.ns.sym, name.name.sym)
        }
    }
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn register_import_method(&mut self, name: &str) -> Ident {
        self.imports
            .entry(name.to_string())
            .or_insert_with(|| private_ident!(format!("_${}", name)))
            .clone()
    }

    pub fn insert_imports(&mut self, module: &mut Module) {
        self.imports.drain().for_each(|(name, val)| {
            prepend_stmt(
                &mut module.body,
                ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                    specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                        local: val,
                        imported: Some(ModuleExportName::Ident(Ident::new(name.into(), DUMMY_SP))),
                        span: DUMMY_SP,
                        is_type_only: false,
                    })],
                    src: Box::new(Str {
                        span: DUMMY_SP,
                        value: "solid-js/web".into(),
                        raw: None,
                    }),
                    span: DUMMY_SP,
                    type_only: false,
                    asserts: None,
                })),
            );
        });
    }
}

pub fn is_dynamic(
    element: &JSXExprContainer,
    check_member: bool,
    check_tags: bool,
    check_call_expression: bool,
    native: bool,
) -> bool {
    let expr = &element.expr;

    if let JSXExpr::Expr(expr) = expr {
        if let Expr::Fn(_) = **expr {
            return false;
        }
    }

    if match expr {
        JSXExpr::JSXEmptyExpr(_) => false,
        JSXExpr::Expr(expr) => match expr.as_ref() {
            Expr::Call(_) => check_call_expression,
            Expr::Member(_) => check_member,
            Expr::Bin(BinExpr {
                op: BinaryOp::In, ..
            }) => check_member,
            Expr::JSXElement(_) => check_tags,
            Expr::JSXFragment(_) => check_tags,
            _ => false,
        },
    } {
        return true;
    }

    let mut dyn_visitor = DynamicVisitor {
        check_member,
        check_tags,
        check_call_expression,
        native,
        dynamic: false,
    };
    expr.visit_with(&mut dyn_visitor);
    dyn_visitor.dynamic
}

struct DynamicVisitor {
    check_member: bool,
    check_tags: bool,
    check_call_expression: bool,
    native: bool,
    dynamic: bool,
}

impl Visit for DynamicVisitor {
    fn visit_function(&mut self, function: &Function) {
        // https://github.com/ryansolid/dom-expressions/blob/main/packages/babel-plugin-jsx-dom-expressions/src/shared/utils.js#L115-L117
        // if (t.isObjectMethod(p.node) && p.node.computed) {
        //   dynamic = isDynamic(p.get("key"), { checkMember, checkTags, checkCallExpressions, native });
        // }
        // p.skip();
        unimplemented!();
    }
    fn visit_call_expr(&mut self, call_expr: &CallExpr) {
        if self.check_call_expression {
            self.dynamic = true;
        }
    }
    fn visit_member_expr(&mut self, member_expr: &MemberExpr) {
        if self.check_member {
            self.dynamic = true;
        }
    }
    fn visit_bin_expr(&mut self, bin_expr: &BinExpr) {
        if self.check_member && bin_expr.op == BinaryOp::In {
            self.dynamic = true;
        }
        bin_expr.visit_children_with(self);
    }
    fn visit_jsx_element(&mut self, element: &JSXElement) {
        if self.check_tags {
            self.dynamic = true;
        }
    }
    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment) {
        if self.check_tags {
            self.dynamic = true;
        }
    }
}

pub fn get_static_expression(node: &JSXElementChild) -> Option<JSXExpr> {
    //   let value, type;
    //   return (
    //     t.isJSXExpressionContainer(node) &&
    //     t.isJSXElement(path.parent) &&
    //     !isComponent(getTagName(path.parent)) &&
    //     !t.isSequenceExpression(node.expression) &&
    //     (value = path.get("expression").evaluate().value) !== undefined &&
    //     ((type = typeof value) === "string" || type === "number") &&
    //     value
    //   );
    None
}

pub fn filter_children(c: &JSXElementChild) -> bool {
    match c {
        JSXElementChild::JSXText(t) => !t.raw.trim().is_empty(),
        JSXElementChild::JSXExprContainer(JSXExprContainer {
            expr: JSXExpr::JSXEmptyExpr(_),
            ..
        }) => false,
        _ => true,
    }
}

pub fn wrapped_by_text(list: &[ImmutableChildTemplateInstantiation], start_index: usize) -> bool {
    let mut index = start_index;
    let mut wrapped = false;
    while index > 0 {
        index -= 1;
        let node = &list[index];
        if node.text {
            wrapped = true;
            break;
        }

        if node.id.is_some() {
            return false;
        }
    }
    if !wrapped {
        return false;
    }
    index = start_index;
    while index < list.len() {
        let node = &list[index];
        if node.text {
            return true;
        }
        if node.id.is_some() {
            return false;
        }
        index += 1;
    }

    return false;
}
