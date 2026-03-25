use clap::Parser;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tiny_http::{Response, Server};

use litquid::{codegen::{build_emitters, TargetEmitter}, process_liquid_file, DEFAULT_LIT_IMPORT};

/// LitQuid Watch Server - File watcher with SSE live reload
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory containing .liquid files
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for generated .template.js files
    #[arg(short, long)]
    output: PathBuf,

    /// HTTP port for the live reload server
    #[arg(short, long, default_value = "35729")]
    port: u16,

    /// Import path for Lit (e.g., "lit", "https://cdn.jsdelivr.net/npm/lit@3/+esm")
    #[arg(long, default_value = DEFAULT_LIT_IMPORT)]
    lit_import: String,

    /// Additional server-side render targets (comma-separated). Supported: csharp
    #[arg(long)]
    emit: Option<String>,

    /// C# namespace for generated code (used with --emit csharp)
    #[arg(long, default_value = "LitQuid.Generated")]
    namespace: String,
}

/// Manages SSE client connections
struct SseClients {
    clients: Vec<Sender<()>>,
}

impl SseClients {
    fn new() -> Self {
        Self { clients: Vec::new() }
    }

    fn add(&mut self, sender: Sender<()>) {
        self.clients.push(sender);
    }

    fn broadcast_reload(&mut self) {
        // Send to all clients, remove any that have disconnected
        self.clients.retain(|client| client.send(()).is_ok());
    }
}

