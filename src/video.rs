use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::{fs, io};

pub async fn handle_upload(mut multipart: Multipart) -> impl IntoResponse {
    fs::create_dir_all("./uploads/temp").expect("Failed to create directory");

    let mut file_name = String::new();
    let mut chunk_number = 0;
    let mut total_chunks = 0;

    while let Some(field) = match multipart.next_field().await {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Error reading multipart field: {:?}", err);
            return StatusCode::BAD_REQUEST;
        }
    } {
        let field_name = field.name().unwrap_or_default();
        match field_name {
            "fileName" => file_name = field.text().await.unwrap_or_default(),
            "chunkNumber" => {
                chunk_number = field.text().await.unwrap_or_default().parse().unwrap_or(0)
            }
            "totalChunks" => {
                total_chunks = field.text().await.unwrap_or_default().parse().unwrap_or(0)
            }
            "chunk" => {
                let file_bytes = field.bytes().await.expect("Failed to extract bytes");
                let file_name = format!("./uploads/temp/{}-{}.part", file_name, chunk_number);
                OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(file_name)
                    .expect("Failed to open file")
                    .write_all(&file_bytes)
                    .expect("Failed to write to file");
            }
            _ => {}
        }
    }

    let upload_complete = verify_upload(&file_name, total_chunks);

    if upload_complete {
        merge_parts(&file_name, total_chunks).expect("Failed to merge chunks");
    }

    StatusCode::OK
}

fn verify_upload(file_name: &str, total_chunks: usize) -> bool {
    let mut upload_complete = true;

    for index in 1..=total_chunks {
        let file_name = format!("./uploads/temp/{}-{}.part", file_name, index);
        if !Path::new(&file_name).exists() {
            upload_complete = false;
            break;
        }
    }

    upload_complete
}

fn merge_parts(file_name: &str, total_chunks: usize) -> anyhow::Result<()> {
    let mut destination = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(format!("./uploads/{}", file_name))?;

    for index in 1..=total_chunks {
        let file_name = format!("./uploads/temp/{}-{}.part", file_name, index);
        let mut source_file = File::open(&file_name)?;
        io::copy(&mut source_file, &mut destination)?;
        fs::remove_file(&file_name)?;
    }

    Ok(())
}
