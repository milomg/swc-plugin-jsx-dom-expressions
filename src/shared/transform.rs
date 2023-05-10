use std::collections::HashSet;

use super::{
    structs::TemplateInstantiation,
    utils::{get_static_expression},
};
use crate::shared::utils::{trim_whitespace, escape_backticks, escape_html};
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
    pub to_be_closed: Option<HashSet<String>>,
    pub do_not_escape: bool
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

    pub fn transform_node(&mut self, node: &JSXElementChild, info: &TransformInfo) -> Option<TemplateInstantiation> {
        if let JSXElementChild::JSXElement(node) = node {
            return Some(self.transform_element(node,info))
        } else if let JSXElementChild::JSXFragment(node) = node {
            let mut results = TemplateInstantiation::default();
            self.transform_fragment_children(&node.children, &mut results);
            return Some(results);
        } else if let JSXElementChild::JSXText(node) = node {
            let text = trim_whitespace(&node.raw);
            if text.is_empty() {
                return None;
            }
            let mut results = TemplateInstantiation {
                template: escape_backticks(&text),
                text: true,
                ..TemplateInstantiation::default()
            };
            if !info.skip_id {
                results.id = Some(self.generate_uid_identifier("el$"));
            }
            return Some(results);
        } else if let Some(static_value) = get_static_expression(node) {
            let text = if info.do_not_escape {
                static_value
            } else {
                escape_html(&static_value, false)
            };
            if text.is_empty() {
                return None;
            }
            let mut results = TemplateInstantiation {
                template: escape_backticks(&text),
                text: true,
                ..TemplateInstantiation::default()
            };
            if !info.skip_id {
                results.id = Some(self.generate_uid_identifier("el$"));
            }
            return Some(results);
        } else if let JSXElementChild::JSXExprContainer(JSXExprContainer { expr, .. }) = node {
            match expr {
                JSXExpr::JSXEmptyExpr(_) => {
                    return None;
                }
                JSXExpr::Expr(exp) => {
                    if !self.is_dynamic(&exp, None, true, info.component_child, true, !info.component_child) {
                        return Some(TemplateInstantiation {
                            exprs: vec![*exp.clone()],
                            ..Default::default()
                        });
                    }
                    let mut expr = vec![];
                    if self.config.wrap_conditionals &&( matches!(**exp, Expr::Bin(_)) || matches!(**exp, Expr::Cond(_)) ) {
                        let result = self.transform_condition(*exp.clone(), info.component_child, false);
                        match result {
                            (Some(stmt0), ex1) => {
                                expr = vec![Expr::Call(CallExpr { 
                                    span: DUMMY_SP,
                                     callee: Callee::Expr(Box::new(Expr::Arrow(ArrowExpr { 
                                        span: DUMMY_SP, 
                                        params: vec![], 
                                        body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt { span: DUMMY_SP, stmts: vec![
                                            stmt0, 
                                            Stmt::Return(ReturnStmt { 
                                                span: DUMMY_SP, 
                                                arg: Some(Box::new(ex1)) })] 
                                            })), 
                                        is_async: false, 
                                        is_generator: false, 
                                        type_params: None, 
                                        return_type: None }))), 
                                     args: vec![], 
                                     type_args: None
                                })];
                            },
                            (None, ex0) => {
                                expr = vec![ex0]
                            }
                        }
                    } else {
                        let mut flag = false;
                        if !info.component_child && info.fragment_child {
                            if let Expr::Call(CallExpr { callee: Callee::Expr(ref ex) , ref args,.. }) = **exp {
                                if !matches!(**ex, Expr::Member(_)) && args.is_empty() {
                                    flag = true;
                                    expr = vec![*ex.clone()];
                                }
                            }
                        } 
                        if !flag {
                            expr = vec![Expr::Arrow(ArrowExpr { 
                                span: DUMMY_SP, 
                                params: vec![], 
                                body: Box::new(BlockStmtOrExpr::Expr(exp.clone())), 
                                is_async: false, 
                                is_generator: false, 
                                type_params: None, 
                                return_type: None
                            })];
                        }
                    }
                    return Some(TemplateInstantiation {
                        exprs: expr,
                        dynamic: true,
                        ..Default::default()
                    });

                },
            } 
        } else if let JSXElementChild::JSXSpreadChild(JSXSpreadChild { expr, .. }) = node {
            if !self.is_dynamic(expr, None, true, false, true, !info.component_child) {
                return Some(TemplateInstantiation {
                    exprs: vec![*expr.clone()],
                    ..Default::default()
                });
            }
            return Some(TemplateInstantiation {
                exprs: vec![Expr::Arrow(ArrowExpr { 
                    span: DUMMY_SP, 
                    params: vec![], 
                    body: Box::new(BlockStmtOrExpr::Expr(expr.clone())), 
                    is_async: false, 
                    is_generator: false, 
                    type_params: None, 
                    return_type: None })],
                dynamic: true,
                ..Default::default()
            });
        }
        None
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
        Default::default()
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

                        if !self.is_dynamic(expr, None, true, info.component_child, true, info.component_child)
                        {
                            return Some(TemplateInstantiation {
                                exprs: vec![*expr.clone()],
                                ..Default::default()
                            });
                        }

                        Some(TemplateInstantiation {
                            exprs: vec![*expr.clone()],
                            dynamic: true,
                            ..Default::default()
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
                    exprs: vec![expr],
                    dynamic: true,
                    ..Default::default()
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
                template: text,
                text: true,
                ..Default::default()
            })
        }
    }
}
