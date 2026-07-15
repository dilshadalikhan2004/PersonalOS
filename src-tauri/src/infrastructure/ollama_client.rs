use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use super::storage_error::StorageError;

const OLLAMA_ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 11434);
pub const GENERATION_MODEL: &str = "qwen2.5:3b-instruct";
pub const EMBEDDING_MODEL: &str = "nomic-embed-text";

pub struct LocalOllamaClient;

impl LocalOllamaClient {
    pub fn generate_json(prompt: &str, format: serde_json::Value) -> Result<String, StorageError> {
        let request = serde_json::json!({
            "model": GENERATION_MODEL,
            "stream": false,
            "format": format,
            "options": {"temperature": 0},
            "prompt": prompt
        });
        let response: GenerateResponse = serde_json::from_slice(&post_json("/api/generate", &request, Duration::from_secs(120))?)?;
        Ok(response.response)
    }

    pub fn embed(input: &str) -> Result<Vec<f32>, StorageError> {
        let request = serde_json::json!({
            "model": EMBEDDING_MODEL,
            "input": input
        });
        let response: EmbedResponse = serde_json::from_slice(&post_json("/api/embed", &request, Duration::from_secs(60))?)?;
        response.embeddings.into_iter().next().filter(|vector| !vector.is_empty()).ok_or(StorageError::InvalidModelResponse)
    }

    pub fn stream_generate(prompt: &str, mut on_token: impl FnMut(&str)) -> Result<String, StorageError> {
        let request = serde_json::json!({
            "model": GENERATION_MODEL,
            "stream": true,
            "options": {"temperature": 0},
            "prompt": prompt
        });
        let body = serde_json::to_vec(&request)?;
        let mut stream = open_stream(Duration::from_secs(300), Duration::from_secs(10))?;
        write_request(&mut stream, "/api/generate", body.len())?;
        stream.write_all(&body).map_err(StorageError::Io)?;
        let mut reader = BufReader::new(stream);
        read_headers(&mut reader)?;
        let mut answer = String::new();
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line).map_err(StorageError::Io)?;
            if bytes == 0 {
                break;
            }
            if line.trim().is_empty() {
                continue;
            }
            let chunk: GenerateChunk = serde_json::from_str(line.trim()).map_err(|_| StorageError::InvalidModelResponse)?;
            if !chunk.response.is_empty() {
                on_token(&chunk.response);
                answer.push_str(&chunk.response);
            }
            if chunk.done {
                break;
            }
        }
        Ok(answer)
    }
}

fn post_json(path: &str, request: &serde_json::Value, read_timeout: Duration) -> Result<Vec<u8>, StorageError> {
    let body = serde_json::to_vec(request)?;
    let mut stream = open_stream(read_timeout, Duration::from_secs(10))?;
    write_request(&mut stream, path, body.len())?;
    stream.write_all(&body).map_err(StorageError::Io)?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(StorageError::Io)?;
    parse_http_response(&response).map(|body| body.to_vec())
}

fn open_stream(read_timeout: Duration, write_timeout: Duration) -> Result<TcpStream, StorageError> {
    let stream = TcpStream::connect_timeout(&OLLAMA_ADDRESS, Duration::from_secs(2)).map_err(|_| StorageError::OllamaUnavailable)?;
    stream.set_read_timeout(Some(read_timeout)).map_err(StorageError::Io)?;
    stream.set_write_timeout(Some(write_timeout)).map_err(StorageError::Io)?;
    Ok(stream)
}

fn write_request(stream: &mut TcpStream, path: &str, content_length: usize) -> Result<(), StorageError> {
    let header = format!("POST {path} HTTP/1.1\r\nHost: 127.0.0.1:11434\r\nContent-Type: application/json\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n");
    stream.write_all(header.as_bytes()).map_err(StorageError::Io)
}

fn read_headers(reader: &mut BufReader<TcpStream>) -> Result<(), StorageError> {
    let mut status = String::new();
    reader.read_line(&mut status).map_err(StorageError::Io)?;
    if !status.starts_with("HTTP/1.1 200") && !status.starts_with("HTTP/1.0 200") {
        return Err(StorageError::OllamaUnavailable);
    }
    let mut line = String::new();
    loop {
        line.clear();
        reader.read_line(&mut line).map_err(StorageError::Io)?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
    }
    Ok(())
}

fn parse_http_response(response: &[u8]) -> Result<&[u8], StorageError> {
    let split = response.windows(4).position(|part| part == b"\r\n\r\n").ok_or(StorageError::OllamaUnavailable)?;
    if !response.starts_with(b"HTTP/1.1 200") && !response.starts_with(b"HTTP/1.0 200") {
        return Err(StorageError::OllamaUnavailable);
    }
    Ok(&response[split + 4..])
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Deserialize)]
struct GenerateChunk {
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
}

#[derive(Serialize, Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}
