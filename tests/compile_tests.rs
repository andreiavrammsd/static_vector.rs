#[cfg(test)]
#[test]
fn compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/cases/capacity_zero.rs"); // Fails on newer versions of Rust due to diagnostic message changes
    t.compile_fail("tests/cases/must_use.rs");
}
