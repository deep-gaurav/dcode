use lazy_static::lazy_static;

use hyper::body::Body;
use hyper::{Request, Response};
use warp::hyper;

use regex::Regex;

lazy_static! {
    static ref port_reg: Regex = Regex::new(r"/portforward/(\d+)(/.*)").unwrap();
}

pub fn is_portforward(url:&str)->bool{
    println!("testing port for {:?}",url );
    let r=port_reg.find(url).is_some();
    println!("result {:?}", r);
    r
}

pub async fn port_forward(mut req: Request<Body>) -> Result<Response<Body>, std::convert::Infallible> {
    let http_client = hyper::Client::new();
    let url = &req.uri().to_string();
    let mat = port_reg.captures(url);
    let port;
    let path;
    match mat {
        Some(mat)=>{
            port=format!("{}",&mat[1]);
            path=format!("{}",&mat[2]);
        }
        None =>{
            let refrer = req.headers()["referer"].to_str().expect("refere not string");
            let mat = port_reg.captures(refrer).expect("Not even matching referer");
            port = format!("{}",&mat[1]);
            path = format!("{}",url);
        }

    }

    // let body_bytes = hyper::body::to_bytes(req.body());
    // let reqbody = Body::wrap_stream(req.body());
    // let mut new_req = Request::builder()
    // .uri(req.uri())
    // .method(req.method())
    // .body(reqbody)
    // .unwrap()
    // ;
    // req.headers_mut().insert("host", "localhost:9000".parse().expect("cant make header"));
    *req.uri_mut() = format!("http://localhost:{}{}",port,path).parse().expect("cant convert to uri");

    // *req.uri_mut() = (req.uri().to_string()+"index.html").parse().unwrap();

    println!("Client req {:?}", req );
    let response = http_client.request(req).await.expect("client error");

    Ok(response)

}
