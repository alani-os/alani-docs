#[test]
fn repository_identity_is_stable() {
    assert_eq!(alani_docs::repository_name(), "alani-docs");
    assert!(!alani_docs::module_names().is_empty());
}
