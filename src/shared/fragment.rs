use super::{
    structs::TemplateInstantiation,
    transform::TransformInfo,
    utils::{filter_children, jsx_text_to_str},
};
pub use crate::shared::structs::TransformVisitor;
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::ast::{ArrayLit, Expr, JSXElementChild, Lit},
};

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
                    match node {
                        JSXElementChild::JSXText(child) => {
                            let value = jsx_text_to_str(&child.value);
                            if value.len() > 0 {
                                memo.push(Expr::Lit(Lit::Str(value.into())));
                            }
                        }
                        _ => {
                            let child = self.transform_node(
                                node,
                                &TransformInfo {
                                    top_level: true,
                                    fragment_child: true,
                                    last_element: true,
                                    ..Default::default()
                                },
                            );
                            memo.push(self.create_template(&mut child.unwrap(), true));
                        }
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
