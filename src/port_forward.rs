use lazy_static::lazy_static;

use hyper::body::Body;
use hyper::{Request, Response};
use warp::hyper;

use regex::Regex;

lazy_static! {
    static ref port_reg: Regex = Regex::new(r"(.*)/portforward/(\d+)(/.*)?").unwrap();
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
    let reff;
    let port;
    let path;
    match mat {
        Some(mat)=>{
            port=format!("{}",&mat[2]);
            path=format!("{}",&mat[3]);

            if let Some(reffr)= req.headers().get("referer"){

                let refrer = reffr.to_str().expect("refere not string");
                let mat = port_reg.captures(refrer).expect("Not even matching referer");
                reff = format!("{}",&mat[1]);
            }
            else{
                reff = "http://localhost".to_owned();
            }
        }
        None =>{
            let refrer = req.headers()["referer"].to_str().expect("refere not string");
            let mat = port_reg.captures(refrer).expect("Not even matching referer");
            port = format!("{}",&mat[2]);
            reff = format!("{}",&mat[1]);
            path = format!("{}",url);

            if req.method() == hyper::Method::GET
            {
                let resp = Response::builder()
                    .status(301)
                    .header("location", format!("/portforward/{}{}",port,path))
                    .body(Body::empty())
                    ;
                return Ok(resp.expect("Cant create Response"));
            }
        }

    }

    req.headers_mut().insert("host", format!("localhost:{}",port).parse().expect("cant make value"));
    req.headers_mut().insert("referer", format!("{}:{}{}",reff,port,path).parse().expect("cant make value"));
    *req.uri_mut() = format!("http://localhost:{}{}",port,path).parse().expect("cant convert to uri");

    println!("Client req {:?}", req );
    let mut response = http_client.request(req).await.unwrap_or_default();
    println!("\n Client resp {:#?}\n", response);
    // if let Some(mut location) = response.headers_mut().get_mut("location"){
    //     location = &mut format!("/portforward/{}{}",port,location.to_str().unwrap()).parse().unwrap();
    // }
    if response.headers().contains_key("location"){
        let location = format!("{}",response.headers()["location"].to_str().unwrap());
        response.headers_mut().insert("location",format!("/portforward/{}{}",port,location).parse().unwrap());
    }
    if response.headers().contains_key("uri"){
        let location = format!("{}",response.headers()["uri"].to_str().unwrap());
        response.headers_mut().insert("uri",format!("/portforward/{}{}",port,location).parse().unwrap());
    }
    // let uri = format!("{}",response.uri_mut());
    // *response.uri_mut() = format!("http://localhost:{}{}",port,path).parse().expect("cant convert to uri");
    Ok(response)

}
