mod data;
mod graph_builder;

use std::fs::OpenOptions;
use std::net::SocketAddr;

use http_body_util::{BodyExt, Empty, Full};
use http_body_util::combinators::BoxBody;
use hyper::{header, Method, Request, Response, StatusCode};
use hyper::body::{Buf, Bytes};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
//use serde::Serialize;
use tokio::net::TcpListener;
use serde_json;
use data::{Datos, DatosConFecha};
use chrono::{self, DateTime, Local, NaiveDate, TimeZone, Utc};

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

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("health_data.csv")
        .unwrap();
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    received_data.timestamp = Some(chrono::offset::Utc::now().timestamp());
    wtr.serialize(received_data).unwrap();
    wtr.flush().unwrap();

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


    basic_graph_builder(vec![0.0,1.0,2.0]).save("test.png").unwrap();

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

