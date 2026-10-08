use std::{env, fs::File, io, path::Path};
use tiny_http::{Header, Method, Request, Response, Server};

const WEB_ROOT: &str = "dist";
const ASSETS: &[(&str, &str)] = &[
    ("index.html", "text/html; charset=utf-8"),
    ("styles.css", "text/css; charset=utf-8"),
    ("app.js", "text/javascript; charset=utf-8"),
    ("mq_js_bundle.js", "text/javascript; charset=utf-8"),
    ("elite.wasm", "application/wasm"),
    ("favicon.svg", "image/svg+xml"),
    ("THIRD_PARTY.md", "text/markdown; charset=utf-8"),
    ("licenses/LICENSE-MIT", "text/plain; charset=utf-8"),
    ("licenses/LICENSE-APACHE", "text/plain; charset=utf-8"),
];

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root = Path::new(WEB_ROOT);
    for (name, _) in ASSETS {
        let path = root.join(name);
        let file = File::open(&path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!(
                    "{}: {error}. Run scripts/build-web.sh before starting the server.",
                    path.display()
                ),
            )
        })?;
        if !file.metadata()?.is_file() {
            return Err(format!("{} is not a file", path.display()).into());
        }
    }

    let address = env::var("WIRESPACE_ADDR").unwrap_or_else(|_| "127.0.0.1:3141".into());
    let server = Server::http(&address)?;
    println!(
        "Wirespace listening on http://{} (serving {})",
        server.server_addr(),
        root.canonicalize()?.display()
    );
    loop {
        if let Err(error) = respond(server.recv()?, root) {
            eprintln!("Could not send response: {error}");
        }
    }
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).expect("valid static HTTP header")
}

fn respond(request: Request, root: &Path) -> io::Result<()> {
    if !matches!(request.method(), Method::Get | Method::Head) {
        return request.respond(
            Response::from_string("Method not allowed\n")
                .with_status_code(405)
                .with_header(header("Allow", "GET, HEAD")),
        );
    }

    let path = request.url().split('?').next().unwrap_or("/");
    let name = if path == "/" {
        "index.html"
    } else {
        path.strip_prefix('/').unwrap_or("")
    };
    // URL text never becomes a filesystem path unless it matches a public asset.
    let Some(&(asset, content_type)) = ASSETS.iter().find(|(asset, _)| *asset == name) else {
        return request.respond(Response::from_string("Not found\n").with_status_code(404));
    };
    let file = match File::open(root.join(asset)) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("Could not read {asset}: {error}");
            return request
                .respond(Response::from_string("Game asset unavailable\n").with_status_code(503));
        }
    };
    request.respond(
        Response::from_file(file)
            .with_header(header("Content-Type", content_type))
            .with_header(header("Cache-Control", "no-cache"))
            .with_header(header("X-Content-Type-Options", "nosniff")),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        io::{Read, Write},
        net::TcpStream,
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
        thread,
        time::Duration,
    };

    struct WebFiles(PathBuf);

    impl WebFiles {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = env::temp_dir().join(format!(
                "wirespace-server-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            fs::write(path.join("index.html"), b"<title>Wirespace</title>").unwrap();
            fs::write(path.join("elite.wasm"), b"\0asm\x01\0\0\0").unwrap();
            fs::write(path.join("private.txt"), b"must not be served").unwrap();
            Self(path)
        }

        fn request(&self, method: &str, path: &str) -> (String, Vec<u8>) {
            let server = Server::http("127.0.0.1:0").unwrap();
            let address = server.server_addr().to_ip().unwrap();
            let root = self.0.clone();
            let worker = thread::spawn(move || {
                let request = server
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap()
                    .expect("request within timeout");
                respond(request, &root).unwrap();
            });
            let mut stream = TcpStream::connect(address).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            write!(
                stream,
                "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
            )
            .unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
            let mut bytes = Vec::new();
            stream.read_to_end(&mut bytes).unwrap();
            worker.join().unwrap();
            let split = bytes
                .windows(4)
                .position(|part| part == b"\r\n\r\n")
                .unwrap();
            (
                String::from_utf8(bytes[..split].to_vec()).unwrap(),
                bytes[split + 4..].to_vec(),
            )
        }
    }

    impl Drop for WebFiles {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn serves_browser_assets_with_correct_mime_and_head() {
        let files = WebFiles::new();
        let (headers, body) = files.request("GET", "/?version=1");
        assert!(headers.starts_with("HTTP/1.1 200"));
        assert!(headers.contains("Content-Type: text/html; charset=utf-8"));
        assert_eq!(body, b"<title>Wirespace</title>");

        let (headers, body) = files.request("GET", "/elite.wasm?version=1");
        assert!(headers.contains("Content-Type: application/wasm"));
        assert_eq!(body, b"\0asm\x01\0\0\0");

        let (headers, body) = files.request("HEAD", "/elite.wasm");
        assert!(headers.contains("Content-Type: application/wasm"));
        assert!(headers.contains("Content-Length: 8"));
        assert!(body.is_empty());
    }

    #[test]
    fn serves_only_read_only_public_assets() {
        let files = WebFiles::new();
        for path in ["/private.txt", "/../private.txt", "/%2e%2e/private.txt"] {
            let (headers, _) = files.request("GET", path);
            assert!(headers.starts_with("HTTP/1.1 404"), "{path}: {headers}");
        }
        let (headers, _) = files.request("POST", "/");
        assert!(headers.starts_with("HTTP/1.1 405"));
        assert!(headers.contains("Allow: GET, HEAD"));

        fs::remove_file(files.0.join("elite.wasm")).unwrap();
        let (headers, _) = files.request("GET", "/elite.wasm");
        assert!(headers.starts_with("HTTP/1.1 503"));
    }
}
