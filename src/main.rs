use crate::process_shell::ProcessShell;
//use serde::{Serialize,Deserialize};
use futures::{FutureExt, SinkExt, StreamExt};
use futures_util::stream::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::string::FromUtf8Error;
use std::sync::{Arc, Mutex};
use warp::filters::ws::Message;
use warp::Filter;

mod process_shell;

#[derive(Serialize, Deserialize, Debug)]
struct TransferData {
    command: String,
    value: String,
    args: Vec<String>,
}

struct Server {
    out: tokio::sync::mpsc::UnboundedSender<Result<warp::filters::ws::Message, warp::Error>>,
    shells: HashMap<String, ProcessShell>,
}

impl Server {
    fn handle_ping(&mut self, data: &TransferData) {
        if let Some(time) = data.args.get(0) {
            if let Ok(time) = time.parse::<i64>() {
                let now_time = chrono::Utc::now().timestamp_millis();
                println!("connection id  : ping one way time : {}", now_time - time);
                let pong = TransferData {
                    command: "ping".to_string(),
                    value: data.value.clone(),
                    args: vec![format!("{}", time), format!("{}", now_time)],
                };
                if let Ok(send_str) = serde_json::to_string(&pong) {
                    self.out.send(Ok(Message::text(send_str)));
                }
            }
        }
    }

    fn list_processes(&self) -> Vec<String> {
        let mut list_process = Vec::new();
        for process in &self.shells {
            list_process.push(process.0.clone());
        }
        list_process
    }

    fn send_process_list(&mut self) {
        let list_process = self.list_processes();
        let response_data = TransferData {
            command: "process".to_string(),
            value: "list".to_string(),
            args: list_process,
        };
        let response_str =
            serde_json::to_string(&response_data).expect("Cant convert Transfer Data to JSON");
        println!("Send process list {}",response_str);
        match self.out.send(Ok(Message::text((response_str)))){
            Ok(d)=>{
                println!("Ok pl send {:?}",d)
            }
            Err(e)=>{
                println!("Err pl {}",e)
            }
        }
    }

    fn handle_process(&mut self, data: &TransferData) {
        match data.value.as_str() {
            "list" => self.send_process_list(),
            "new" => {
                for pid in &data.args {
                    self.shells
                        .insert(pid.clone(), ProcessShell::new().expect("Cant start pty"));
                }
                self.send_process_list()
            }
            "kill" => {
                for pid in &data.args {
                    match self.shells.get_mut(&pid.clone()) {
                        Some(shell) => {
                            shell.kill();
                            self.shells.remove(pid);
                        }
                        None => println!("shell id {} not found", pid),
                    }
                }
                self.send_process_list()
            }
            _ => {
                println!("Unknown Process command {}", data.value);
            }
        }
    }

    fn handle_exec(&mut self, data: &TransferData) {
        let process = self.shells.get_mut(data.value.as_str());
        match process {
            Some(process) => {
                let inp: String = data.args.join(" ");
                println!("exec {}", inp);
                process.write(&Vec::from(inp));
            }
            None => {
                println!("Cant find processid : {} to exec ", data.value);
            }
        }
    }
}

impl Server {
    fn on_message(&mut self, msg: Message) {
        //        println!("receive msg ");
        let msg_bytes= msg.as_bytes();
        if let Ok(string_msg) = String::from_utf8(msg_bytes.to_owned()) {
            match serde_json::from_str::<TransferData>(&string_msg) {
                Ok(data) => match data.command.as_str() {
                    "ping" => self.handle_ping(&data),
                    "process" => self.handle_process(&data),
                    "exec" => self.handle_exec(&data),
                    _ => {
                        println!("Unrecognised command {}", data.command);
                    }
                },
                Err(err) => {
                    println!("Error {}", err);
                }
            }
        }
    }

    fn on_close(&mut self) {
        for process in &mut self.shells {
            process.1.kill();
        }
        println!("Connection closing due to ");
    }

