use swc_common::DUMMY_SP;
use swc_plugin::{ast::*, plugin_transform, utils::prepend_stmt, TransformPluginProgramMetadata};

struct TemplateThing {
    template: String,
    decl: Vec<Stmt>,
    exprs: Vec<Stmt>,
    dynamics: Vec<Stmt>,
}

pub struct TransformVisitor {
    templates: Vec<String>,
    current_template: Option<TemplateThing>,
}

impl TransformVisitor {
    pub fn new() -> Self {
        Self {
            current_template: None,
            templates: vec![],
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

impl Visit for TransformVisitor {
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
            self.current_template = Some(TemplateThing {
                template: String::new(),
                decl: vec![],
                exprs: vec![],
                dynamics: vec![],
            });
        }

        {
            let tpl = self.current_template.as_mut().unwrap();

            tpl.template.push_str(&buffer);
        }

        el.visit_children_with(self);

        {
            let tpl = self.current_template.as_mut().unwrap();
            tpl.template.push_str(&format!("</{}>", tag_name));
        }
    }
}

impl VisitMut for TransformVisitor {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Expr::JSXElement(el) = expr {
            expr.visit_with(self);
            let mut val = None;
            std::mem::swap(&mut val, &mut self.current_template);

            let val = val.unwrap();
            self.templates.push(val.template);

            *expr = Expr::Arrow(ArrowExpr {
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
            });
        }

        expr.visit_mut_children_with(self);
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(self);

        let tIdent = Ident::new("_$template".into(), DUMMY_SP);
        let specifier = ImportSpecifier::Named(ImportNamedSpecifier {
            span: DUMMY_SP,
            local: tIdent.clone(),
            imported: Some(ModuleExportName::Ident(Ident::new(
                "template".into(),
                DUMMY_SP,
            ))),
            is_type_only: false,
        });

        prepend_stmt(
            &mut module.body,
            ModuleItem::Stmt(Stmt::Decl(Decl::Var(VarDecl {
                span: DUMMY_SP,
                kind: VarDeclKind::Const,
                declare: false,
                decls: vec![VarDeclarator {
                    name: Pat::Ident(BindingIdent::from(Ident::new("_$tmpl".into(), DUMMY_SP))),
                    definite: false,
                    span: DUMMY_SP,
                    init: Some(Box::new(Expr::Call(CallExpr {
                        span: DUMMY_SP,
                        callee: Callee::Expr(Box::new(Expr::Ident(tIdent))),
                        type_args: None,
                        args: vec![ExprOrSpread{
                            spread: None,
                            expr: Box::new(Expr::Tpl(Tpl {
                            span: DUMMY_SP,
                            exprs: vec![],
                            quasis: vec![TplElement {
                                span: DUMMY_SP,
                                cooked: None,
                                tail: true,
                                raw: self.templates[0].clone().into(),
                            }],
                        }))
                    }],
                    }))),
                }],
            }))),
        );

        prepend_stmt(
            &mut module.body,
            ModuleItem::ModuleDecl(ModuleDecl::Import(ImportDecl {
                span: DUMMY_SP,
                specifiers: vec![specifier],
                src: Str {
                    span: DUMMY_SP,
                    raw: None,
                    value: "solid-js/web".into(),
                },
                type_only: Default::default(),
                asserts: Default::default(),
            })),
        )
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor::new()))
}
