use actix_web::{web, App, HttpServer, HttpResponse};
use actix_web::web::Bytes;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

const WORKSPACE: &str = "/home/omrylcn/lllm-control";

#[derive(Deserialize)]
struct FileQuery {
    path: String,
}

#[derive(Deserialize)]
struct SaveRequest {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct RunRequest {
    command: String,
}

#[derive(Serialize)]
struct FileEntry {
    name: String,
    is_dir: bool,
}

#[derive(Serialize)]
struct RunResult {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

fn safe_path(requested: &str) -> Option<PathBuf> {
    let base = PathBuf::from(WORKSPACE);
    let full = base.join(requested);
    let canonical = full.canonicalize().ok()?;
    if canonical.starts_with(&base) {
        Some(canonical)
    } else {
        None
    }
}

async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("static/index.html"))
}

async fn list_files(query: web::Query<FileQuery>) -> HttpResponse {
    let dir = match safe_path(&query.path) {
        Some(p) => p,
        None => return HttpResponse::BadRequest().json(serde_json::json!({"error": "invalid path"})),
    };

    let mut entries = Vec::new();
    if let Ok(mut read_dir) = tokio::fs::read_dir(&dir).await {
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            entries.push(FileEntry { name, is_dir });
        }
    }
    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
    HttpResponse::Ok().json(entries)
}

async fn read_file(query: web::Query<FileQuery>) -> HttpResponse {
    let path = match safe_path(&query.path) {
        Some(p) => p,
        None => return HttpResponse::BadRequest().json(serde_json::json!({"error": "invalid path"})),
    };
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => HttpResponse::Ok().json(serde_json::json!({"content": content})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn save_file(body: web::Json<SaveRequest>) -> HttpResponse {
    let path = match safe_path(&body.path) {
        Some(p) => p,
        None => {
            let p = PathBuf::from(WORKSPACE).join(&body.path);
            if !p.starts_with(WORKSPACE) {
                return HttpResponse::BadRequest().json(serde_json::json!({"error": "invalid path"}));
            }
            p
        }
    };
    match tokio::fs::write(&path, &body.content).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "saved"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn upload_file(query: web::Query<FileQuery>, body: Bytes) -> HttpResponse {
    let base = PathBuf::from(WORKSPACE);
    let target = base.join(&query.path);
    if !target.starts_with(&base) {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "invalid path"}));
    }
    if let Some(parent) = target.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    match tokio::fs::write(&target, &body).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "uploaded", "path": query.path})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn download_file(query: web::Query<FileQuery>) -> HttpResponse {
    let path = match safe_path(&query.path) {
        Some(p) => p,
        None => return HttpResponse::BadRequest().json(serde_json::json!({"error": "invalid path"})),
    };
    match tokio::fs::read(&path).await {
        Ok(data) => {
            let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            HttpResponse::Ok()
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                .insert_header(("Content-Type", "application/octet-stream"))
                .body(data)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

#[derive(Deserialize)]
struct CompleteRequest {
    partial: String,
}

async fn tab_complete(body: web::Json<CompleteRequest>) -> HttpResponse {
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "export PATH=$HOME/.local/bin:$PATH && cd {} && compgen -f -- '{}'",
            WORKSPACE, body.partial
        ))
        .output()
        .await;

    match output {
        Ok(out) => {
            let suggestions: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.to_string())
                .collect();
            HttpResponse::Ok().json(suggestions)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

async fn run_command(body: web::Json<RunRequest>) -> HttpResponse {
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!(
            "export PATH=$HOME/.local/bin:$PATH && cd {} && {}",
            WORKSPACE, body.command
        ))
        .output()
        .await;

    match output {
        Ok(out) => {
            let result = RunResult {
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code().unwrap_or(-1),
            };
            HttpResponse::Ok().json(result)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Pi Editor running at http://0.0.0.0:3000");
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/api/files", web::get().to(list_files))
            .route("/api/file", web::get().to(read_file))
            .route("/api/file", web::post().to(save_file))
            .route("/api/run", web::post().to(run_command))
            .route("/api/complete", web::post().to(tab_complete))
            .route("/api/upload", web::post().to(upload_file))
            .route("/api/download", web::get().to(download_file))
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
