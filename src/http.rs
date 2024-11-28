use std::{fs, io::Write, path::Path};

pub fn get_body(url: &str) -> Result<Vec<u8>, String> {
    let mut res: Vec<u8> = Vec::new();
    http_req::request::get(url, &mut res)
        .map_err(|e| format!("Failed to download from {}: {:#?}", url, e))?;
    Ok(res)
}

pub fn get_body_string(url: &str) -> Result<String, String> {
    let body = get_body(url)?;
    String::from_utf8(body).map_err(|e| format!("Failed to parse response as UTF-8: {:#?}", e))
}

pub fn download_file(url: &str, file_path: &Path) -> Result<(), String> {
    let body = get_body(url)?;

    let parent = file_path
        .parent()
        .ok_or_else(|| format!("Failed to get parent directory of {}", file_path.display()))?;

    fs::create_dir_all(parent)
        .map_err(|e| format!("Failed to create directory {}: {:#?}", parent.display(), e))?;

    fs::File::create(file_path)
        .map_err(|e| format!("Failed to create file {}: {:#?}", file_path.display(), e))?
        .write_all(&body)
        .map_err(|e| format!("Failed to write to file {}: {:#?}", file_path.display(), e))?;

    Ok(())
}
