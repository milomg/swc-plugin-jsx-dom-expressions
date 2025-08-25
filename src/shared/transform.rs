use super::structs::TemplateInstantiation;
use crate::shared::utils::{escape_backticks, escape_html, trim_whitespace};
pub use crate::shared::{
    structs::TransformVisitor,
    utils::{get_tag_name, is_component},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::HashSet;
use swc_core::{
    common::{DUMMY_SP, comments::Comments},
    ecma::{
        ast::*,
        utils::{ExprFactory, private_ident},
        visit::{Visit, VisitMut, VisitMutWith, VisitWith},
    },
};

#[derive(Default)]
pub struct VarBindingCollector {
    pub const_var_bindings: FxHashMap<Id, Option<Expr>>,
    pub function_bindings: FxHashSet<Id>,
}

impl VarBindingCollector {
    pub fn new() -> Self {
        Self::default()
    }

    fn collect_pat(&mut self, pat: &Pat, init: Option<Expr>) {
        match pat {
            Pat::Ident(id) => {
                self.const_var_bindings.insert(id.to_id(), init);
            }
            Pat::Array(a) => {
                for p in a.elems.iter().flatten() {
                    self.collect_pat(p, None);
                }
            }
            Pat::Rest(rest) => self.collect_pat(&rest.arg, None),
            _ => {}
        };
    }
}

impl Visit for VarBindingCollector {
    fn visit_import_decl(&mut self, import_dect: &ImportDecl) {
        for spec in &import_dect.specifiers {
            match spec {
                ImportSpecifier::Named(s) => self.const_var_bindings.insert(s.local.to_id(), None),
                ImportSpecifier::Default(s) => {
                    self.const_var_bindings.insert(s.local.to_id(), None)
                }
                ImportSpecifier::Namespace(s) => {
                    self.const_var_bindings.insert(s.local.to_id(), None)
                }
            };
        }
    }

    fn visit_var_decl(&mut self, n: &VarDecl) {
        if n.kind == VarDeclKind::Const {
            for decl in &n.decls {
                self.collect_pat(&decl.name, decl.init.clone().map(|v| *v));
            }
        }
        n.visit_children_with(self);
    }

    fn visit_fn_decl(&mut self, f: &FnDecl) {
        self.function_bindings.insert(f.ident.to_id());
    }
}

pub struct ThisBlockVisitor {
    this_id: Option<Ident>,
    has_jsx: bool,
}

impl ThisBlockVisitor {
    pub fn new() -> Self {
        Self {
            this_id: None,
            has_jsx: false,
        }
    }
}

impl VisitMut for ThisBlockVisitor {
    fn visit_mut_block_stmt(&mut self, block: &mut BlockStmt) {
        let mut id = self.this_id.clone();
        self.has_jsx = false;
        block.visit_mut_children_with(self);
        std::mem::swap(&mut self.this_id, &mut id);
        if let Some(id) = id {
            block.stmts.insert(
                0,
                Stmt::Decl(Decl::Var(Box::new(VarDecl {
                    span: DUMMY_SP,
                    kind: VarDeclKind::Const,
                    declare: false,
                    decls: vec![VarDeclarator {
                        span: DUMMY_SP,
                        name: Pat::Ident(id.into()),
                        init: Some(Box::new(Expr::This(ThisExpr { span: DUMMY_SP }))),
                        definite: false,
                    }],
                    ..Default::default()
                }))),
            )
        }
    }
    fn visit_mut_jsx_element(&mut self, el: &mut JSXElement) {
        self.has_jsx = true;
        el.visit_mut_children_with(self);
    }
    fn visit_mut_jsx_fragment(&mut self, el: &mut JSXFragment) {
        self.has_jsx = true;
        el.visit_mut_children_with(self);
    }
    fn visit_mut_expr(&mut self, n: &mut Expr) {
        if let Expr::This(_) = n {
            if !self.has_jsx {
                return;
            }
            if self.this_id.is_none() {
                self.this_id = Some(private_ident!("_self$"));
            }
            *n = Expr::Ident(self.this_id.clone().unwrap());
        } else {
            n.visit_mut_children_with(self);
        }
    }
}

#[derive(Default)]
pub struct TransformInfo {
    pub top_level: bool,
    pub skip_id: bool,
    pub component_child: bool,
    pub last_element: bool,
    pub fragment_child: bool,
    pub to_be_closed: Option<HashSet<String>>,
    pub do_not_escape: bool,
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
            },
        };
        let result = self.transform_node(node, &info);
        self.create_template(&mut result.unwrap(), false)
    }

    pub fn transform_node(
        &mut self,
        node: &JSXElementChild,
        info: &TransformInfo,
    ) -> Option<TemplateInstantiation> {
        if let JSXElementChild::JSXElement(node) = node {
            return Some(self.transform_element(node, info));
        } else if let JSXElementChild::JSXFragment(node) = node {
            let mut results = TemplateInstantiation::default();
            self.transform_fragment_children(&node.children, &mut results);
            return Some(results);
        } else if let JSXElementChild::JSXText(node) = node {
            let text =
                trim_whitespace(&html_escape::encode_text(&node.raw).replace('\u{a0}', "&nbsp;"));
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
        } else if let Some(static_value) = self.get_static_expression(node) {
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
        } else if let JSXElementChild::JSXExprContainer(JSXExprContainer { expr, span }) = node {
            match expr {
                JSXExpr::JSXEmptyExpr(_) => {
                    return None;
                }
                JSXExpr::Expr(exp) => {
                    if !self.is_dynamic(
                        exp,
                        Some(*span),
                        true,
                        info.component_child,
                        true,
                        !info.component_child,
                    ) {
                        return Some(TemplateInstantiation {
                            exprs: vec![*exp.clone()],
                            ..Default::default()
                        });
                    }
                    let mut expr = vec![];
                    if self.config.wrap_conditionals
                        && self.config.generate != "ssr"
                        && (matches!(**exp, Expr::Bin(_)) || matches!(**exp, Expr::Cond(_)))
                    {
                        let result =
                            self.transform_condition(*exp.clone(), info.component_child, false);
                        match result {
                            (Some(stmt0), ex1) => {
                                expr = vec![Expr::Call(CallExpr {
                                    span: DUMMY_SP,
                                    callee: Callee::Expr(Box::new(Expr::Arrow(ArrowExpr {
                                        span: DUMMY_SP,
                                        params: vec![],
                                        body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                                            span: DUMMY_SP,
                                            stmts: vec![
                                                stmt0,
                                                Stmt::Return(ReturnStmt {
                                                    span: DUMMY_SP,
                                                    arg: Some(Box::new(ex1)),
                                                }),
                                            ],
                                            ..Default::default()
                                        })),
                                        ..Default::default()
                                    }))),
                                    ..Default::default()
                                })];
                            }
                            (None, ex0) => expr = vec![ex0],
                        }
                    } else {
                        let mut flag = false;
                        if !info.component_child
                            && (self.config.generate != "ssr" || info.fragment_child)
                            && let Expr::Call(CallExpr {
                                callee: Callee::Expr(ref ex),
                                ref args,
                                ..
                            }) = **exp
                            && !matches!(**ex, Expr::Member(_))
                            && args.is_empty()
                        {
                            flag = true;
                            expr = vec![*ex.clone()];
                        }
                        if !flag {
                            expr = vec![exp.clone().into_lazy_arrow(vec![]).into()];
                        }
                    }
                    return Some(TemplateInstantiation {
                        exprs: expr,
                        dynamic: true,
                        ..Default::default()
                    });
                }
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
                    ..Default::default()
                })],
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
}
