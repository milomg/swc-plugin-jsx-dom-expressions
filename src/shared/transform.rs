use super::{
    structs::{TemplateInstantiation},
    utils::is_dynamic,
};
pub use crate::shared::{
    structs::TransformVisitor,
    utils::{get_tag_name, is_component},
};
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::{ast::*, utils::private_ident},
};

pub struct TransformInfo {
    pub top_level: bool,
    pub skip_id: bool,
    pub component_child: bool,
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
                component_child: false,
            },
        );
        self.create_template(&mut results, false)
    }
    pub fn transform_jsx_element(&mut self, node: &JSXElement) -> TemplateInstantiation {
        let info = TransformInfo {
            top_level: false,
            skip_id: false,
            component_child: false,
        };
        self.transform_element(node, &info)
    }
    pub fn transform_jsx_fragment(&mut self, node: &JSXFragment) -> TemplateInstantiation {
        let info = TransformInfo {
            top_level: false,
            skip_id: false,
            component_child: false,
        };
        TemplateInstantiation {
            template: "".into(),
            id: None,
            tag_name: "".into(),
            decl: VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: false,
                decls: vec![],
            },
            exprs: vec![],
            post_exprs: vec![],
            dynamics: vec![],
            is_svg: false,
            is_void: false,
            has_custom_element: false,
            text: false,
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
            return self.transform_component(node);
        }
        self.transform_element_dom(node, info)
    }

    pub fn transform_jsx_child(
        &mut self,
        node: &JSXElementChild,
        info: &TransformInfo,
    ) -> Option<TemplateInstantiation> {
        match node {
            JSXElementChild::JSXElement(node) => Some(self.transform_element(node, info)),
            JSXElementChild::JSXFragment(node) => {
                // TODO: fixme
                Some(self.transform_jsx_fragment(node))
            }
            JSXElementChild::JSXText(node) => {
                let text = node.value.trim().to_string();
                if text.trim().is_empty() {
                    None
                } else {
                    Some(TemplateInstantiation {
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
                            declare: false,
                            decls: vec![],
                        },
                        exprs: vec![],
                        dynamics: vec![],
                        post_exprs: vec![],
                        has_custom_element: false,
                        is_svg: false,
                        is_void: false,
                        text: true,
                        dynamic: false,
                    })
                }
            }
            JSXElementChild::JSXExprContainer(node) => {
                match &node.expr {
                    JSXExpr::JSXEmptyExpr(_) => None,
                    JSXExpr::Expr(expr) => {
                        if !is_dynamic(
                            node,
                            true,
                            info.component_child,
                            false,
                            info.component_child,
                        ) {
                            return Some(TemplateInstantiation {
                                id: None,
                                tag_name: "".into(),
                                template: "".into(),
                                decl: VarDecl {
                                    span: DUMMY_SP,
                                    kind: VarDeclKind::Const,
                                    declare: false,
                                    decls: vec![],
                                },
                                exprs: vec![*expr.clone()],
                                dynamics: vec![],
                                post_exprs: vec![],
                                has_custom_element: false,
                                is_svg: false,
                                is_void: false,
                                text: false,
                                dynamic: false,
                            });
                        }

                        // let expr = expr;
                        Some(TemplateInstantiation {
                            id: None,
                            tag_name: "".into(),
                            template: "".into(),
                            decl: VarDecl {
                                span: DUMMY_SP,
                                kind: VarDeclKind::Const,
                                declare: false,
                                decls: vec![],
                            },
                            exprs: vec![*expr.clone()],
                            dynamics: vec![],
                            post_exprs: vec![],
                            has_custom_element: false,
                            is_svg: false,
                            is_void: false,
                            text: false,
                            dynamic: true,
                        })
                    }
                }
            }
            JSXElementChild::JSXSpreadChild(node) => {
                // TODO: add is_dynamic check for optimization
                let expr = Expr::Arrow(ArrowExpr {
                    span: DUMMY_SP,
                    params: vec![],
                    body: BlockStmtOrExpr::Expr(node.expr.clone()),
                    is_async: false,
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                });
                Some(TemplateInstantiation {
                    id: None,
                    tag_name: "".into(),
                    template: "".into(),
                    decl: VarDecl {
                        span: DUMMY_SP,
                        kind: VarDeclKind::Const,
                        declare: false,
                        decls: vec![],
                    },
                    exprs: vec![expr],
                    dynamics: vec![],
                    post_exprs: vec![],
                    has_custom_element: false,
                    is_svg: false,
                    is_void: false,
                    text: false,
                    dynamic: true,
                })
            }
        }
    }
}
