//! HTTP/1.1 mínimo sobre la conexión TLS: lee la petición, enruta y responde.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use rustls::{ServerConfig, ServerConnection};

use crate::hilo_actual;

/// Petición HTTP simplificada.
#[derive(Debug, PartialEq, Eq)]
struct Request {
    method: String,
    path: String,
    #[allow(dead_code)]
    version: String,
    body: Vec<u8>,
}

/// Respuesta HTTP simplificada.
#[derive(Debug, PartialEq, Eq)]
struct Response {
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
}

impl Response {
    fn new(status: u16, content_type: &'static str, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            content_type,
            body: body.into(),
        }
    }

    fn text(status: u16, body: impl AsRef<[u8]>) -> Self {
        Self::new(status, "text/plain", body.as_ref().to_vec())
    }

    fn html(status: u16, body: &str) -> Self {
        Self::new(status, "text/html", body.as_bytes().to_vec())
    }

    fn json(status: u16, body: &str) -> Self {
        Self::new(status, "application/json", body.as_bytes().to_vec())
    }

    /// Serializa cabeceras + cuerpo. `Content-Length` evita keep-alive:
    /// el cliente sabe exactamente cuándo termina la respuesta.
    fn as_bytes(&self) -> Vec<u8> {
        let motivo = match self.status {
            200 => "OK",
            404 => "Not Found",
            405 => "Method Not Allowed",
            _ => "Unknown",
        };
        let cabeceras = format!(
            "HTTP/1.1 {} {}\r\n\
             Content-Type: {}; charset=utf-8\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             Server: servidor-https-rust/{}\r\n\
             \r\n",
            self.status,
            motivo,
            self.content_type,
            self.body.len(),
            env!("CARGO_PKG_VERSION"),
        );
        let mut bytes = cabeceras.into_bytes();
        bytes.extend_from_slice(&self.body);
        bytes
    }
}

/// Punto de entrada de cada trabajo del pool: handshake TLS + petición + respuesta.
pub fn handle_connection(
    mut socket: TcpStream,
    config: Arc<ServerConfig>,
    threads: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let inicio = std::time::Instant::now();
    let peer = socket.peer_addr()?;
    // Evita que un cliente lento o silencioso ocupe un worker para siempre.
    socket.set_read_timeout(Some(Duration::from_secs(10)))?;
    socket.set_write_timeout(Some(Duration::from_secs(10)))?;

    println!("[{}] nueva conexión TLS desde {}", hilo_actual(), peer);

    let mut conn = ServerConnection::new(config)?;

    // `rustls::Stream` adapta la conexión TLS a los traits Read/Write de Rust,
    // así que el código HTTP de abajo es idéntico al de un TcpStream plano.
    {
        let mut tls = rustls::Stream::new(&mut conn, &mut socket);
        let request = read_request(&mut tls)?;
        let response = route(&request, threads);
        println!(
            "[{}] {} {} -> {}",
            hilo_actual(),
            request.method,
            request.path,
            response.status
        );
        tls.write_all(&response.as_bytes())?;
        tls.flush()?;
    }

    // Cerramos la sesión TLS con close_notify (un "adiós" criptográfico).
    conn.send_close_notify();
    {
        let mut tls = rustls::Stream::new(&mut conn, &mut socket);
        tls.flush()?;
    }

    println!(
        "[{}] conexión de {} cerrada en {:?}",
        hilo_actual(),
        peer,
        inicio.elapsed()
    );
    Ok(())
}

