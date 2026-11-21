# servidor-https-rust

Servidor **HTTPS multihilo** escrito en Rust usando un **pool de hilos fijo** y **rustls** (TLS implementado en Rust, sin OpenSSL). Incluye además documentación de investigación y un guion para grabar el video explicativo.

**Entrega:** domingo 22 de noviembre de 2026  
**Integrantes (grupo máx. 2):** [Nombre 1] · [Nombre 2]

---

## Contenido del repositorio

| Ruta | Descripción |
|---|---|
| `src/main.rs` | Punto de entrada: argumentos, configuración TLS, accept loop y envío de trabajos al pool |
| `src/pool.rs` | Pool de hilos con cola `mpsc` + `Arc<Mutex<Receiver>>` |
| `src/tls.rs` | Carga de PEM / generación de certificado autofirmado con `rcgen` |
| `src/http.rs` | Lectura de peticiones HTTP/1.1, enrutado y respuestas |
| `documentacion/investigacion.md` | Marco teórico: TLS, hilos, rustls, HTTP y seguridad |
| `documentacion/guion_video.md` | Guion cronometrado para explicar el código en video |
| `subir_github.ps1` | Script auxiliar para crear el repo y hacer push con GitHub CLI |

---

## Requisitos

- Rust **1.98+** (`rustc --version`, `cargo --version`)
- Compilador de Microsoft Visual Studio (ya instalado en el equipo; lo usa `ring`)
- `curl` para las pruebas (incluido en Windows 10/11)

> No hace falta OpenSSL ni instalar certificados: el programa genera un certificado autofirmado la primera vez que arranca.

---

## Ejecutar

```powershell
# Compilar y arrancar con 4 hilos
cargo run --release -- --threads 4

# Escuchar en otra dirección
cargo run --release -- --addr 127.0.0.1:9443 --threads 2

# Usar certificado propio en vez del autofirmado
cargo run --release -- --cert certs/cert.pem --key certs/key.pem

# Ver opciones
cargo run -- --help
```

Al arrancar se crean `certs/cert.pem` y `certs/key.pem` (la clave privada **no** se sube a GitHub; está en `.gitignore`).

---

## Probar

El certificado es autofirmado, por eso `curl` necesita `-k` (o `--cacert` para confiar en él explícitamente):

```powershell
curl -k https://localhost:8443/
curl -k https://localhost:8443/health
curl -k https://localhost:8443/info
curl -k -X POST -d "hola desde rust" https://localhost:8443/echo
curl --cacert certs/cert.pem https://localhost:8443/health
```

| Método | Ruta | Respuesta |
|---|---|---|
| `GET` | `/` | Página HTML de presentación |
| `GET` | `/health` | `{"status":"ok"}` |
| `GET` | `/info` | Versión, número de hilos y worker que atendió |
| `POST` | `/echo` | Devuelve el cuerpo recibido |
| `GET` | `/echo` | `405 Method Not Allowed` |
| cualquiera | otra | `404 Not Found` |

En la consola verás mensajes como:

```
[worker-2] ejecutando trabajo
[worker-2] GET /health -> 200
[worker-2] conexión de 127.0.0.1:52134 cerrada en 1.2ms
```

---

## Pruebas automatizadas

```powershell
cargo test
cargo clippy --all-targets
cargo fmt --check
```

Los tests cubren el parser de peticiones (GET, POST con `Content-Length`), el enrutador, la serialización de respuestas y que el pool ejecuta todos los trabajos.

---

## Arquitectura (resumen)

```
   TcpListener  ──accept──>  TcpStream
        │
        ▼
   ThreadPool { sender: mpsc::Sender<Job>, workers: Vec<Worker> }
        │  cada worker: receiver.lock().recv() → job()
        ▼
   rustls::ServerConnection  ──TLS──>  HTTP/1.1 (read/route/write)
```

* Un solo hilo acepta conexiones y las mete en la cola; nunca bloquea haciendo TLS.
* Los `N` workers son reales hilos del SO (`std::thread`), no tareas async.
* `Arc<ServerConfig>` permite compartir la configuración TLS sin copiarla.
* `Connection: close` + `Content-Length` simplifican el ciclo petición/respuesta.

---

## Seguridad

* El certificado autofirmado sirve **solo para desarrollo/demo**. En producción: certificado de una CA (Let's Encrypt) y renovación automática.
* `rustls` usa `ring` (auditado) y no carga código C de OpenSSL.
* Timeout de lectura/escritura de 10 s por conexión para mitigar clientes lentos.
* Límite de 64 KiB de cabeceras para evitar peticiones gigantes.

---

## Autores

[Nombre 1] · [Nombre 2]  
Trabajo grupal — máximo 2 integrantes.
