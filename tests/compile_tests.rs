#[cfg(test)]
#[test]
fn must_use() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/cases/must_use.rs");
}
