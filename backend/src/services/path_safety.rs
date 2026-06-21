use std::path::Path;

fn canonicalize_best_effort(path: &Path) -> Option<std::path::PathBuf> {
    if let Ok(canon) = path.canonicalize() {
        return Some(canon);
    }
    let parent = path.parent()?;
    let parent_canon = parent.canonicalize().ok()?;
    path.file_name()
        .map(|name| parent_canon.join(name))
}

pub fn allowed_music_roots() -> Vec<String> {
    let mut roots = Vec::new();
    if let Ok(folder) = std::env::var("MUSIC_FOLDER") {
        roots.push(folder);
    }
    if let Ok(upload) = std::env::var("MUSIC_UPLOAD_DIR") {
        roots.push(upload);
    }
    if roots.is_empty() {
        roots.push("uploads".to_string());
    }
    roots
}

pub fn allowed_download_roots() -> Vec<String> {
    allowed_music_roots()
        .into_iter()
        .map(|root| format!("{}/downloads", root.trim_end_matches('/')))
        .collect()
}

pub fn is_path_within_roots(path: &str, roots: &[String]) -> bool {
    let path = Path::new(path);
    let Some(path_canon) = canonicalize_best_effort(path) else {
        return false;
    };
    roots.iter().any(|root| {
        canonicalize_best_effort(Path::new(root))
            .is_some_and(|root_canon| path_canon.starts_with(&root_canon))
    })
}

pub fn is_allowed_music_path(path: &str) -> bool {
    is_path_within_roots(path, &allowed_music_roots())
}

pub fn is_allowed_download_dir(path: &str) -> bool {
    is_path_within_roots(path, &allowed_download_roots())
}
