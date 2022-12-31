use std::{cell::RefCell, collections::HashMap};

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

// pub struct TemplateCreation {
//     template: String,
//     id: Ident,
//     tag_count: f64,
// }

pub struct TransformVisitor<C>
where
    C: Comments,
{
    pub templates: Vec<Template>,
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
            imports: HashMap::new(),
            comments,
        }
    }
}
