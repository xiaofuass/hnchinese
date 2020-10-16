extern crate rss;
use curl::easy::Easy;
use hyper::header::{HeaderName, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use rss::Channel;
use serde_json::{Result, Value};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::str;
use std::time::Duration;
use simple_logger::SimpleLogger;

fn translate(data: &str) -> String {
    let url = format!( "https://translation.googleapis.com/language/translate/v2?target=zh-cn&key=xxxxxxxx&q={}",data);
    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(url.as_str()).unwrap();
    handle.timeout(Duration::from_secs(3)).unwrap();
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        match transfer.perform() {
            Ok(res) => {
                res
            }
            Err(e) => {
                log::error!("Error new {}",e);
                transfer.perform().unwrap();
            }
        };
    }

    let s = str::from_utf8(&data).unwrap();
    let json_data: Value = serde_json::from_str(s).unwrap();
    json_data["data"]["translations"][0]["translatedText"].to_string()
}

fn get_xml_response_element() -> String {
    let mut channel = match Channel::from_url("https://hnrss.org/newest") {
        Ok(ok) => {
            ok
        }
        Err(e) => {
            log::error!("Error new {}",e);
            Channel::from_url("https://hnrss.org/newest").unwrap()
        }
    };
    for item in channel.items_mut() {
        match item.title() {
            Some(s) => {
                let mut translate_data = String::new();
                translate_data.push_str(translate(s.replace(" ", "%20").as_str()).as_str());
                let format_title = format!("{} | {:?}", translate_data, s);
                item.set_title(format_title);
            }
            None => {
                log::error!("No title found");
            }
        }
    }
    channel.to_string()
}

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>> {
    let mut res = Response::new(get_xml_response_element().into());
    res.headers_mut().insert(
        HeaderName::from_lowercase(b"content-type").unwrap(),
        HeaderValue::from_str("text/xml;charset=UTF-8").unwrap(),
    );

    Ok(res)
}
#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    log::info!("Server is running");
    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    log::info!("Listening on http://{}", addr);
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        log::error!("server error: {}", e)
    }
}
