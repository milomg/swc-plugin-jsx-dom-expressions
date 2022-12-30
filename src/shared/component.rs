use swc_core::{
    common::{comments::Comments, Span, DUMMY_SP},
    ecma::{
        ast::*,
        utils::prepend_stmt,
        visit::{as_folder, FoldWith, Visit, VisitMut, VisitMutWith, VisitWith},
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

pub fn transform_component(expr: &mut JSXElement) {}
