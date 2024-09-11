mod data;
mod graph_builder;

use std::fs::OpenOptions;
use std::net::SocketAddr;
use std::str;

use http_body_util::{BodyExt, Full, StreamBody};
use http_body_util::combinators::BoxBody;
use hyper::{Method, Request, Response, StatusCode};
use hyper::body::{Bytes, Frame};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use futures_util::TryStreamExt;

use data::{Datos, DatoConFecha};
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

async fn echo(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, std::io::Error>>, hyper::Error> {
    println!("Request made: {:#?}", (req.method(), req.uri().path()));
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/update") => {
            save_health_data(req).await
        }

        (&Method::GET, "/") => {
            simple_file_send("html/index.html").await
        }

        (&Method::GET, "/scale_data") => {
            read_health_data().await
        }

        (&Method::GET, "/new_data_form") => {
            simple_file_send("html/templates/new_data_form.html").await
        }

        (&Method::GET, path) if path.starts_with("/graphs/") => simple_file_send(remove_first(path).expect("No pude eliminar el primer caracter")).await,
        (&Method::GET, path) if path.starts_with("/styles/") => simple_file_send(remove_first(path).expect("No pude eliminar el primer caracter")).await,

        // Return 404 Not Found for other routes.
        _ => {
            Ok(not_found())
        }
    }
}
async fn save_health_data(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, std::io::Error>>, hyper::Error> {
    let received_data = req
        .into_body()
        .collect()
        .await?
        .to_bytes();
    let mut received_data: Datos = Datos::deserialize(
        std::string::String::from_utf8(
            received_data.to_vec()
            ).expect("Datos recibidos corruptos\n") + "&");
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

    read_health_data().await
}


async fn read_health_data() -> Result<Response<BoxBody<Bytes, std::io::Error>>, hyper::Error> {
    let mut rdr = csv::Reader::from_path("health_data.csv").expect("Archivo no encontrado");

    let mut health_data: Vec<Datos> = vec![];

    for result in rdr.deserialize() {
        let datos: Datos = result.unwrap();

        if datos.is_recent()
        {
            health_data.push(datos);
        }
    }
    make_graphs(health_data);

    //let content = read_to_string("html/index.html").await.unwrap_or_else(|_| "File not found".to_string());
    
    simple_file_send("html/templates/health_data.html").await
}

async fn simple_file_send(filename: &str) ->Result<Response<BoxBody<Bytes, std::io::Error>>, hyper::Error> {
    // Open file for reading
    let file = File::open(filename).await;
    if file.is_err() {
        eprintln!("ERROR: Unable to open file.");
        return Ok(not_found());
    }

    let file: File = file.unwrap();

    // Wrap to a tokio_util::io::ReaderStream
    let reader_stream = ReaderStream::new(file);

    // Convert to http_body_util::BoxBody
    let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
    let boxed_body = stream_body.boxed();

    // Send response
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(boxed_body)
        .unwrap();

    Ok(response)
}

//async fn serve_image(path: &str) -> Result<Response<BoxBody<Bytes, std::io::Error>>, hyper::Error>
//{
//
//}

fn make_graphs(health_data: Vec<Datos>)
{
    let mut peso: Vec<DatoConFecha> = vec![];
    let mut grasa_visceral: Vec<DatoConFecha> = vec![];
    let mut grasa_corporal: Vec<DatoConFecha> = vec![];
    let mut musculo: Vec<DatoConFecha> = vec![];
    let mut agua: Vec<DatoConFecha> = vec![];
    let mut proteina: Vec<DatoConFecha> = vec![];
    let mut metabolismo_basal: Vec<DatoConFecha> = vec![];
    let mut masa_osea: Vec<DatoConFecha> = vec![];
    let mut diametro_cintura: Vec<DatoConFecha> = vec![];
    for dato in health_data
    {
        let timestamp;
        match dato.timestamp
        {
            Some(time) => timestamp = time,
            None => continue,
        }
        if let Some(dato) = dato.peso{
            peso.push(
                DatoConFecha{
                    timestamp,
                    dato,
                }
            );
        }
        if let Some(dato) = dato.grasa_visceral{
            grasa_visceral.push(
                DatoConFecha{
                    timestamp,
                    dato
                }
                );
        }

        if let Some(dato) = dato.grasa_corporal{
            grasa_corporal.push(
                DatoConFecha{
                    timestamp,
                    dato
                }
                );
        }

        if let Some(dato) = dato.musculo{
            musculo.push(
                DatoConFecha{
                    timestamp,
                    dato
                }
                );
        }

        if let Some(dato) = dato.agua{
            agua.push(
                DatoConFecha{
                    timestamp,
                    dato
                }
                );
        }

        if let Some(dato) = dato.proteina{
            proteina.push(
                DatoConFecha{
                    timestamp,
                    dato
                }
                );
        }

        if let Some(dato) = dato.metabolismo_basal{
            metabolismo_basal.push(
                DatoConFecha{
                    timestamp,
                    dato
                }
                );
        }

        if let Some(dato) = dato.masa_osea{
            masa_osea.push(
                DatoConFecha{
                    timestamp,
                    dato
                }
                );
        }

        if let Some(dato) = dato.diametro_cintura{
            diametro_cintura.push(
                DatoConFecha{
                    timestamp,
                    dato
                }
                );
        }

    }
    basic_graph_builder(peso).save("graphs/peso.png").unwrap();
    basic_graph_builder(grasa_visceral).save("graphs/grasa_visceral.png").unwrap();
    basic_graph_builder(grasa_corporal).save("graphs/grasa_corporal.png").unwrap();
    basic_graph_builder(musculo).save("graphs/musculo.png").unwrap();
    basic_graph_builder(agua).save("graphs/agua.png").unwrap();
    basic_graph_builder(proteina).save("graphs/proteina.png").unwrap();
    basic_graph_builder(metabolismo_basal).save("graphs/metabolismo_basal.png").unwrap();
    basic_graph_builder(masa_osea).save("graphs/masa_osea.png").unwrap();
    basic_graph_builder(diametro_cintura).save("graphs/diametro_cintura.png").unwrap();
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
//fn empty() -> BoxBody<Bytes, hyper::Error> {
//    Empty::<Bytes>::new()
//        .map_err(|never| match never {})
//        .boxed()
//}
//
//fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
//    Full::new(chunk.into())
//        .map_err(|never| match never {})
//        .boxed()
//}

//fn dummy_ok() -> Result<Response<BoxBody<Bytes, std::io::Error>>, hyper::Error> {
//    Ok(
//        Response::builder()
//        .status(StatusCode::OK)
//        .body(Full::new(String::from("").into()).map_err(|e| match e {}).boxed())
//        .unwrap()
//      )
//}

fn not_found() ->Response<BoxBody<Bytes, std::io::Error>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(String::from("Not Found").into()).map_err(|e| match e {}).boxed())
        .unwrap()
}

fn remove_first(s: &str) -> Option<&str> {
    s.chars().next().map(|c| &s[c.len_utf8()..])
}
