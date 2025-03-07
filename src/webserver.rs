use crate::ClapHandler;
use crate::write::write_multipart_file;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Multipart};
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use axum::serve::ListenerExt;
use axum::{Router, response::IntoResponse, routing::post};

async fn accept_form(mut multipart: Multipart) -> StatusCode {
    let mut name: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut data: Option<Bytes> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        println!("Field received: {:?}", field.name());
        if let Some(n) = field.name() {
            name = Some(n.to_string());
        };
        if let Some(f_n) = field.file_name() {
            file_name = Some(f_n.clone().to_string())
        }
        if let Some(f) = field.content_type() {
            content_type = Some(f.to_string());
        }
        if let Some(c) = field.content_type() {
            content_type = Some(c.to_string());
        }
        if let Ok(bytes) = field.bytes().await {
            println!("Bytes received: {:?}", bytes.len());
            data = Some(bytes.clone());
        }

        println!(
            "Length of `{name:?}` (`{file_name:?}`: `{content_type:?}`) is {} bytes",
            match &data {
                Some(v) => v.len(),
                _ => 0,
            }
        );
    }

    if name.is_none() && file_name.is_none() && data.is_none() {
        return StatusCode::BAD_REQUEST;
    } else {
       if let Err(e) = write_multipart_file(file_name.unwrap(), name.unwrap(), content_type.unwrap(), data.unwrap()) {
           println!("Error writing to Multipart: {}", e);
       }
        return StatusCode::CREATED;
    }
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>GPX File Upload</title>
</head>
<body>
    <h2>Upload GPX Files (Minimum 2, Maximum 5)</h2>
    <form id="uploadForm">
        <input type="file" name="file" accept=".gpx" required> <br><br>
        <input type="file" name="file" accept=".gpx"> <br><br>
        <input type="file" name="file" accept=".gpx"> <br><br>
        <input type="file" name="file" accept=".gpx"> <br><br>
        <input type="file" name="file" accept=".gpx"> <br><br>
        <button type="submit">Upload</button>
    </form>

    <p id="status"></p>

    <script>
        document.getElementById("uploadForm").addEventListener("submit", async function(event) {
            event.preventDefault();
            const files = Array.from(document.querySelectorAll('input[type="file"]')).map(input => input.files[0]).filter(file => file);

            if (files.length < 1) {
                document.getElementById("status").textContent = "Please select at least 2 files.";
                return;
            }

            document.getElementById("status").textContent = "Uploading files...";

            let uploadPromises = files.map(async (file) => {
                let formData = new FormData();
                formData.append("file", file);

                let response = await fetch("/upload", {
                    method: "POST",
                    body: formData
                });

                return response.ok;
            });

            let results = await Promise.all(uploadPromises);
            if (results.every(success => success)) {
                document.getElementById("status").textContent = "All files uploaded successfully!";
            } else {
                document.getElementById("status").textContent = "Some files failed to upload.";
            }
        });
    </script>
</body>
</html>"#;

async fn home_page() -> impl IntoResponse {
    Html(INDEX_HTML)
}

const MEGABYTE_SIZE: usize = 1024 * 1000;

pub async fn handle_web(handler: &ClapHandler) {
    let app = Router::new()
        .route("/upload", post(accept_form))
        .route("/", get(home_page))
        .layer(DefaultBodyLimit::max(MEGABYTE_SIZE * 50));

    let bind = "127.0.0.1:8080".to_string();
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .expect("Cannot bind server to address and port")
        .tap_io(|tcp_stream| {
            if let Err(err) = tcp_stream.set_nodelay(true) {
                println!("failed to set TCP_NODELAY on incoming connection: {err:#}");
            }
        });

    let _ = axum::serve(listener, app).await.unwrap();
}
