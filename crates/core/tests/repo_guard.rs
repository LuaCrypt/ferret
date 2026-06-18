use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn implementation_files_stay_under_300_lines() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    collect(&root, &mut files);
    let offenders = files
        .into_iter()
        .filter_map(|path| {
            let text = fs::read_to_string(&path).ok()?;
            let count = text.lines().count();
            (count > 300).then_some(format!("{}:{count}", path.display()))
        })
        .collect::<Vec<_>>();
    assert!(offenders.is_empty(), "oversized files: {offenders:?}");
}

fn collect(path: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if name == ".git" || name == "target" {
            continue;
        }
        if path.ends_with("tests/vendor") || path.ends_with("tests/output") {
            continue;
        }
        if path.is_dir() {
            collect(&path, files);
        } else if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("rs" | "lua" | "md")
        ) {
            files.push(path);
        }
    }
}
