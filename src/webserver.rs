use crate::ClapHandler;
use crate::merge::merge_traces;
use crate::read::read_gpx;
use crate::write::write_multipart_file;
use axum::body::{Body, Bytes};
use axum::extract::{DefaultBodyLimit, Multipart};
use axum::http::StatusCode;
use axum::response::{Html, Response};
use axum::routing::get;
use axum::serve::ListenerExt;
use axum::{Json, Router, response::IntoResponse, routing::post};
use gpx::Gpx;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TriggerBody {
    hashes: Vec<String>,
}

#[axum::debug_handler]
async fn send_to_trigger(Json(body): Json<TriggerBody>) -> Result<Response, StatusCode> {
    // Ok(Response::default())

    if body.hashes.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut gpx_extract: Vec<Gpx> = Vec::new();

    for item in body.hashes {
        let file_path = format!("{}/{}.gpx", "traces_import", item);
        let r = read_gpx(&file_path);
        match r {
            Ok(gpx) => gpx_extract.push(gpx),
            Err(e) => {
                println!("Error reading gpx: {} ({})", e, file_path);
                return Err(StatusCode::BAD_REQUEST);
            }
        }
    }

    let merged = merge_traces(&gpx_extract);

    let mut vec = Vec::new();
    if let Err(e) = gpx::write(&merged, &mut vec) {
        println!("Error writing traces: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let string = match String::from_utf8(vec) {
        Ok(s) => s,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from(string))
        .expect("Failed to create response"))

    // Ok((axum::http::header::CONTENT_TYPE, "text/plain"), Bytes::from(contents))
}

#[derive(Serialize, Clone, Debug)]
pub struct ReplyCreate {
    pub hash: String,
}

async fn accept_form(mut multipart: Multipart) -> Result<Response, StatusCode> {
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
            file_name = Some(f_n.to_string())
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
        Err(StatusCode::BAD_REQUEST)
    } else {
        let t = write_multipart_file(
            file_name.unwrap(),
            data.unwrap(),
        );
        if let Err(e) = t {
            println!("Error writing to Multipart: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        } else {
            let file_hash = t.unwrap();
            Ok(Response::builder()
                .status(StatusCode::CREATED)
                .body(Body::from(
                    serde_json::to_string(&ReplyCreate { hash: file_hash }).unwrap(),
                ))
                .expect("Failed to create response"))
        }
    }
}

const INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Upload GPX Files</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; padding: 20px; }
        input[type="file"] { display: block; margin: 10px auto; }
        button { margin-top: 10px; padding: 10px; }
        #status { margin-top: 20px; font-weight: bold; color: green; }
    </style>
</head>
<body>
    <h2>Upload GPX Files</h2>
    <p>Upload up to 5 files (at least 2 required).</p>

    <form id="uploadForm">
        <input type="file" name="file" id="file1" required>
        <input type="file" name="file" id="file2" required>
        <input type="file" name="file" id="file3">
        <input type="file" name="file" id="file4">
        <input type="file" name="file" id="file5">
        <button type="submit">Upload</button>
    </form>

    <p id="status"></p>

    <script>
        document.getElementById("uploadForm").addEventListener("submit", async function(event) {
            event.preventDefault();

            let fileInputs = document.querySelectorAll('input[type="file"]');
            let selectedFiles = 0;
            let hashes = []; // Store all file hashes

            let uploadPromises = [];

            for (let input of fileInputs) {
                if (input.files.length > 0) {
                    selectedFiles++;

                    let formData = new FormData();
                    formData.append("file", input.files[0]);

                    let uploadPromise = fetch("/upload", {
                        method: "POST",
                        body: formData
                    })
                    .then(response => response.json())
                    .then(data => {
                        if (data.hash) {
                            hashes.push(data.hash);
                        }
                    })
                    .catch(error => {
                        console.error("Upload error:", error);
                    });

                    uploadPromises.push(uploadPromise);
                }
            }

            if (selectedFiles < 2) {
                document.getElementById("status").innerText = "Please select at least 2 files.";
                document.getElementById("status").style.color = "red";
                return;
            }

            document.getElementById("status").innerText = "Uploading files...";
            document.getElementById("status").style.color = "blue";

            try {
                await Promise.all(uploadPromises);

                if (hashes.length > 0) {
                    document.getElementById("status").innerText = "Files uploaded! Sending hashes...";
                    document.getElementById("status").style.color = "green";

                    // Send all hashes to /trigger
                    let triggerResponse = await fetch("/trigger", {
                        method: "POST",
                        headers: { "Content-Type": "application/json" },
                        body: JSON.stringify({ "hashes": hashes })
                    });

                    if (!triggerResponse.ok) throw new Error("Trigger request failed");

                    let triggerData = await triggerResponse.text(); // Get response as text

                    // Create file and download
                    let blob = new Blob([triggerData], { type: "text/plain" });
                    let link = document.createElement("a");
                    link.href = URL.createObjectURL(blob);
                    link.download = "merged_files.gpx";
                    document.body.appendChild(link);
                    link.click();
                    document.body.removeChild(link);

                    document.getElementById("status").innerText = "Hashes sent successfully! File downloaded.";
                } else {
                    throw new Error("No hashes received.");
                }
            } catch (error) {
                document.getElementById("status").innerText = "Error: " + error.message;
                document.getElementById("status").style.color = "red";
            }
        });
    </script>
</body>
</html>
"#;

async fn home_page() -> impl IntoResponse {
    Html(INDEX_HTML)
}

const MEGABYTE_SIZE: usize = 1024 * 1000;

pub async fn handle_web(args: ClapHandler) {
    let app = Router::new()
        .route("/trigger", post(send_to_trigger))
        .route("/upload", post(accept_form))
        .route("/", get(home_page))
        .layer(DefaultBodyLimit::max(MEGABYTE_SIZE * 50));

    println!("Binding to {}", args.webserver_bind);
    let listener = tokio::net::TcpListener::bind(&args.webserver_bind)
        .await
        .expect("Cannot bind server to address and port")
        .tap_io(|tcp_stream| {
            if let Err(err) = tcp_stream.set_nodelay(true) {
                println!("failed to set TCP_NODELAY on incoming connection: {err:#}");
            }
        });

    axum::serve(listener, app).await.unwrap();
}
