use crate::webview::WindowId;
use fig_os_shim::Context;
use std::{borrow::Cow, sync::Arc};
use wry::http::{Request, Response, StatusCode};

pub async fn clear_index_cache() {}
fn resolve(root: &std::path::Path, uri_path: &str) -> Option<std::path::PathBuf> {
    let root = root.canonicalize().ok()?;
    let path = root.join(uri_path.trim_start_matches('/')).canonicalize().ok()?;
    (path.starts_with(&root) && path.is_file()).then_some(path)
}
pub async fn handle(
    ctx: Arc<Context>,
    request: Request<Vec<u8>>,
    _: WindowId,
) -> anyhow::Result<Response<Cow<'static, [u8]>>> {
    let root = fig_util::directories::resources_path_ctx(&ctx)?.join("specs");
    let path = (request.uri().host() == Some("localhost"))
        .then(|| resolve(&root, request.uri().path()))
        .flatten();
    let Some(path) = path else {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Cow::Borrowed(b"Not Found".as_slice()))?);
    };
    let content_type = match path.extension().and_then(|s| s.to_str()) {
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        _ => "application/javascript",
    };
    Ok(Response::builder()
        .header("Content-Type", content_type)
        .body(Cow::Owned(std::fs::read(path)?))?)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn specs_are_scoped_to_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let specs = dir.path().join("specs");
        std::fs::create_dir(&specs).unwrap();
        std::fs::write(specs.join("git.js"), "export default {};").unwrap();
        std::fs::write(dir.path().join("private.js"), "secret").unwrap();
        assert!(resolve(&specs, "/git.js").is_some());
        assert!(resolve(&specs, "/../private.js").is_none());
        assert!(resolve(&specs, "/missing.js").is_none());
        std::os::unix::fs::symlink(dir.path().join("private.js"), specs.join("link.js")).unwrap();
        assert!(resolve(&specs, "/link.js").is_none());
    }
}
