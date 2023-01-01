use std::collections::HashMap;
use swc_core::{common::comments::Comments, ecma::ast::*};

pub struct TemplateConstruction {
    pub template: String,
    pub id: Ident,
    pub tag_count: f64,
}

pub struct TemplateInstantiation {
    pub template: String,
    pub id: Option<Ident>,
    pub tag_name: String,
    pub decl: VarDecl,
    pub exprs: Vec<Expr>,
    pub dynamics: Vec<Expr>,
    pub is_svg: bool,
    pub is_void: bool,
    pub has_custom_element: bool,
    pub dynamic: bool
}

pub struct TransformVisitor<C>
where
    C: Comments,
{
    pub template: Option<TemplateInstantiation>,
    pub templates: Vec<TemplateConstruction>,
    pub imports: HashMap<String, Ident>,
    comments: C,
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn new(comments: C) -> Self {
        Self {
            templates: vec![],
            template: None,
            imports: HashMap::new(),
            comments,
        }
    }
}
