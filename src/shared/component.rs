use crate::TransformVisitor;

use super::{structs::TemplateInstantiation, transform::TransformInfo};
use swc_core::{
    common::{comments::Comments, DUMMY_SP},
    ecma::ast::*,
};

enum TagId {
    Ident(Ident),
    StringLiteral(Str),
    MemberExpr(Box<MemberExpr>),
}

fn get_component_identifier(node: &JSXElementName) -> TagId {
    match node {
        JSXElementName::Ident(ident) => TagId::Ident(ident.clone()),
        JSXElementName::JSXMemberExpr(member) => {
            let obj = get_component_identifier(&match &member.obj {
                JSXObject::JSXMemberExpr(member) => JSXElementName::JSXMemberExpr(*member.clone()),
                JSXObject::Ident(ident) => JSXElementName::Ident(ident.clone()),
            });
            TagId::MemberExpr(Box::new(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(match obj {
                    TagId::Ident(ident) => Expr::Ident(ident),
                    TagId::StringLiteral(str) => Expr::Lit(Lit::Str(str)),
                    TagId::MemberExpr(member) => Expr::Member(*member),
                }),
                prop: MemberProp::Ident(member.prop.clone()),
            }))
        }
        JSXElementName::JSXNamespacedName(name) => {
            let name = format!("{}:{}", name.ns.sym, name.name.sym);
            let name = Str {
                span: DUMMY_SP,
                value: Into::into(name),
                raw: None,
            };
            TagId::StringLiteral(name)
        }
    }
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn transform_component(&mut self, expr: &JSXElement) -> TemplateInstantiation {
        let name = &expr.opening.name;
        let tag_id = get_component_identifier(name);

        let has_children = !expr.children.is_empty();

        for attribute in &expr.opening.attrs {
            match attribute {
                JSXAttrOrSpread::SpreadElement(node) => {}
                _ => {}
            }
        }

        // Placeholder to satisfy type checking
        TemplateInstantiation {
            template: "".into(),
            tag_name: "".into(),
            decl: VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: false,
                decls: vec![],
            },
            exprs: vec![],
            dynamics: vec![],
            post_exprs: vec![],
            is_svg: false,
            is_void: false,
            id: None,
            has_custom_element: false,
            text: false,
            dynamic: false,
        }
    }

    fn transform_component_children(
        &mut self,
        children: Vec<JSXElementChild>,
    ) -> Option<(Expr, bool)> {
        let filtered_children = children
            .into_iter()
            .filter(|child| match child {
                JSXElementChild::JSXElement(_)
                | JSXElementChild::JSXFragment(_)
                | JSXElementChild::JSXSpreadChild(_) => true,
                JSXElementChild::JSXText(child) => child.raw.chars().any(|c| !c.is_whitespace()),
                JSXElementChild::JSXExprContainer(_) => false,
            })
            .collect::<Vec<_>>();
        if filtered_children.len() == 0 {
            return None;
        }

        let mut dynamic = false;
        let is_filtered_children_plural = filtered_children.len() > 1;

        let transformed_children: Vec<Expr> = filtered_children
            .iter()
            .filter_map(|child| {
                match child {
                    JSXElementChild::JSXText(child) => {
                        let decoded = html_escape::decode_html_entities(child.raw.trim());
                        if decoded.len() > 0 {
                            return Some(Lit::Str(decoded.to_string().into()).into());
                        }
                    }
                    node => {
                        let child = self.transform_jsx_child(
                            &node,
                            &TransformInfo {
                                top_level: true,
                                component_child: true,
                                skip_id: false,
                            },
                        );
                        if let Some(mut child) = child {
                            dynamic = dynamic || child.dynamic;

                            if is_filtered_children_plural && child.dynamic {
                                if let Some(Expr::Arrow(ArrowExpr {
                                    body: BlockStmtOrExpr::Expr(expr),
                                    ..
                                })) = child.exprs.first()
                                {
                                    child.exprs.insert(0, *expr.clone());
                                }
                            }

                            return Some(
                                self.create_template(&mut child, is_filtered_children_plural),
                            );
                        }
                    }
                };

                None
            })
            .collect();

        if transformed_children.len() == 1 {
            let first_children = transformed_children.into_iter().next().unwrap();

            match filtered_children.first() {
                Some(JSXElementChild::JSXExprContainer(_))
                | Some(JSXElementChild::JSXSpreadChild(_))
                | Some(JSXElementChild::JSXText(_))
                | None => Some((first_children, dynamic)),
                _ => {
                    let expr = match &first_children {
                        Expr::Call(CallExpr {
                            callee: Callee::Expr(callee_expr),
                            args,
                            ..
                        }) if args.is_empty() => match *callee_expr.clone() {
                            Expr::Ident(_) => None,
                            expr => Some(expr),
                        },
                        _ => None,
                    }
                    .unwrap_or(
                        ArrowExpr {
                            span: DUMMY_SP,
                            params: vec![],
                            body: BlockStmtOrExpr::Expr(Box::new(first_children)),
                            is_async: false,
                            is_generator: false,
                            type_params: None,
                            return_type: None,
                        }
                        .into(),
                    );

                    Some((expr, true))
                }
            }
        } else {
            Some((
                ArrowExpr {
                    span: DUMMY_SP,
                    params: vec![],
                    body: BlockStmtOrExpr::Expr(
                        ArrayLit {
                            span: DUMMY_SP,
                            elems: transformed_children
                                .into_iter()
                                .map(|expr| Some(expr.into()))
                                .collect(),
                        }
                        .into(),
                    ),
                    is_async: false,
                    is_generator: false,
                    type_params: None,
                    return_type: None,
                }
                .into(),
                true,
            ))
        }
    }
}
