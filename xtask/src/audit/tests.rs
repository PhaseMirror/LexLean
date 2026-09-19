use super::{gather, SKIP_DIRS};

#[test]
fn source_audits_exclude_only_repository_relative_generated_paths() {
    let temporary = tempfile::tempdir().expect("temporary repository");
    for ancestor in SKIP_DIRS {
        for basename in ["repository", ancestor] {
            let root = temporary.path().join(ancestor).join(basename);
            std::fs::create_dir_all(root.join("crates/source")).unwrap();
            std::fs::write(root.join("crates/source/main.rs"), "source").unwrap();
            std::fs::write(root.join("crates/source/other.txt"), "not selected").unwrap();
            std::fs::write(root.join("README.md"), "root document").unwrap();
            for generated in SKIP_DIRS {
                let nested = root.join("crates/source").join(generated);
                std::fs::create_dir_all(&nested).unwrap();
                std::fs::write(nested.join("generated.rs"), "generated").unwrap();
            }
            let mut selected = Vec::new();
            gather(&root, &["crates"], &[".rs"], &mut selected);
            assert_eq!(
                selected,
                [root.join("README.md"), root.join("crates/source/main.rs")],
                "ancestor {ancestor}, basename {basename}"
            );
        }
    }
}
