use anyhow::Result;
use axum::{
    Router,
    body::Body,
    extract::{Json, Query},
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;

use omnideps::{
    analyzer::{analyze_code_snippet, analyze_project},
    config::AnalyzerConfig,
    language::SupportedLanguage,
};

#[derive(RustEmbed)]
#[folder = "visualizer-svelte/build/"]
struct Assets;

#[derive(RustEmbed)]
#[folder = "docs/"]
struct DocsAssets;

#[derive(RustEmbed)]
#[folder = "tests/"]
struct TestsAssets;

#[derive(Deserialize)]
struct AnalyzeRequest {
    path: Option<String>,
    code: Option<String>,
    extension: Option<String>,
}

#[derive(Deserialize)]
struct DocsQuery {
    path: String,
}

#[derive(Deserialize)]
struct RunBenchmarkRequest {
    benchmark: String,
}

#[derive(Serialize, Clone)]
struct DocsNode {
    #[serde(rename = "type")]
    node_type: String,
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<DocsNode>>,
}

pub fn execute(port: u16) -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let app = Router::new()
                .route("/api/analyze", post(api_analyze))
                .route("/api/benchmarks", get(api_benchmarks))
                .route("/api/benchmark-suites", get(api_benchmark_suites))
                .route("/api/run-benchmark", post(api_run_benchmark))
                .route("/api/docs/tree", get(api_docs_tree))
                .route("/api/docs/content", get(api_docs_content))
                .fallback(static_handler);

            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            let url = format!("http://{}", addr);
            println!("Starting Web Visualizer at {}", url);

            // Tenta di aprire il browser automaticamente
            if let Err(e) = open::that(&url) {
                log::warn!("Could not open browser automatically: {}", e);
            }

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });

    Ok(())
}

// -----------------------------------------------------------------------------
// ANALYZE API
// -----------------------------------------------------------------------------
async fn api_analyze(Json(payload): Json<AnalyzeRequest>) -> impl IntoResponse {
    let config = AnalyzerConfig::default_strategies();

    let graph = if let Some(code) = &payload.code {
        if let Some(ext) = &payload.extension {
            let lang = match SupportedLanguage::from_extension(ext) {
                Some(l) => l,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        axum::Json(serde_json::json!({ "error": "Unsupported extension" })),
                    )
                        .into_response();
                }
            };
            match analyze_code_snippet(lang, code, "temp", &config) {
                Ok((_, graph)) => graph,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(serde_json::json!({ "error": format!("Parsing error: {}", e) })),
                    )
                        .into_response();
                }
            }
        } else {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Missing extension" })),
            )
                .into_response();
        }
    } else if let Some(path_str) = &payload.path {
        let is_embedded_test = path_str.starts_with("omnideps-builtin/");
        let path = Path::new(path_str);

        let temp_dir; // declare outside to keep it alive
        let actual_path = if is_embedded_test {
            let target_prefix = path_str.strip_prefix("omnideps-builtin/").unwrap();
            temp_dir = tempfile::tempdir().unwrap();
            let temp_path = temp_dir.path();

            let mut extracted_any = false;
            for file in TestsAssets::iter() {
                let file_path_str = file.as_ref();
                if file_path_str.starts_with(target_prefix) {
                    extracted_any = true;
                    let dest_path = temp_path.join(file_path_str);
                    if let Some(parent) = dest_path.parent() {
                        std::fs::create_dir_all(parent).unwrap();
                    }
                    let content = TestsAssets::get(file_path_str).unwrap();
                    std::fs::write(&dest_path, content.data).unwrap();
                }
            }
            if !extracted_any {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({ "error": "Path not found" })),
                )
                    .into_response();
            }
            temp_path.join(target_prefix)
        } else {
            if !path.exists() {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({ "error": "Path not found" })),
                )
                    .into_response();
            }
            path.to_path_buf()
        };

        match analyze_project(&actual_path, &config) {
            Ok((_, graph)) => graph,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response();
            }
        }
    } else {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Provide 'path' or 'code'+'extension'" })),
        )
            .into_response();
    };

    (
        StatusCode::OK,
        axum::Json(serde_json::json!({
            "rawOutput": graph,
            "elements": omnideps::export::cytoscape::convert_to_cyto_elements(&[graph])
        })),
    )
        .into_response()
}

// -----------------------------------------------------------------------------
// BENCHMARKS API
// -----------------------------------------------------------------------------
async fn api_benchmarks() -> impl IntoResponse {
    let mut files_or_folders = std::collections::HashSet::new();

    for file in TestsAssets::iter() {
        let path_str = file.as_ref();

        if path_str.starts_with("generics/") {
            let ext = Path::new(path_str)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if matches!(
                ext,
                "java" | "rs" | "py" | "c" | "cpp" | "cxx" | "cc" | "hxx" | "hpp" | "h"
            ) {
                files_or_folders.insert(format!("omnideps-builtin/{}", path_str));
            }
        } else if path_str.starts_with("benchmarks/benchmark-") {
            let parts: Vec<&str> = path_str.split('/').collect();
            if parts.len() >= 2 {
                files_or_folders.insert(format!("omnideps-builtin/benchmarks/{}", parts[1]));
            }
        }
    }

    let mut result: Vec<String> = files_or_folders.into_iter().collect();
    result.sort();
    (StatusCode::OK, axum::Json(result)).into_response()
}

