# InfiniteAPI

![Rust](https://img.shields.io/badge/rust-stable-orange.svg)

A dynamic HTTP server that simulates any API endpoint using Google's Gemini AI. InfiniteAPI analyzes incoming requests and generates realistic responses based on endpoint patterns.

## ✨ Features

-   **Dynamic Response Generation**: Creates realistic responses for any endpoint path
-   **Multiple Response Types**:
    -   🌐 **HTML pages** for web endpoints
    -   📊 **JSON data** for API endpoints
    -   📝 **Text** for simple resources
    -   📁 **Binary** descriptions for file endpoints
    -   ↪️ **Redirects** for moved resources
    -   ⚠️ **Error responses** with appropriate HTTP status codes
-   **Content-Type Awareness**: Sets appropriate MIME types based on response type
-   **Realistic Mock Data**: Generates plausible data structures based on endpoint semantics

## 🚀 Getting Started

### Prerequisites

-   Google Gemini API key

### Installation

1. Download the release from releases page

2. Set your Gemini API key as an environment variable:

Windows
```bash
set GEMINI_API_KEY=your_api_key_here
```
Linux/macOS
```bash
export GEMINI_API_KEY=your_api_key_here
```
3. Run server

### Installation (building for you own system)

1. Clone the repository:
```bash
git clone https://github.com/yourusername/infinite_api.git
cd infinite_api
```
2. Set your Gemini API key as an environment variable:

Windows
```bash
set GEMINI_API_KEY=your_api_key_here
```
Linux/macOS
```bash
export GEMINI_API_KEY=your_api_key_here
```
3. Build the project:
```bash
cargo build --release
```

## 🔧 Usage

1. Start the server (manual mode)
```bash
cargo run --release
```

3. The server will listen on http://127.0.0.1:1337

4. Access any endpoint to see dynamically generated responses:
```
http://127.0.0.1:1337/api/users
http://127.0.0.1:1337/products/123
http://127.0.0.1:1337/login
http://127.0.0.1:1337/about
http://127.0.0.1:1337/status
```

5. Observe how different endpoints return different response types with appropriate content

## 📖 How It Works

1. InfiniteAPI receives an HTTP request for any path
2. The server forwards the request path to Google's Gemini AI
3. The AI analyzes the path and generates a realistic response based on endpoint conventions
4. The server transforms the AI response into an appropriate HTTP response
5. The client receives a response that mimics what a real API would return

## 🛠️ Configuration

You can configure the behavior by setting these environment variables:

-   GEMINI_API_KEY: Your Google Gemini API key (required)
-   GEMINI_MODEL: The model to use (defaults to "gemini-2.0-flash")
