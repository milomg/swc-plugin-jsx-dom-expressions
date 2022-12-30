use swc_core::{
    common::{comments::Comments, Span, DUMMY_SP},
    ecma::{
        ast::*,
        utils::prepend_stmt,
        visit::{as_folder, FoldWith, Visit, VisitMut, VisitMutWith, VisitWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

pub struct TemplateInstantiation {
    template: String,
    id: Ident,
    decl: Vec<Stmt>,
    exprs: Vec<Stmt>,
    dynamics: Vec<Stmt>,
    tag_count: f64,
}

pub struct TemplateCreation {
    template: String,
    id: Ident,
    tag_count: f64,
}

pub struct TransformVisitor<C>
where
    C: Comments,
{
    templates: Vec<TemplateCreation>,
    current_template: Option<TemplateInstantiation>,
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