async fn api_benchmark_suites() -> impl IntoResponse {
    let mut suites = std::collections::HashSet::new();

    for file in TestsAssets::iter() {
        let path_str = file.as_ref();
        if path_str.starts_with("benchmarks/benchmark-") {
            let parts: Vec<&str> = path_str.split('/').collect();
            if parts.len() >= 2 {
                suites.insert(format!("omnideps-builtin/benchmarks/{}", parts[1]));
            }
        }
    }

    let mut suites_vec: Vec<String> = suites.into_iter().collect();
    suites_vec.sort();
    (StatusCode::OK, axum::Json(suites_vec)).into_response()
}

async fn api_run_benchmark(Json(payload): Json<RunBenchmarkRequest>) -> impl IntoResponse {
    let bench_path_str = payload.benchmark;
    if !bench_path_str.starts_with("omnideps-builtin/benchmarks/benchmark-")
        || bench_path_str.contains("..")
    {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Invalid benchmark path" })),
        )
            .into_response();
    }

    let target_prefix = bench_path_str.strip_prefix("omnideps-builtin/").unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    for file in TestsAssets::iter() {
        let file_path_str = file.as_ref();
        if file_path_str.starts_with(target_prefix) {
            let dest_path = temp_path.join(file_path_str);
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let content = TestsAssets::get(file_path_str).unwrap();
            std::fs::write(&dest_path, content.data).unwrap();
        }
    }

    let target_path = temp_path.join(target_prefix);
    let config = AnalyzerConfig::default_strategies();

    if let Err(e) = crate::commands::benchmark::execute_run_with_config(&target_path, None, &config)
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(
                serde_json::json!({ "error": "Failed to run benchmark", "details": e.to_string() }),
            ),
        )
            .into_response();
    }

    let report_path = target_path.join("report.md");
    match std::fs::read_to_string(&report_path) {
        Ok(markdown_content) => (StatusCode::OK, axum::Json(serde_json::json!({ "markdown": markdown_content }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": "Failed to read report.md", "details": e.to_string() }))).into_response(),
    }
}

// -----------------------------------------------------------------------------
// DOCS API
// -----------------------------------------------------------------------------
fn build_docs_tree_from_embedded() -> Vec<DocsNode> {
    let mut tree_map: std::collections::HashMap<String, DocsNode> =
        std::collections::HashMap::new();

    for file in DocsAssets::iter() {
        let path_str = file.as_ref();
        if path_str.starts_with('.') || (!path_str.ends_with(".md") && !path_str.ends_with(".html"))
        {
            continue;
        }

        let parts: Vec<&str> = path_str.split('/').collect();
        let mut current_path = String::new();

        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                current_path.push('/');
            }
            current_path.push_str(part);

            if !tree_map.contains_key(&current_path) {
                let is_file = i == parts.len() - 1;
                tree_map.insert(
                    current_path.clone(),
                    DocsNode {
                        node_type: if is_file {
                            "file".to_string()
                        } else {
                            "directory".to_string()
                        },
                        name: part.to_string(),
                        path: current_path.clone(),
                        children: if is_file { None } else { Some(vec![]) },
                    },
                );
            }
        }
    }

    // Reconstruct tree hierarchy
    let mut root_nodes = vec![];
    let mut cloned_map = tree_map.clone();

    let mut paths: Vec<String> = cloned_map.keys().cloned().collect();
    paths.sort_by_key(|p| p.split('/').count());
    paths.reverse(); // Process deepest nodes first

    for path in paths {
        if let Some(parent_idx) = path.rfind('/') {
            let parent_path = &path[0..parent_idx];
            if let Some(node) = cloned_map.remove(&path)
                && let Some(parent_node) = cloned_map.get_mut(parent_path)
                && let Some(children) = &mut parent_node.children
            {
                children.push(node);
            }
        } else {
            if let Some(node) = cloned_map.remove(&path) {
                root_nodes.push(node);
            }
        }
    }

    // Sort nodes recursively
    fn sort_tree(nodes: &mut Vec<DocsNode>) {
        nodes.sort_by(|a, b| {
            if a.node_type == b.node_type {
                a.name.cmp(&b.name)
            } else if a.node_type == "directory" {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });
        for node in nodes {
            if let Some(children) = &mut node.children {
                sort_tree(children);
            }
        }
    }

    sort_tree(&mut root_nodes);
    root_nodes
}

async fn api_docs_tree() -> impl IntoResponse {
    let tree = build_docs_tree_from_embedded();
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({ "tree": tree })),
    )
        .into_response()
}

async fn api_docs_content(Query(query): Query<DocsQuery>) -> impl IntoResponse {
    let file_path_str = query.path;

    if file_path_str.contains("..") {
        return (
            StatusCode::FORBIDDEN,
            axum::Json(serde_json::json!({ "error": "Invalid path" })),
        )
            .into_response();
    }

    match DocsAssets::get(&file_path_str) {
        Some(content) => {
            let is_html = file_path_str.ends_with(".html");
            let text = String::from_utf8_lossy(&content.data).into_owned();
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "content": text, "isHtml": is_html })),
            )
                .into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "File not found" })),
        )
            .into_response(),
    }
}

// -----------------------------------------------------------------------------
// STATIC HANDLER (SVELTEKIT)
// -----------------------------------------------------------------------------
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Assets::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data))
                .unwrap()
        }
        None => {
            if let Some(content) = Assets::get("index.html") {
                Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(content.data))
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("404 Not Found"))
                    .unwrap()
            }
        }
    }
}
