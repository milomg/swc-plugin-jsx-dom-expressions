use swc_core::{
    common::{comments::Comments, Span, DUMMY_SP},
    ecma::{
        ast::*,
        utils::prepend_stmt,
        visit::{as_folder, FoldWith, Visit, VisitMut, VisitMutWith, VisitWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

struct TemplateInstance {
    template: String,
    id: Ident,
    decl: Vec<Stmt>,
    exprs: Vec<Stmt>,
    dynamics: Vec<Stmt>,
    tag_count: f64,
}

struct TemplateCreation {
    template: String,
    id: Ident,
    tag_count: f64,
}
pub struct TransformVisitor<C>
where
    C: Comments,
{
    templates: Vec<TemplateCreation>,
    current_template: Option<TemplateInstance>,
    comments: C,
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn new(comments: C) -> Self {
        Self {
            current_template: None,
            templates: vec![],
            comments,
        }
    }
}

fn jsx_object_to_str(x: &JSXObject) -> String {
    match x {
        JSXObject::JSXMemberExpr(y) => jsx_object_to_str(&y.obj) + "." + &y.prop.sym,
        JSXObject::Ident(y) => y.sym.to_string(),
    }
}

fn name_to_str(x: &JSXElementName) -> String {
    match x {
        JSXElementName::JSXMemberExpr(y) => jsx_object_to_str(&y.obj) + "." + &y.prop.sym,
        JSXElementName::Ident(ident) => ident.sym.to_string(),
        JSXElementName::JSXNamespacedName(JSXNamespacedName { ns, name }) => {
            ns.sym.to_string() + ":" + &name.sym
        }
    }
}

fn attr_name_to_str(x: &JSXAttrName) -> String {
    match x {
        JSXAttrName::Ident(ident) => ident.sym.to_string(),
        JSXAttrName::JSXNamespacedName(JSXNamespacedName { ns, name }) => {
            ns.sym.to_string() + ":" + &name.sym
        }
    }
}

impl<C> Visit for TransformVisitor<C>
where
    C: Comments,
{
    fn visit_jsx_text(&mut self, el: &JSXText) {
        let tpl = self.current_template.as_mut().unwrap();
        tpl.template.push_str(&el.raw.trim());
    }

    fn visit_jsx_element(&mut self, el: &JSXElement) {
        let level = self.current_template.is_none();

        let tag_name = name_to_str(&el.opening.name);

        let mut buffer = format!("<{}", tag_name);

        for attr in &el.opening.attrs {
            match attr {
                JSXAttrOrSpread::JSXAttr(attr) => {
                    let name = attr_name_to_str(&attr.name);
                    if let Some(val) = &attr.value {
                        match val {
                            JSXAttrValue::JSXExprContainer(expr) => {
                                let expr = expr.expr.clone();
                                // buffer.push_str(&format!(" {}={}", name, expr));
                            }
                            JSXAttrValue::Lit(lit) => {
                                match lit {
                                    Lit::Str(str_lit) => {
                                        let str_lit = str_lit.value.to_string();
                                        buffer.push_str(&format!(" {}=\"{}\"", name, str_lit));
                                    }
                                    _ => {
                                        panic!("unexpected lit");
                                    }
                                }
                                // buffer.push_str(&format!(" {}={}", name, lit));
                            }
                            _ => {
                                panic!("unexpected jsx attr value");
                            }
                        }
                        // let value = jsx_object_to_str(val);
                        // println!("JSXAttr: {}={}", name, value);
                        // buffer.push_str(&format!(" {}={}", name, value));
                    }
                }
                _ => {}
            }
            // buffer += &format!(" {}=\"{}\"", attr.name.sym, attr.value.sym);
        }
        buffer.push('>');

        if level {
            self.current_template = Some(TemplateInstance {
                template: String::new(),
                id: Ident::new(format!("_tmpl${}", self.templates.len()).into(), DUMMY_SP),
                decl: vec![],
                exprs: vec![],
                dynamics: vec![],
                tag_count: 0.0,
            });
        }

        {
            let tpl = self.current_template.as_mut().unwrap();
            tpl.template.push_str(&buffer);
            tpl.tag_count += 1.0;
        }

        el.visit_children_with(self);

        {
            let tpl = self.current_template.as_mut().unwrap();
            tpl.template.push_str(&format!("</{}>", tag_name));
            tpl.tag_count += 1.0;
        }
    }
}

impl<C> VisitMut for TransformVisitor<C>
where
    C: Comments,
{
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Expr::JSXElement(el) = expr {
            el.visit_with(self);
            let val = std::mem::take(&mut self.current_template);

            let mut val = val.unwrap();

            self.templates.push(TemplateCreation {
                template: val.template,
                id: val.id.clone(),
                tag_count: val.tag_count
            });

            let el0 = Ident::new("_el$0".into(), DUMMY_SP);
            val.decl.push(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: false,
                decls: vec![VarDeclarator {
                    name: Pat::Ident(BindingIdent::from(el0.clone())),
                    definite: false,
                    span: DUMMY_SP,
                    init: Some(Box::new(Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                            span: DUMMY_SP,
                            obj: Box::new(Expr::Ident(val.id)),
                            prop: MemberProp::Ident(Ident::new("cloneNode".into(), DUMMY_SP)),
                        }))),
                        args: vec![ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Lit(Lit::Bool(Bool {
                                span: DUMMY_SP,
                                value: true,
                            }))),
                        }],
                        type_args: None,
                    }))),
                }],
            }))));

            expr.visit_mut_children_with(self);

            val.decl.push(Stmt::Return(ReturnStmt {
                span: DUMMY_SP,
                arg: Some(Box::new(Expr::Ident(el0))),
            }));

            *expr = Expr::Call(CallExpr {
                args: vec![],
                span: DUMMY_SP,
                type_args: None,
                callee: Callee::Expr(Box::new(Expr::Arrow(ArrowExpr {
                    return_type: None,
                    type_params: None,
                    span: DUMMY_SP,
                    params: vec![],
                    is_async: false,
                    is_generator: false,
                    body: BlockStmtOrExpr::BlockStmt(BlockStmt {
                        span: DUMMY_SP,
                        stmts: val.decl,
                    }),
                }))),
            });
        } else {
            expr.visit_mut_children_with(self);
        }
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(self);

        let t_ident = Ident::new("_$template".into(), DUMMY_SP);
        let specifier = ImportSpecifier::Named(ImportNamedSpecifier {
            span: DUMMY_SP,
            local: t_ident.clone(),
            imported: Some(ModuleExportName::Ident(Ident::new(
                "template".into(),
                DUMMY_SP,
            ))),
            is_type_only: false,
        });

        let span = Span::dummy_with_cmt();

        self.comments.add_pure_comment(span.lo);

        prepend_stmt(
            &mut module.body,
            ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: false,
                decls: vec![VarDeclarator {
                    name: Pat::Ident(BindingIdent::from(self.templates[0].id.clone())),
                    definite: false,
                    span: DUMMY_SP,
                    init: Some(Box::new(Expr::Call(CallExpr {
                        span,
                        callee: Callee::Expr(Box::new(Expr::Ident(t_ident))),
                        type_args: None,
                        args: vec![ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Tpl(Tpl {
                                span: DUMMY_SP,
                                exprs: vec![],
                                quasis: vec![TplElement {
                                    span: DUMMY_SP,
                                    cooked: None,
                                    tail: true,
                                    raw: self.templates[0].template.clone().into(),
                                }],
                            })),
                        }, ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Lit(Lit::Num(Number {
                                span: DUMMY_SP,
                                value: self.templates[0].tag_count,
                                raw: None,
                            }))),
                        }],
                    }))),
                }],
            })))),
        );

        prepend_stmt(
            &mut module.body,
            ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                span: DUMMY_SP,
                specifiers: vec![specifier],
                src: Box::new(Str {
                    span: DUMMY_SP,
                    raw: None,
                    value: "solid-js/web".into(),
                }),
                type_only: false,
                asserts: None,
            })),
        )
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor::new(&metadata.comments)))
}
