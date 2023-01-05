use super::structs::{ChildTemplateInstantiation, TemplateInstantiation};
pub use crate::shared::{
    component::transform_component,
    structs::TransformVisitor,
    utils::{get_tag_name, is_component},
};
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::{ast::*, utils::private_ident, visit::VisitMutWith},
};

pub struct TransformInfo {
    pub top_level: bool,
    pub skip_id: bool,
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_jsx_expr(&mut self, node: &mut JSXElement) -> Expr {
        let mut results = self.transform_element(
            node,
            &TransformInfo {
                top_level: true,
                skip_id: false,
            },
        );
        node.visit_mut_children_with(self);
        self.create_template(node, &mut results, false)
    }
    pub fn transform_jsx_element(&mut self, node: &JSXElement) -> TemplateInstantiation {
        let info = TransformInfo {
            top_level: false,
            skip_id: false,
        };
        self.transform_element(node, &info)
    }
    pub fn transform_jsx_fragment(&mut self, node: &JSXFragment) -> TemplateInstantiation {
        let info = TransformInfo {
            top_level: false,
            skip_id: false,
        };
        TemplateInstantiation {
            template: "".into(),
            id: None,
            tag_name: "".into(),
            decl: VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: true,
                decls: vec![],
            },
            exprs: vec![],
            post_exprs: vec![],
            dynamics: vec![],
            is_svg: false,
            is_void: false,
            has_custom_element: false,
            dynamic: false,
        }
    }

    pub fn transform_element(
        &mut self,
        node: &JSXElement,
        info: &TransformInfo,
    ) -> TemplateInstantiation {
        let tag_name = get_tag_name(node);
        if is_component(&tag_name) {
            return transform_component(node);
        }
        self.transform_element_dom(node, info)
    }

    pub fn transform_jsx_child(
        &mut self,
        node: &JSXElementChild,
        info: &TransformInfo,
    ) -> Option<ChildTemplateInstantiation> {
        match node {
            JSXElementChild::JSXElement(node) => {
                let result = self.transform_element(node, info);
                Some(ChildTemplateInstantiation {
                    id: result.id,
                    tag_name: result.tag_name,
                    template: result.template,
                    decl: result.decl,
                    exprs: result.exprs,
                    dynamics: result.dynamics,
                    post_exprs: result.post_exprs,
                    has_custom_element: result.has_custom_element,
                    text: false,
                })
            }
            JSXElementChild::JSXFragment(node) => {
                // TODO: fixme
                let result = self.transform_jsx_fragment(node);
                Some(ChildTemplateInstantiation {
                    id: result.id,
                    tag_name: result.tag_name,
                    template: result.template,
                    decl: result.decl,
                    exprs: result.exprs,
                    dynamics: result.dynamics,
                    post_exprs: result.post_exprs,
                    has_custom_element: result.has_custom_element,
                    text: false,
                })
            }
            JSXElementChild::JSXText(node) => {
                let text = node.value.trim().to_string();
                if text.trim().is_empty() {
                    None
                } else {
                    Some(ChildTemplateInstantiation {
                        id: if info.skip_id {
                            None
                        } else {
                            Some(private_ident!("el$"))
                        },
                        tag_name: "".into(),
                        template: text,
                        decl: VarDecl {
                            span: DUMMY_SP,
                            kind: VarDeclKind::Const,
                            declare: true,
                            decls: vec![],
                        },
                        exprs: vec![],
                        dynamics: vec![],
                        post_exprs: vec![],
                        has_custom_element: false,
                        text: true,
                    })
                }
            }
            JSXElementChild::JSXSpreadChild(node) => {
                let expr = Expr::Arrow(ArrowExpr {
                    span: DUMMY_SP,
                    params: vec![],
                    body: BlockStmtOrExpr::Expr(node.expr.clone()),
                    is_async: false,
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                });
                Some(ChildTemplateInstantiation {
                    id: None,
                    tag_name: "".into(),
                    template: "".into(),
                    decl: VarDecl {
                        span: DUMMY_SP,
                        kind: VarDeclKind::Const,
                        declare: true,
                        decls: vec![],
                    },
                    exprs: vec![expr],
                    dynamics: vec![],
                    post_exprs: vec![],
                    has_custom_element: false,
                    text: false,
                })
            }
            _ => {
                panic!("not implemented");
            }
        }
    }
}