/// Lee la petición: primero las cabeceras hasta `\r\n\r\n` y luego los bytes
/// del cuerpo indicados por `Content-Length`.
fn read_request(stream: &mut impl Read) -> io::Result<Request> {
    let mut buf = Vec::with_capacity(1024);
    let mut trozo = [0_u8; 1024];
    let mut fin_cabeceras = None;

    while buf.len() < 64 * 1024 {
        if let Some(pos) = encontrar(&buf, b"\r\n\r\n") {
            fin_cabeceras = Some(pos + 4);
            break;
        }
        let n = stream.read(&mut trozo)?;
        if n == 0 {
            break; // cliente cerró antes de terminar las cabeceras
        }
        buf.extend_from_slice(&trozo[..n]);
    }

    let fin = fin_cabeceras.unwrap_or(buf.len());
    let cabeceras = String::from_utf8_lossy(&buf[..fin]);
    let mut lineas = cabeceras.split("\r\n");

    let mut partes = lineas.next().unwrap_or("").split_whitespace();
    let method = partes.next().unwrap_or("").to_string();
    let path = partes.next().unwrap_or("/").to_string();
    let version = partes.next().unwrap_or("HTTP/1.1").to_string();

    let mut content_length = 0_usize;
    for linea in lineas {
        if let Some((clave, valor)) = linea.split_once(':') {
            if clave.eq_ignore_ascii_case("content-length") {
                content_length = valor.trim().parse().unwrap_or(0);
            }
        }
    }

    while buf.len() < fin + content_length {
        let n = stream.read(&mut trozo)?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&trozo[..n]);
    }
    let body = if buf.len() > fin {
        buf[fin..fin + content_length.min(buf.len() - fin)].to_vec()
    } else {
        Vec::new()
    };

    Ok(Request {
        method,
        path,
        version,
        body,
    })
}

/// Busca un patrón byte a byte (equivalente a `windows().position()`).
fn encontrar(buf: &[u8], patron: &[u8]) -> Option<usize> {
    if patron.is_empty() || buf.len() < patron.len() {
        return None;
    }
    buf.windows(patron.len()).position(|w| w == patron)
}

/// Enrutador: decide qué responder según método y ruta.
fn route(req: &Request, threads: usize) -> Response {
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => Response::html(200, PORTADA),
        ("GET", "/health") => Response::json(200, "{\"status\":\"ok\"}"),
        ("GET", "/info") => Response::json(
            200,
            &format!(
                "{{\"version\":\"{}\",\"hilos\":{},\"worker\":\"{}\"}}",
                env!("CARGO_PKG_VERSION"),
                threads,
                hilo_actual()
            ),
        ),
        ("POST", "/echo") => Response::text(200, &req.body),
        ("GET", "/echo") => Response::new(405, "text/plain", "Usa POST con un cuerpo"),
        _ => Response::html(404, "<h1>404</h1><p>Ruta no encontrada</p>"),
    }
}

const PORTADA: &str = r#"<!doctype html>
<html lang="es">
<head><meta charset="utf-8"><title>Servidor HTTPS en Rust</title></head>
<body>
  <h1>servidor-https-rust</h1>
  <p>HTTPS con rustls + pool de hilos. Elaborado por Francisco Sevilla y Yeimar Gonzalez.</p>
  <ul>
    <li><a href="/health">/health</a></li>
    <li><a href="/info">/info</a></li>
  </ul>
</body>
</html>"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn encuentra_el_fin_de_cabeceras() {
        assert_eq!(
            encontrar(b"GET / HTTP/1.1\r\n\r\nbody", b"\r\n\r\n"),
            Some(14)
        );
        assert_eq!(encontrar(b"sin fin", b"\r\n\r\n"), None);
    }

    #[test]
    fn parsea_get_simple() {
        let raw = b"GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = read_request(&mut Cursor::new(raw.to_vec())).unwrap();
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/health");
        assert_eq!(req.version, "HTTP/1.1");
        assert!(req.body.is_empty());
    }

    #[test]
    fn parsea_post_con_body() {
        let raw = b"POST /echo HTTP/1.1\r\nContent-Length: 5\r\n\r\nhola!";
        let req = read_request(&mut Cursor::new(raw.to_vec())).unwrap();
        assert_eq!(req.method, "POST");
        assert_eq!(req.body, b"hola!");
    }

    #[test]
    fn enruta_y_serializa() {
        let req = Request {
            method: "GET".into(),
            path: "/health".into(),
            version: "HTTP/1.1".into(),
            body: Vec::new(),
        };
        let res = route(&req, 4);
        assert_eq!(res.status, 200);
        let bytes = res.as_bytes();
        let texto = String::from_utf8(bytes).unwrap();
        assert!(texto.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(texto.contains("Content-Length: 15\r\n"));
    }

    #[test]
    fn desconocido_da_404() {
        let req = Request {
            method: "GET".into(),
            path: "/nope".into(),
            version: "HTTP/1.1".into(),
            body: Vec::new(),
        };
        assert_eq!(route(&req, 4).status, 404);
    }
}
