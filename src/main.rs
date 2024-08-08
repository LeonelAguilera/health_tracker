mod data;
mod graph_builder;

use std::fs::OpenOptions;
use std::net::SocketAddr;

use http_body_util::{BodyExt, Empty};
use http_body_util::combinators::BoxBody;
use hyper::{Method, Request, Response, StatusCode};
use hyper::body::{Buf, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
//use serde::Serialize;
use tokio::net::TcpListener;
use serde_json;
use data::{Datos, DatosConFecha};
use chrono;

use graph_builder::basic_graph_builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(
            async move
            {
                // Finally, we bind the incoming connection to our `hello` service
                if let Err(err) = http1::Builder::new()
                    // `service_fn` converts our function in a `Service`
                    .serve_connection(io, service_fn(echo))
                        .await
                        {
                            eprintln!("Error serving connection: {:?}", err);
                        }
            }
        );
    }
}

async fn echo(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/scale_data") => {
            save_health_data(req).await
        }

        (&Method::GET, "/scale_data") => {
            read_health_data(req).await
        }

        // Return 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
async fn save_health_data(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let received_data = req.into_body().collect().await?.aggregate();
    let mut received_data: Datos = serde_json::from_reader(received_data.reader()).expect("Could not read JSON");
    received_data.timestamp = Some(chrono::offset::Local::now().timestamp());

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("health_data.csv")
        .unwrap();
    let mut file = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    file.serialize(received_data).unwrap();
    file.flush().unwrap();

    dummy_ok()
}

async fn read_health_data(_req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let mut rdr = csv::Reader::from_path("health_data.csv").expect("Archivo no encontrado");

    let mut health_data: Vec<Datos> = vec![];

    for result in rdr.deserialize() {
        let datos: Datos = result.unwrap();

        if datos.is_recent()
        {
            health_data.push(datos);
        }
    }
    
    let mut peso: Vec<DatosConFecha> = vec![];
    let mut grasa_visceral: Vec<DatosConFecha> = vec![];
    let mut grasa_corporal: Vec<DatosConFecha> = vec![];
    let mut musculo: Vec<DatosConFecha> = vec![];
    let mut agua: Vec<DatosConFecha> = vec![];
    let mut proteina: Vec<DatosConFecha> = vec![];
    let mut metabolismo_basal: Vec<DatosConFecha> = vec![];
    let mut masa_osea: Vec<DatosConFecha> = vec![];
    let mut diametro_cintura: Vec<DatosConFecha> = vec![];
    for dato in health_data
    {
        if let Some(peso_s) = dato.peso
        {
            peso.push(DatosConFecha{
                timestamp: dato.timestamp.unwrap(),
                datos: peso_s,
            }
            );
        }
        if let Some(grasa_visceral_s) = dato.grasa_visceral
        {
            grasa_visceral.push(DatosConFecha{
                timestamp: dato.timestamp.unwrap(),
                datos: grasa_visceral_s,
            }
            );
        }
        if let Some(grasa_corporal_s) = dato.grasa_corporal
        {
            grasa_corporal.push(DatosConFecha{
                timestamp: dato.timestamp.unwrap(),
                datos: grasa_corporal_s,
            }
            );
        }
        if let Some(musculo_s) = dato.musculo
        {
            musculo.push(DatosConFecha{
                timestamp: dato.timestamp.unwrap(),
                datos: musculo_s,
            }
            );
        }
        if let Some(agua_s) = dato.agua
        {
            agua.push(DatosConFecha{
                timestamp: dato.timestamp.unwrap(),
                datos: agua_s,
            }
            );
        }
        if let Some(proteina_s) = dato.proteina
        {
            proteina.push(DatosConFecha{
                timestamp: dato.timestamp.unwrap(),
                datos: proteina_s,
            }
            );
        }
        if let Some(metabolismo_basal_s) = dato.metabolismo_basal
        {
            metabolismo_basal.push(DatosConFecha{
                timestamp: dato.timestamp.unwrap(),
                datos: metabolismo_basal_s,
            }
            );
        }
        if let Some(masa_osea_s) = dato.masa_osea
        {
            masa_osea.push(DatosConFecha{
                timestamp: dato.timestamp.unwrap(),
                datos: masa_osea_s,
            }
            );
        }
        if let Some(diametro_cintura_s) = dato.diametro_cintura
        {
            diametro_cintura.push(DatosConFecha{
                timestamp: dato.timestamp.unwrap(),
                datos: diametro_cintura_s,
            }
            );
        }
    }


    basic_graph_builder(peso).save("test.png").unwrap();

    dummy_ok()
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}

//fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
//    Full::new(chunk.into())
//        .map_err(|never| match never {})
//        .boxed()
//}

fn dummy_ok() -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    let mut done = Response::new(empty());
    *done.status_mut() = StatusCode::OK;
    Ok(done)
}

