use super::structs::TemplateInstantiation;
use swc_core::{common::DUMMY_SP, ecma::ast::*};

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

pub fn transform_component(expr: &JSXElement) -> TemplateInstantiation {
    let name = &expr.opening.name;
    let tag_id = get_component_identifier(name);

    let has_children = !expr.children.is_empty();

    for attribute in &expr.opening.attrs {
        match attribute {
            JSXAttrOrSpread::SpreadElement(_) => {}
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
            declare: true,
            decls: vec![],
        },
        exprs: vec![],
        dynamics: vec![],
        post_exprs: vec![],
        is_svg: false,
        is_void: false,
        id: None,
        has_custom_element: false,
        dynamic: false,
    }
}
