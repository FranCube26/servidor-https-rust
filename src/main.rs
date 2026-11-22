mod http;
mod pool;
mod tls;

use std::env;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use rustls::ServerConfig;

struct Args {
    addr: String,
    threads: usize,
    cert: Option<PathBuf>,
    key: Option<PathBuf>,
}

impl Args {
    fn parse() -> Result<Self, String> {
        let mut addr = "127.0.0.1:8443".to_string();
        let mut threads = num_cpus::get();
        let mut cert = None;
        let mut key = None;

        let mut iter = env::args().skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!(
                        "Uso: servidor-https-rust [opciones]\n\
                         \x20   --addr <IP:PUERTO>   Dirección de escucha (por defecto 127.0.0.1:8443)\n\
                         \x20   --threads <N>       Número de hilos (por defecto: número de CPUs)\n\
                         \x20   --cert <ruta>       Certificado PEM (si no se da, se genera uno autofirmado)\n\
                         \x20   --key <ruta>        Clave privada PEM\n\
                         \x20   -h, --help          Mostrar esta ayuda"
                    );
                    std::process::exit(0);
                }
                "--addr" => addr = iter.next().ok_or("--addr necesita un valor")?,
                "--threads" => {
                    let raw = iter.next().ok_or("--threads necesita un valor")?;
                    threads = raw.parse().map_err(|_| "--threads debe ser un número")?;
                }
                "--cert" => {
                    cert = Some(PathBuf::from(
                        iter.next().ok_or("--cert necesita un valor")?,
                    ))
                }
                "--key" => key = Some(PathBuf::from(iter.next().ok_or("--key necesita un valor")?)),
                other => return Err(format!("opción desconocida: {other}")),
            }
        }

        if threads == 0 {
            return Err("--threads debe ser al menos 1".to_string());
        }
        if cert.is_some() != key.is_some() {
            return Err("debes pasar --cert y --key juntos, o ninguno de los dos".to_string());
        }

        Ok(Self {
            addr,
            threads,
            cert,
            key,
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| "no se pudo instalar el proveedor crypto de ring")?;

    let args = Args::parse()?;

    let config: Arc<ServerConfig> = Arc::new(tls::build_server_config(
        args.cert.as_deref(),
        args.key.as_deref(),
    )?);

    let pool = pool::ThreadPool::new(args.threads);
    let listener = TcpListener::bind(&args.addr)?;

    println!(
        "Servidor HTTPS escuchando en https://{}",
        listener.local_addr()?
    );
    println!("Hilos workers: {}", pool.size());
    println!("Rutas: GET / | GET /health | GET /info | POST /echo");
    println!("Prueba: curl -k https://{}/health", args.addr);

    let inicio = Instant::now();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let config = Arc::clone(&config);
                let threads = args.threads;
                pool.execute(move || {
                    if let Err(e) = http::handle_connection(stream, config, threads) {
                        eprintln!("[{}] error en conexión: {e}", hilo_actual());
                    }
                });
            }
            Err(e) => eprintln!("Error al aceptar conexión: {e}"),
        }
    }

    println!("Servidor apagado después de {:?}", inicio.elapsed());
    Ok(())
}

pub fn hilo_actual() -> String {
    std::thread::current()
        .name()
        .unwrap_or("desconocido")
        .to_string()
}
