use super::{
    structs::TemplateInstantiation,
    transform::TransformInfo,
    utils::{filter_children, jsx_text_to_str, IntoFirst},
};
pub use crate::shared::structs::TransformVisitor;
use swc_core::{
    common::{DUMMY_SP, comments::Comments},
    ecma::ast::{ArrayLit, Expr, JSXElementChild, JSXExpr, JSXExprContainer, Lit},
};
fn do_default<C>(visitor: &mut TransformVisitor<C>, node: JSXElementChild) -> Expr
where
    C: Comments,
{
    let child = visitor.transform_node(
        node,
        &TransformInfo {
            top_level: true,
            fragment_child: true,
            last_element: true,
            ..Default::default()
        },
    );
    visitor.create_template(child.unwrap(), true)
}
impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_fragment_children(
        &mut self,
        children: Vec<JSXElementChild>,
        results: &mut TemplateInstantiation,
    ) {
        let child_nodes: Vec<Expr> =
            children
                .into_iter()
                .filter(filter_children)
                .fold(vec![], |mut memo, node| {
                    match node {
                        JSXElementChild::JSXText(child) => {
                            let value = jsx_text_to_str(&child.value);
                            if !value.is_empty() {
                                memo.push(Expr::Lit(Lit::Str(value.into())))
                            }
                        }
                        JSXElementChild::JSXExprContainer(JSXExprContainer {
                            expr: JSXExpr::Expr(expr),
                            ..
                        }) if expr.is_lit() || expr.is_ident() => memo.push(*expr),
                        _ => memo.push(do_default(self, node)),
                    };
                    memo
                });

        if child_nodes.len() == 1 {
            results.exprs.push(child_nodes.into_first())
        } else {
            results.exprs.push(Expr::Array(ArrayLit {
                span: DUMMY_SP,
                elems: child_nodes
                    .into_iter()
                    .map(|expr| Some(expr.into()))
                    .collect(),
            }));
        }
    }
}
