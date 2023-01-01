use std::collections::HashMap;

use swc_core::{common::comments::Comments, ecma::ast::*};

pub struct Template {
    pub template: String,
    pub id: Option<Ident>,
    pub tag_name: String,
    pub decl: Vec<Stmt>,
    pub exprs: Vec<Stmt>,
    pub dynamics: Vec<Stmt>,
    pub tag_count: f64,
    pub is_svg: bool,
    pub is_void: bool,
    pub has_custom_element: bool,
}

pub struct TemplateCreation {
    pub template: String,
    pub id: Ident,
    pub tag_count: f64,
}

pub struct TransformVisitor<C>
where
    C: Comments,
{
    pub template: Option<Template>,
    pub templates: Vec<TemplateCreation>,
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
