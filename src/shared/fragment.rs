use super::{
    structs::TemplateInstantiation,
    transform::TransformInfo,
    utils::{filter_children, jsx_text_to_str},
};
pub use crate::shared::structs::TransformVisitor;
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::ast::{ArrayLit, Expr, JSXElementChild, JSXExpr, Lit},
};
fn do_default<C>(visitor: &mut TransformVisitor<C>, node: &JSXElementChild) -> Expr
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
    visitor.create_template(&mut child.unwrap(), true)
}
impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_fragment_children(
        &mut self,
        children: &[JSXElementChild],
        results: &mut TemplateInstantiation,
    ) {
        let child_nodes: Vec<Expr> =
            children
                .iter()
                .filter(|c| filter_children(c))
                .fold(vec![], |mut memo, node| {
                    let res = match node {
                        JSXElementChild::JSXText(child) => {
                            let value = jsx_text_to_str(&child.value);
                            if value.len() > 0 {
                                Some(Expr::Lit(Lit::Str(value.into())))
                            }
                            else{
                                None
                            }
                        }
                        JSXElementChild::JSXExprContainer(child) => {
                            if let JSXExpr::Expr(new_expr) = &child.expr && let Expr::Lit(Lit::Str(_) | Lit::Num(_) ) = &**new_expr {
                                Some(*new_expr.clone())
                            }
                            else{
                                Some(do_default(self, node))
                            }
                        }
                        _ => Some(do_default(self, node)),
                    };
                    if let Some(expr) = res {
                        memo.push(expr);
                    }
                    memo
                });

        if child_nodes.len() == 1 {
            results.exprs.push(child_nodes[0].clone())
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
