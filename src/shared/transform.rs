use std::collections::HashSet;

use super::{
    structs::TemplateInstantiation,
    utils::{get_static_expression, is_dynamic},
};
pub use crate::shared::{
    structs::TransformVisitor,
    utils::{get_tag_name, is_component},
};
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::{ast::*, utils::private_ident},
};

#[derive(Default)]
pub struct TransformInfo {
    pub top_level: bool,
    pub skip_id: bool,
    pub component_child: bool,
    pub last_element: bool,
    pub fragment_child: bool,
    pub to_be_closed: Option<HashSet<String>>
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{

    pub fn transform_jsx(&mut self, node: &JSXElementChild) -> Expr {
        let info = match node {
            JSXElementChild::JSXFragment(_) => Default::default(),
            _ => TransformInfo {
                top_level: true,
                last_element: true,
                ..Default::default()
            }
        };
        let result = self.transform_node(node, &info);
        return self.create_template(&mut result.unwrap(), false);
    }

    // todo!
    pub fn transform_node(&mut self, node: &JSXElementChild, info: &TransformInfo) -> Option<TemplateInstantiation> {
        // let config = &self.config;
        match node {
            JSXElementChild::JSXElement(node) => {return Some(self.transform_element(node,info))},
            JSXElementChild::JSXFragment(node) => {
                let mut results = TemplateInstantiation::default();
                self.transform_fragment_children(&node.children, &mut results);
                return Some(results);
            },
            _ => return None
        }
    }

    pub fn transform_jsx_expr(&mut self, node: &mut JSXElement) -> Expr {
        let mut results = self.transform_element(
            node,
            &TransformInfo {
                top_level: true,
                ..Default::default()
            },
        );
        self.create_template(&mut results, false)
    }
    pub fn transform_jsx_element(&mut self, node: &JSXElement) -> TemplateInstantiation {
        self.transform_element(node, &Default::default())
    }
    pub fn transform_jsx_fragment(&mut self, _: &JSXFragment) -> TemplateInstantiation {
        TemplateInstantiation {
            template: "".into(),
            declarations: vec![], //
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
            to_be_closed: HashSet::new()
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
                self.transform_text_child(node.value.to_string(), info)
            }
            con @ JSXElementChild::JSXExprContainer(node) => {
                match &node.expr {
                    JSXExpr::JSXEmptyExpr(_) => None,
                    JSXExpr::Expr(expr) => {
                        if let Some(evaluated) = get_static_expression(con) {
                            if !info.component_child {
                                return self.transform_text_child(evaluated, info);
                            }
                        }

                        if !is_dynamic(expr, true, info.component_child, true, info.component_child)
                        {
                            return Some(TemplateInstantiation {
                                id: None,
                                tag_name: "".into(),
                                template: "".into(),
                                declarations: vec![], //
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
                                to_be_closed: HashSet::new()
                            });
                        }

                        // let expr = expr;
                        Some(TemplateInstantiation {
                            id: None,
                            tag_name: "".into(),
                            declarations: vec![], //
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
                            to_be_closed: HashSet::new()
                        })
                    }
                }
            }
            JSXElementChild::JSXSpreadChild(node) => {
                // TODO: add is_dynamic check for optimization
                let expr = Expr::Arrow(ArrowExpr {
                    span: DUMMY_SP,
                    params: vec![],
                    body: Box::new(BlockStmtOrExpr::Expr(node.expr.clone())),
                    is_async: false,
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                });
                Some(TemplateInstantiation {
                    id: None,
                    tag_name: "".into(),
                    template: "".into(),
                    declarations: vec![], //
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
                    to_be_closed: HashSet::new()
                })
            }
        }
    }

    pub fn transform_text_child(
        &self,
        text: String,
        info: &TransformInfo,
    ) -> Option<TemplateInstantiation> {
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
                declarations: vec![], //
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
                to_be_closed: HashSet::new()
            })
        }
    }
}
