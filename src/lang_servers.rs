use regex::Regex;
use futures_util::sink::SinkExt;
use futures::{FutureExt, StreamExt};
use futures_util::stream::TryStreamExt;
use std::io::Write;

use super::process_shell::child_stream_to_vec;
use super::LangServerConfig;

pub async fn handle_language_servers(socket:warp::ws::WebSocket,lang:String) ->() {



    let langs = std::fs::File::open("lang-servers.json").expect("Cant open lang_servers.json");

    let langs:Vec<LangServerConfig> = serde_json::from_reader(langs).expect("Unable to parse langs");

    let lang = langs.iter().find(|l|l.name==lang);
    if let Some(lang) = lang{
        println!("Starting language server {:#?}",lang);
        let jsonreg = Regex::new(r"\{(?:[^{}]|(?R))*\}").unwrap();

        let mut rls_child =

                    // std::process::Command::new("rls")
                    std::process::Command::new(lang.program.clone())
                    .stdout(std::process::Stdio::piped())
                    .stdin(std::process::Stdio::piped())
                    // .args(&["--cli"])
                    // .args(&["google.com"])
                    .args(&lang.args)
                    .stderr(std::process::Stdio::piped())
                    .spawn().expect(&format!("Cant spawn {}",&lang.name));

        let (tx,mut rx) = socket.split();
        // rls_child.unwrap().stdout.unwrap().forward();
        let stdout = rls_child.stdout.expect("stdout");
        let stderr = rls_child.stderr.expect("stderr");
        let mut stdin = rls_child.stdin.expect("stdin");

        // rls_child.kill();

        let out = child_stream_to_vec(stdout);
        let err_out = child_stream_to_vec(stderr);

        let (ftx, frx) = tokio::sync::mpsc::unbounded_channel();

        tokio::task::spawn(frx.forward(tx).map(|result| {
            if let Err(e) = result {
                eprintln!("websocket send error: {}", e);
            }
        }));

        std::thread::spawn(
            move||{
                loop{
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    let o =out.clone().lock().expect("!lock").to_vec();
                    if o.len()>0{
                        println!("stdout send {}",String::from_utf8(o.clone()).unwrap_or_default());
                        let outstr = String::from_utf8(o).expect("Output not utf8");

                        for jsn in  jsonreg.captures_iter(&outstr){
                            ftx.send(Ok(warp::ws::Message::text(&jsn[0])));
                        }
                        out.clone().lock().expect("!lock").clear();

                        // if let Some(header) = splits.next(){
                        //     if let Some(body) = splits.next(){
                        //     }
                        // }
                    }

                    let i = err_out.clone().lock().expect("!lock").to_vec();
                    if i.len()>0{
                        println!("stderr send {}",String::from_utf8(i.clone()).unwrap_or_default());
                        ftx.send(Ok(warp::ws::Message::binary(i)));
                        err_out.clone().lock().expect("!lock").clear();
                    }
                }
            }
        );

        // let _stream_fut=rx.try_for_each(
        //     move |msg|{
        //         let bytes = msg.as_bytes();
        //         let msg = String::from_utf8(bytes.to_vec()).unwrap_or_default();
        //         // stdin.write(msg.as_bytes());
        //         let header = format!("Content-Length: {}\r\n\r\n{}",bytes.len(),msg);
        //         write!(stdin,"{}", header);
        //         println!("recvt {:?}",header );
        //         futures::future::ok(())
        //     }
        // ).then(move |_result| {
        //     println!("Disconnected rlssocket ");
        //     // rls_child.kill();
        //     futures::future::ready(())
        // });

        while let Ok(msg) = rx.try_next().await {
            if let Some(msg)=msg{
                let bytes = msg.as_bytes();
                let msg = String::from_utf8(bytes.to_vec()).unwrap_or_default();
                // stdin.write(msg.as_bytes());
                let header = format!("Content-Length: {}\r\n\r\n{}",bytes.len(),msg);
                write!(stdin,"{}", header);
                println!("recvt {:?}",header );
            }else{
                break;
            }
        }
        println!("Disconnected");
        println!("ending ls");

    }else{
        println!("Language not in lang-servers.json" );
        ()
    }

    // futures::future::ready(())
    // let timefut = async { Ok::<(), ()>(()) };
    // futures::future::join(stream_fut, timefut).then(|_res| futures::future::ready(()))

}
