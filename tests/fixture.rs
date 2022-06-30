use std::path::PathBuf;

use jsx_dom_expressions::TransformVisitor;
use swc_common::{chain, Mark};
use swc_ecma_transforms_testing::test_fixture;
use swc_ecmascript::{
    parser::{EsConfig, Syntax},
    transforms::resolver,
};
use swc_plugin::ast::as_folder;
use testing::fixture;

fn syntax() -> Syntax {
    Syntax::Es(EsConfig {
        jsx: true,
        ..Default::default()
    })
}

#[fixture("tests/fixture/**/input.js")]
fn jsx_dom_expressions_fixture(input: PathBuf) {
    let output = input.parent().unwrap().join("output.js");
    test_fixture(
        syntax(),
        &|_| {
            chain!(
                resolver(Mark::new(), Mark::new(), false),
                as_folder(TransformVisitor::new())
            )
        },
        &input,
        &output,
    );
}