fn main() {
    let args = Args::parse();

    if !args.input.exists() {
        eprintln!("Error: Input directory does not exist: {:?}", args.input);
        std::process::exit(1);
    }

    std::fs::create_dir_all(&args.output).expect("Failed to create output directory");

    let emitters: Arc<Vec<Box<dyn TargetEmitter>>> =
        Arc::new(build_emitters(args.emit.as_deref(), &args.namespace));

    // Shared state for SSE clients
    let sse_clients: Arc<Mutex<SseClients>> = Arc::new(Mutex::new(SseClients::new()));
    let sse_clients_watcher = Arc::clone(&sse_clients);

    // Canonicalize paths for consistent path comparison
    let input_dir = args.input.canonicalize().expect("Failed to canonicalize input path");
    let output_dir = args.output.canonicalize().expect("Failed to canonicalize output path");
    let lit_import = args.lit_import.clone();

    // Initial processing of all templates
    println!("[litquid-watch] Initial processing...");
    process_all_templates(&input_dir, &output_dir, &lit_import, &emitters);
    println!("[litquid-watch] Ready!");

    // Start file watcher in a thread
    let watcher_input = input_dir.clone();
    let watcher_output = output_dir.clone();
    let watcher_lit_import = lit_import.clone();
    let watcher_emitters = Arc::clone(&emitters);
    thread::spawn(move || {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(100)),
        )
        .expect("Failed to create watcher");

        watcher
            .watch(&watcher_input, RecursiveMode::Recursive)
            .expect("Failed to watch directory");

        println!("[litquid-watch] Watching {:?} for changes...", watcher_input);

        // Debounce: collect events for a short period
        let mut pending_files: Vec<PathBuf> = Vec::new();
        let debounce_duration = Duration::from_millis(50);

        loop {
            match rx.recv_timeout(debounce_duration) {
                Ok(event) => {
                    for path in event.paths {
                        if path.extension().map_or(false, |ext| ext == "liquid") {
                            if !pending_files.contains(&path) {
                                pending_files.push(path);
                            }
                        }
                    }
                }
                Err(_) => {
                    // Timeout - process any pending files
                    if !pending_files.is_empty() {
                        let mut any_success = false;

                        for path in pending_files.drain(..) {
                            println!("[litquid-watch] Processing: {:?}", path);

                            // Canonicalize the changed file path to match our canonicalized input path
                            let canonical_path = match path.canonicalize() {
                                Ok(p) => p,
                                Err(e) => {
                                    eprintln!("[litquid-watch] Failed to canonicalize path {:?}: {}", path, e);
                                    continue;
                                }
                            };

                            let relative_path = match canonical_path.strip_prefix(&watcher_input) {
                                Ok(p) => p,
                                Err(_) => {
                                    eprintln!("[litquid-watch] Path {:?} is not under {:?}", canonical_path, watcher_input);
                                    continue;
                                }
                            };

                            let output_js_path = watcher_output
                                .join(relative_path)
                                .with_extension("template.js");
                            let output_json_path = watcher_output
                                .join(relative_path)
                                .with_extension("template.json");

                            if let Some(parent) = output_js_path.parent() {
                                std::fs::create_dir_all(parent).ok();
                            }

                            let template_name = canonical_path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("template");

                            match process_liquid_file(&canonical_path, Some(&watcher_lit_import)) {
                                Ok(parsed) => {
                                    std::fs::write(&output_js_path, parsed.to_js_module()).ok();
                                    std::fs::write(&output_json_path, parsed.to_json_manifest()).ok();
                                    println!("[litquid-watch] Generated: {:?}", output_js_path);

                                    for emitter in watcher_emitters.iter() {
                                        let content = emitter.emit(template_name, &parsed);
                                        let ext = format!("template.{}", emitter.file_extension());
                                        let emit_path = watcher_output
                                            .join(relative_path)
                                            .with_extension(&ext);
                                        std::fs::write(&emit_path, content).ok();
                                        println!("[litquid-watch] Generated: {:?}", emit_path);
                                    }

                                    any_success = true;
                                }
                                Err(e) => {
                                    eprintln!("[litquid-watch] Error: {}", e);
                                }
                            }
                        }

                        // Trigger reload for all connected browsers
                        if any_success {
                            let mut clients = sse_clients_watcher.lock().unwrap();
                            clients.broadcast_reload();
                            println!("[litquid-watch] Triggered browser reload");
                        }
                    }
                }
            }
        }
    });

    // Start HTTP server with SSE endpoint
    let addr = format!("0.0.0.0:{}", args.port);
    let server = Server::http(&addr).expect("Failed to start HTTP server");
    println!("[litquid-watch] Live reload server running on port {}", args.port);
    println!("[litquid-watch] Add to your HTML: <script src=\"http://localhost:{}/livereload.js\"></script>", args.port);

    for request in server.incoming_requests() {
        match request.url() {
            "/live-reload" => {
                // SSE endpoint - keep connection open and send reload events
                let sse_clients_clone = Arc::clone(&sse_clients);
                thread::spawn(move || {
                    handle_sse_connection(request, sse_clients_clone);
                });
            }
            "/livereload.js" => {
                // Serve the live reload client script
                let script = format!(r#"(() => {{
    const es = new EventSource('http://localhost:{}/live-reload');
    es.onmessage = (e) => {{
        if (e.data === 'reload') {{
            console.log('[litquid] Reloading...');
            location.reload();
        }}
    }};
    es.onerror = () => console.log('[litquid] Live reload disconnected, retrying...');
}})();
"#, args.port);
                let response = Response::from_string(script)
                    .with_header(tiny_http::Header::from_bytes("Content-Type", "application/javascript").unwrap())
                    .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap());
                let _ = request.respond(response);
            }
            "/health" => {
                let response = Response::from_string("ok")
                    .with_header(tiny_http::Header::from_bytes("Access-Control-Allow-Origin", "*").unwrap());
                let _ = request.respond(response);
            }
            _ => {
                let response = Response::from_string("Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    }
}

fn handle_sse_connection(request: tiny_http::Request, sse_clients: Arc<Mutex<SseClients>>) {
    // Create a channel for this client
    let (tx, rx) = channel::<()>();

    // Register this client
    {
        let mut clients = sse_clients.lock().unwrap();
        clients.add(tx);
    }

    // Get the underlying writer
    let mut writer = request.into_writer();

    // Write HTTP response headers manually
    let _ = writer.write_all(b"HTTP/1.1 200 OK\r\n");
    let _ = writer.write_all(b"Content-Type: text/event-stream\r\n");
    let _ = writer.write_all(b"Cache-Control: no-cache\r\n");
    let _ = writer.write_all(b"Connection: keep-alive\r\n");
    let _ = writer.write_all(b"Access-Control-Allow-Origin: *\r\n");
    let _ = writer.write_all(b"\r\n");

    // Send initial connected message
    if writer.write_all(b"data: connected\n\n").is_err() {
        return;
    }
    if writer.flush().is_err() {
        return;
    }

    // Wait for reload signals or keepalive
    loop {
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(()) => {
                // Reload signal received
                if writer.write_all(b"data: reload\n\n").is_err() {
                    break;
                }
                if writer.flush().is_err() {
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Send keepalive comment
                if writer.write_all(b": keepalive\n\n").is_err() {
                    break;
                }
                if writer.flush().is_err() {
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }
}

fn process_all_templates(
    input: &PathBuf,
    output: &PathBuf,
    lit_import: &str,
    emitters: &[Box<dyn TargetEmitter>],
) {
    for entry in walkdir::WalkDir::new(input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "liquid"))
    {
        let input_path = entry.path();
        let relative_path = input_path.strip_prefix(input).unwrap();
        let template_name = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("template");

        let output_js_path = output.join(relative_path).with_extension("template.js");
        let output_json_path = output.join(relative_path).with_extension("template.json");

        if let Some(parent) = output_js_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        match process_liquid_file(input_path, Some(lit_import)) {
            Ok(parsed) => {
                std::fs::write(&output_js_path, parsed.to_js_module()).ok();
                std::fs::write(&output_json_path, parsed.to_json_manifest()).ok();
                println!("[litquid-watch] Generated: {:?}", output_js_path);

                for emitter in emitters {
                    let content = emitter.emit(template_name, &parsed);
                    let ext = format!("template.{}", emitter.file_extension());
                    let emit_path = output.join(relative_path).with_extension(&ext);
                    std::fs::write(&emit_path, content).ok();
                    println!("[litquid-watch] Generated: {:?}", emit_path);
                }
            }
            Err(e) => {
                eprintln!("[litquid-watch] Error processing {:?}: {}", input_path, e);
            }
        }
    }
}

