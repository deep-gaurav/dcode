use lazy_static::lazy_static;

use hyper::body::Body;
use hyper::{Request, Response};
use warp::hyper;

use regex::Regex;

lazy_static! {
    static ref port_reg: Regex = Regex::new(r"^/portforward/(\d+)(/.*)").unwrap();
}

pub fn is_portforward(req:&hyper::Uri)->bool{
    let url = &req.to_string();
    port_reg.find(url).is_some()
}

pub async fn port_forward(mut req: Request<Body>) -> Result<Response<Body>, std::convert::Infallible> {
    let http_client = hyper::Client::new();
    let url = &req.uri().to_string();
    let mat = port_reg.captures(url).expect("forward url not found");

    // let body_bytes = hyper::body::to_bytes(req.body());
    // let reqbody = Body::wrap_stream(req.body());
    // let mut new_req = Request::builder()
    // .uri(req.uri())
    // .method(req.method())
    // .body(reqbody)
    // .unwrap()
    // ;
    req.headers_mut().insert("host", "localhost:9000".parse().expect("cant make header"));
    *req.uri_mut() = format!("http://localhost:{}{}",&mat[1],&mat[2]).parse().expect("cant convert to uri");

    // *req.uri_mut() = (req.uri().to_string()+"index.html").parse().unwrap();

    println!("Client req {:?}", req );
    let response = http_client.request(req).await.expect("client error");

    Ok(response)

}
