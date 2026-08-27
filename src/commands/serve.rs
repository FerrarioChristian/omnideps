use anyhow::Result;
use axum::{
    body::Body,
    extract::{Json, Query},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use omnideps::{
    analyzer::{analyze_project, parse_source},
    config::AnalyzerConfig,
    language::SupportedLanguage,
    resolver::primitives::PrimitiveRegistry,
};

#[derive(RustEmbed)]
#[folder = "visualizer-svelte/build/"]
struct Assets;

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

#[derive(Serialize)]
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
            println!("Avvio Web Visualizer all'indirizzo http://{}", addr);

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

    let (modules, primitives) = if let Some(code) = &payload.code {
        if let Some(ext) = &payload.extension {
            let lang = match SupportedLanguage::from_extension(ext) {
                Some(l) => l,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        axum::Json(serde_json::json!({ "error": "Estensione non supportata" })),
                    )
                        .into_response();
                }
            };
            match parse_source(lang, code, Path::new("temp"), &config) {
                Ok(res) => res,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(serde_json::json!({ "error": format!("Errore di parsing: {}", e) })),
                    )
                        .into_response();
                }
            }
        } else {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Estensione mancante" })),
            )
                .into_response();
        }
    } else if let Some(path_str) = &payload.path {
        let path = Path::new(path_str);
        if !path.exists() {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Percorso non trovato" })),
            )
                .into_response();
        }

        let mut all_modules = vec![];
        let mut combined_primitives = PrimitiveRegistry::empty();

        if path.is_file() {
            if let Some(lang) = SupportedLanguage::from_path(path) {
                if let Ok(source) = std::fs::read_to_string(path) {
                    let rel_path = path.file_name().map(Path::new).unwrap_or(path);
                    if let Ok((mut file_modules, file_primitives)) =
                        parse_source(lang, &source, rel_path, &config)
                    {
                        all_modules.append(&mut file_modules);
                        combined_primitives.merge(file_primitives);
                    }
                }
            }
        } else {
            for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file()
                    && let Some(lang) = SupportedLanguage::from_path(entry.path())
                    && let Ok(source) = std::fs::read_to_string(entry.path())
                {
                    let rel_path = entry.path().strip_prefix(path).unwrap_or(entry.path());
                    if let Ok((mut file_modules, file_primitives)) =
                        parse_source(lang, &source, rel_path, &config)
                    {
                        all_modules.append(&mut file_modules);
                        combined_primitives.merge(file_primitives);
                    }
                }
            }
        }
        (all_modules, combined_primitives)
    } else {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Fornire 'path' o 'code'+'extension'" })),
        )
            .into_response();
    };

    let (_, graph, _) = analyze_project(modules, primitives, &config);

    (
        StatusCode::OK,
        axum::Json(serde_json::json!({
            "rawOutput": graph,
            "elements": omnideps::export::cytoscape::convert_to_cyto_elements(&[graph])
        }))
    ).into_response()
}

// -----------------------------------------------------------------------------
// BENCHMARKS API
// -----------------------------------------------------------------------------
fn walk_benchmark_files(dir: &Path, files_list: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                walk_benchmark_files(&path, files_list);
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if matches!(ext, "java" | "rs" | "py" | "c" | "cpp" | "cxx" | "cc" | "hxx" | "hpp" | "h") {
                    if let Some(path_str) = path.to_str() {
                        files_list.push(path_str.to_string());
                    }
                }
            }
        }
    }
}

async fn api_benchmarks() -> impl IntoResponse {
    let bench_generics = Path::new("tests/generics");
    let bench_suites = Path::new("tests/benchmarks");
    
    let mut files = vec![];
    if bench_generics.exists() {
        walk_benchmark_files(bench_generics, &mut files);
    }
    if bench_suites.exists() {
        if let Ok(entries) = std::fs::read_dir(bench_suites) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("benchmark-") {
                            if let Some(path_str) = path.to_str() {
                                files.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    (StatusCode::OK, axum::Json(files)).into_response()
}

async fn api_benchmark_suites() -> impl IntoResponse {
    let bench_suites = Path::new("tests/benchmarks");
    let mut suites = vec![];
    
    if bench_suites.exists() {
        if let Ok(entries) = std::fs::read_dir(bench_suites) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("benchmark-") {
                            if let Some(path_str) = path.to_str() {
                                suites.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    (StatusCode::OK, axum::Json(suites)).into_response()
}

async fn api_run_benchmark(Json(payload): Json<RunBenchmarkRequest>) -> impl IntoResponse {
    let bench_path_str = payload.benchmark;
    if !bench_path_str.starts_with("tests/benchmarks/benchmark-") || bench_path_str.contains("..") {
        return (StatusCode::BAD_REQUEST, axum::Json(serde_json::json!({ "error": "Invalid benchmark path" }))).into_response();
    }

    let target_path = PathBuf::from(&bench_path_str);
    let config = AnalyzerConfig::default_strategies();

    if let Err(e) = crate::commands::benchmark::execute_run(&target_path, None, &config) {
        return (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": "Failed to run benchmark", "details": e.to_string() }))).into_response();
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
fn build_docs_tree(dir: &Path, base_path: &str) -> Vec<DocsNode> {
    let mut tree = vec![];
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            
            let full_path = entry.path();
            let relative_path = if base_path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", base_path, name)
            };
            
            if full_path.is_dir() {
                let children = build_docs_tree(&full_path, &relative_path);
                if !children.is_empty() {
                    tree.push(DocsNode {
                        node_type: "directory".to_string(),
                        name,
                        path: relative_path,
                        children: Some(children),
                    });
                }
            } else if full_path.is_file() && (name.ends_with(".md") || name.ends_with(".html")) {
                tree.push(DocsNode {
                    node_type: "file".to_string(),
                    name,
                    path: relative_path,
                    children: None,
                });
            }
        }
    }
    
    tree.sort_by(|a, b| {
        if a.node_type == b.node_type {
            a.name.cmp(&b.name)
        } else if a.node_type == "directory" {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    
    tree
}

async fn api_docs_tree() -> impl IntoResponse {
    let docs_dir = Path::new("docs");
    if !docs_dir.exists() {
        return (StatusCode::OK, axum::Json(serde_json::json!({ "tree": [] }))).into_response();
    }
    
    let tree = build_docs_tree(docs_dir, "");
    (StatusCode::OK, axum::Json(serde_json::json!({ "tree": tree }))).into_response()
}

async fn api_docs_content(Query(query): Query<DocsQuery>) -> impl IntoResponse {
    let file_path_str = query.path;
    let docs_dir = Path::new("docs");
    
    // Simple path traversal prevention
    if file_path_str.contains("..") {
        return (StatusCode::FORBIDDEN, axum::Json(serde_json::json!({ "error": "Invalid path" }))).into_response();
    }
    
    let absolute_path = docs_dir.join(&file_path_str);
    if !absolute_path.exists() {
        return (StatusCode::NOT_FOUND, axum::Json(serde_json::json!({ "error": "File not found" }))).into_response();
    }
    
    match std::fs::read_to_string(&absolute_path) {
        Ok(content) => {
            let is_html = absolute_path.extension().and_then(|e| e.to_str()) == Some("html");
            (StatusCode::OK, axum::Json(serde_json::json!({ "content": content, "isHtml": is_html }))).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
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
