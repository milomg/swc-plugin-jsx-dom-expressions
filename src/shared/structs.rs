use swc_core::{common::comments::Comments, ecma::ast::*};

pub struct Template {
    pub template: String,
    pub id: Ident,
    pub tag_name: String,
    pub decl: Vec<Stmt>,
    pub exprs: Vec<Stmt>,
    pub dynamics: Vec<Stmt>,
    pub tag_count: f64,
    pub is_svg: bool,
    pub is_void: bool,
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
    // templates: Vec<TemplateCreation>,
    // current_template: Option<TemplateInstantiation>,
    comments: C,
}

impl<C> TransformVisitor<C>
where
    C: Comments,
{
    pub fn new(comments: C) -> Self {
        Self {
            // current_template: None,
            // templates: vec![],
            comments,
        }
    }
}
