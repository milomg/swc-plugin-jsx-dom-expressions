use std::path::PathBuf;

use jsx_dom_expressions::process_transform;
use swc_common::{chain, FileName, Mark};
use swc_ecma_transforms_testing::test_fixture;
use swc_ecmascript::{
    parser::{EsConfig, Syntax},
    transforms::resolver,
};
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
        &|t| {
            chain!(
                resolver(Mark::new(), Mark::new(), false),
                process_transform(
                    t.cm.clone(),
                    FileName::Real(PathBuf::from("/some-project/src/some-file.js"))
                )
            )
        },
        &input,
        &output,
    );
}
