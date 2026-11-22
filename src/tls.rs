use std::fs;
use std::io::BufReader;
use std::path::Path;

use rcgen::{CertificateParams, DnType, KeyPair};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;

pub fn build_server_config(
    cert_path: Option<&Path>,
    key_path: Option<&Path>,
) -> Result<ServerConfig, Box<dyn std::error::Error>> {
    let (cert, key) = match (cert_path, key_path) {
        (Some(c), Some(k)) => load_pem(c, k)?,
        (None, None) => generate_and_save_self_signed()?,
        _ => unreachable!("Args valida que --cert y --key vayan juntos"),
    };

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)?;

    Ok(config)
}

fn load_pem(
    cert_path: &Path,
    key_path: &Path,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), Box<dyn std::error::Error>> {
    let mut cert_file = BufReader::new(fs::File::open(cert_path)?);
    let certs: Vec<CertificateDer<'static>> =
        rustls_pemfile::certs(&mut cert_file).collect::<Result<Vec<_>, _>>()?;

    let mut key_file = BufReader::new(fs::File::open(key_path)?);
    let key = rustls_pemfile::private_key(&mut key_file)?
        .ok_or("no se encontró una clave privada en el archivo")?;

    println!("Certificado cargado desde {}", cert_path.display());
    Ok((certs, key))
}

fn generate_and_save_self_signed(
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), Box<dyn std::error::Error>> {
    let mut params = CertificateParams::new(vec!["localhost".to_string()])?;
    params
        .distinguished_name
        .push(DnType::CommonName, "localhost");

    let clave = KeyPair::generate()?;
    let certificado = params.self_signed(&clave)?;

    let cert_der = certificado.der().clone();
    let key_der = PrivateKeyDer::Pkcs8(clave.serialize_der().into());

    fs::create_dir_all("certs")?;
    fs::write("certs/cert.pem", certificado.pem())?;
    fs::write("certs/key.pem", clave.serialize_pem())?;
    println!("Certificado autofirmado generado en certs/cert.pem y certs/key.pem");
    println!("Para confiar en él: curl --cacert certs/cert.pem https://localhost:8443/health");

    Ok((vec![cert_der], key_der))
}