    fn on_timeout(&mut self) {
        for process in &mut self.shells {
            let (stdout, stderr) = process.1.read();
            let mut out_string = String::new();
            let mut err_string = String::new();
            if !&stdout.is_empty() {
                //                        let stdout = strip_ansi_escapes::strip(stdout.clone()).unwrap_or(stdout);
                let out_st = String::from_utf8(stdout);
                match out_st {
                    Ok(out_st) => out_string = out_st,
                    Err(err) => println!("{}", err),
                }
            }
            if !&stderr.is_empty() {
                let stderr = strip_ansi_escapes::strip(stderr.clone()).unwrap_or(stderr);
                let err_st = String::from_utf8(stderr);
                match err_st {
                    Ok(err_st) => {
                        err_string = err_st;
                    }
                    Err(err) => println!("{}", err),
                }
            }
            if !out_string.is_empty() || !err_string.is_empty() {
                println!("Received out {} {}", out_string, err_string);
                let msg = TransferData {
                    command: "exec".to_string(),
                    value: process.0.to_string(),
                    args: vec![out_string, err_string],
                };
                let msg_json = serde_json::to_string(&msg);
                match msg_json {
                    Ok(msg_str) => {
                        println!("Send {}",msg_str);
                        match self.out.send(Ok(Message::text(msg_str))){
                            Ok(e)=>{
                                println!("Sent {:?}",e);
                            }

                            Err(e)=>{
                                println!("Error {}",e)
                            }
                        }
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }
        }
    }
}

// fn main_old(){
//     let addr = format!("{}:{}",std::env::var("HOST").unwrap_or("0.0.0.0".to_owned()),std::env::var("PORT").unwrap_or("3012".to_owned()));
//     println!("Running on address {} ",addr);

//     let ws_ser = Builder::new().with_settings(Settings{
//         tcp_nodelay:true,
//         ..Settings::default()
//     }).build(|out| {
//         Server{out, shells:HashMap::new()}}).unwrap();

//     if let Err(error) = ws_ser.listen(&addr) {
//         // Inform the user of failure
//         println!("Failed to create WebSocket due to {:?}", error);
//     }
// }

enum WSMes {
    Connect,
    Disconnect,
    Message(Message),
}

#[tokio::main]
async fn main() {
    let ws_serve = warp::path("ws").and(warp::ws()).map(|ws: warp::ws::Ws| {
        ws.on_upgrade(|socket| {
            println!("New connection");
            let (mut tx, mut rx) = socket.split();

            let (ctx, crx) = std::sync::mpsc::channel();

            let (ftx, frx) = tokio::sync::mpsc::unbounded_channel();

            tokio::task::spawn(frx.forward(tx).map(|result| {
                if let Err(e) = result {
                    eprintln!("websocket send error: {}", e);
                }
            }));

            std::thread::spawn(move||{
                
                let mut ws_handler = Server {
                    shells: HashMap::new(),
                    out: ftx.clone(),
                };
                println!("New handler");
                for msg in crx{
                    match msg {
                        WSMes::Connect => {
                            println!("Connected");
                        }
                        WSMes::Disconnect => {
                            println!("Disconnected");
                            ws_handler.on_close();
                            break;
                        }
                        WSMes::Message(msg) => {
                            // ftx.send(Ok(msg));
                            ws_handler.on_message(msg);
                            ws_handler.on_timeout();
                        }
                    }
                    // futures::future::ready(())
                }
            });

            ctx.send(WSMes::Connect);
            let ctx_dis = ctx.clone();

            let stream_fut = rx
                .try_for_each(move |msg| {
                    // println!("message {}", msg.());
                    // ctx.send(
                    //     Ok(warp::ws::Message::text("anything".clone()))
                    // );

                    // let sendt = tx.send(Message::text("anything"));

                    // sendt
                    ctx.send(WSMes::Message(msg));
                    futures::future::ok(())

                    // futures::future::join(sendt, futures::future::ready((()))).then(|res|{futures::future::ok(())})
                })
                .then(move |result| {
                    println!("Disconnected ");
                    ctx_dis.clone().send(WSMes::Disconnect);
                    futures::future::ready(())
                })
                .map(|result| println!("{:?}", result));
            let timefut = async { Ok::<(), ()>(()) };
            futures::future::join(stream_fut, timefut).then(|res| futures::future::ready(()))
        })
    });

    let fs_s = warp::path("files").and(warp::fs::dir("./files"));

    let routes = ws_serve.or(fs_s);

    // let addr = format!("{}:{}",std::env::var("HOST").unwrap_or("0.0.0.0".to_owned()),std::env::var("PORT").unwrap_or("3012".to_owned()));
    
    warp::serve(routes).run(([0, 0, 0, 0], std::env::var("PORT").unwrap_or("3012".to_owned()).parse().unwrap())).await
}
