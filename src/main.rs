use std::{env, net::SocketAddr, sync::LazyLock};

use bytes::Bytes;
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::{
    Method, Request, Response, Result, StatusCode, server::conn::http1, service::service_fn,
};
use hyper_util::rt::TokioIo;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    response_type: String,
    status: String,
    content: String,
    metadata: Metadata,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    content_type: String,
}

#[derive(Debug, Deserialize)]
struct GeminiTextResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
}
#[derive(Debug, Deserialize)]
struct Content {
    parts: Vec<Part>,
}
#[derive(Debug, Deserialize)]
struct Part {
    text: String,
}

static KEY: LazyLock<String> = LazyLock::new(|| {
    env::var("GEMINI_API_KEY").unwrap_or_else(|_| {
        eprintln!("Error: GEMINI_API_KEY environment variable not set");
        std::process::exit(1);
    })
});

static MODEL: LazyLock<String> =
    LazyLock::new(|| env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.0-flash".to_string()));

static PROMPT: &str = "You are InfiniteAPI, an AI designed to pass API responses for any endpoint. For each request path I provide, generate a JSON object that represents what a real API might return for that endpoint.

Analyze the endpoint path and return a JSON object with the following structure:

{
  \"response_type\": \"[type of response]\",
  \"status\": \"[HTTP status code in string]\",
  \"content\": \"[appropriate content based on endpoint in string]\",
  \"metadata\": {
    \"content_type\": \"[appropriate MIME type]\",
  }
}

Based on the endpoint pattern, choose from these response types:

1. \"html\" - For webpage endpoints (/about, /user/profile, etc.)
   - Include realistic HTML (with CSS) content with appropriate structure
   - Use content-type: text/html

2. \"json\" - For API endpoints (/api/users, /data/metrics, etc.)
   - Generate realistic data structures matching the resource name
   - Use content-type: application/json

3. \"text\" - For simple responses (/status, /version, etc.)
   - Return plain text appropriate to the endpoint
   - Use content-type: text/plain

4. \"binary\" - For file endpoints (/download/file, /images/, etc.)
   - Indicate binary data with a description (actual binary would be handled by the server)
   - Use appropriate content-type (application/octet-stream, image/jpeg, etc.)

5. \"redirect\" - For endpoints that would redirect (/login, /old-page, etc.)
   - Include target location
   - Use appropriate 3xx status code

6. \"error\" - For invalid requests, authentication failures, etc.
   - Use appropriate error status code (400, 401, 403, 404, 500)
   - Include meaningful error messages

Be creative and realistic with your responses. Consider REST conventions and HTTP semantics when determining the appropriate response type.

Examples of endpoints to respond to:
- /api/users
- /products/123
- /login
- /docs/guide
- /status
- /download/report.pdf\nRESPONSE ONLY JSON OBJECT";

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = "127.0.0.1:1337".parse().unwrap();

    let listener = TcpListener::bind(addr).await?;
    println!("Listening on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(response_examples))
                .await
            {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}

async fn response_examples(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, std::io::Error>>> {
    if req.method() != Method::GET {
        return Ok(not_found());
    }
    let path = req.uri().path();

    if path == "/favicon.ico" {
        return Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Full::new(Bytes::new()).map_err(|e| match e {}).boxed())
            .unwrap());
    }

    let request_body = json!({
        "contents": [
            {
                "role": "user",
                "parts": [
                    {
                        "text": PROMPT.to_string() + &format!("Response to endpoint Endpoint: {}", path)
                    }
                ]
            }]
    });

    let client = Client::new();
    let resp = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            MODEL.as_str(),
            KEY.as_str()
        ))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Failed to send request");
    let body = resp.bytes().await.expect("Failed to read response body");
    let body = String::from_utf8(body.to_vec()).expect("Failed to convert bytes to string");

    let candidates: GeminiTextResponse =
        serde_json::from_str(&body).expect("Failed to parse JSON. Recheck connection and API key.");

    let json_response: GeminiResponse = serde_json::from_str(
        &candidates.candidates[0].content.parts[0]
            .text
            .replace("```json", "")
            .replace("```", "")
            .replace("\n", ""),
    )
    .expect("Failed to parse Gemini response");

    match json_response.response_type.as_str() {
        "redirect" => Ok(Response::builder()
            .status(StatusCode::MOVED_PERMANENTLY)
            .header("Location", json_response.content)
            .body(
                Full::new("Redirecting...".to_string().into())
                    .map_err(|never| match never {})
                    .boxed(),
            )
            .unwrap()),
        "error" => Ok(Response::builder()
            .status(json_response.status.parse::<u16>().unwrap())
            .header("Content-Type", json_response.metadata.content_type)
            .body(
                Full::new(json_response.content.into())
                    .map_err(|e| match e {})
                    .boxed(),
            )
            .unwrap()),
        _ => {
            let body = Full::new(json_response.content.into())
                .map_err(|e| match e {})
                .boxed();
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", json_response.metadata.content_type)
                .body(body)
                .unwrap())
        }
    }
}

fn not_found() -> Response<BoxBody<Bytes, std::io::Error>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(
            Full::new("Not Found".to_string().into())
                .map_err(|e| match e {})
                .boxed(),
        )
        .unwrap()
}
