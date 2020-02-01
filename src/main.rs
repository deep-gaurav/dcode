use ws::{listen, Sender, Handler, Handshake, CloseCode, Message, Builder, Settings};
use crate::process_shell::ProcessShell;
use ws::util::Token;
//use serde::{Serialize,Deserialize};
use serde::{Serialize,Deserialize};
use std::collections::HashMap;
use std::string::FromUtf8Error;

mod process_shell;

#[derive(Serialize,Deserialize,Debug)]
struct TransferData{
    command:String,
    value:String,
    args:Vec<String>
}

const PING :Token = Token(1);
const PROCESS_TICK:Token = Token(2);

struct Server{
    out:Sender,
    shells:HashMap<String,ProcessShell>
}

impl Server{
    fn handle_ping(&mut self,data:&TransferData){
        if let Some(time)= data.args.get(0){
            if let Ok(time)=time.parse::<i64>(){
                let now_time = chrono::Utc::now().timestamp_millis();
                println!("connection id {} : ping one way time : {}",self.out.connection_id(),now_time-time);
                let pong = TransferData{
                    command: "ping".to_string(),
                    value: data.value.clone(),
                    args: vec![format!("{}",time),format!("{}",now_time)]
                };
                if let Ok(send_str)=serde_json::to_string(&pong){
                    self.out.send(Message::from(send_str));
                }
            }
        }
    }

    fn list_processes(&self)->Vec<String>{

        let mut list_process = Vec::new();
        for process in &self.shells {
            list_process.push(process.0.clone());
        }
        list_process
    }

    fn send_process_list(&mut self){

        let list_process = self.list_processes();
        let response_data = TransferData{
            command:"process".to_string(),
            value:"list".to_string(),
            args:list_process
        };
        let response_str = serde_json::to_string(&response_data).expect("Cant convert Transfer Data to JSON");
        self.out.send(Message::from(response_str));

    }

    fn handle_process(&mut self,data:&TransferData){
        match data.value.as_str() {

            "list" => self.send_process_list(),
            "new" => {
                for pid in &data.args{
                    self.shells.insert(
                        pid.clone(),
                        ProcessShell::new().expect("Cant start pty")
                    );
                }
                self.send_process_list()
            },
            "kill" => {
                for pid in &data.args{
                    match self.shells.get_mut(&pid.clone()) {
                        Some(shell) =>{
                            shell.kill();
                            self.shells.remove(pid);
                        }
                        None => {
                            println!("shell id {} not found",pid)
                        }
                    }
                }
                self.send_process_list()
            },
            _ => {
                println!("Unknown Process command {}",data.value);
            }
        }
    }

    fn handle_exec(&mut self,data:&TransferData){
        let process = self.shells.get_mut(data.value.as_str());
        match process {
            Some(process) => {
                let inp:String = data.args.join(" ");
                println!("exec {}",inp);
                process.write(
                    &Vec::from(inp)
                );
            },
            None => {
                println!("Cant find processid : {} to exec ",data.value);
            }
        }
    }

}

impl Handler for Server{

    fn on_open(&mut self,shake:Handshake) -> ws::Result<()>{
        if let Some(ip_addr) = shake.remote_addr()? {
            println!("Connection opened from {}. connection_id {} ", ip_addr,self.out.connection_id())
        } else {
            println!("Unable to obtain client's IP address.")
        }
        self.out.timeout(100,PROCESS_TICK);
        Ok(())
    }



    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
//        println!("receive msg ");
        if let Ok(string_msg) = String::from_utf8(msg.into_data()){
            match serde_json::from_str::<TransferData>(&string_msg){
                Ok(data)=>{
                    match data.command.as_str() {
                        "ping" => self.handle_ping(&data),
                        "process" => self.handle_process(&data),
                        "exec" => self.handle_exec(&data),
                        _ => {
                            println!("Unrecognised command {}",data.command);
                        }
                    }
                }
                Err(err)=>{
                    println!("Error {}",err);
                }
            }
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        for process in &mut self.shells {
            process.1.kill();
        }
        println!("Connection closing due to ({:?}) {}, connection_id {} ", code, reason,self.out.connection_id());
    }


    fn on_timeout(&mut self, event: Token) -> ws::Result<()> {
        match event {
            PROCESS_TICK =>{
                for process in &mut self.shells{
                    let (stdout,stderr) = process.1.read();
                    let mut out_string = String::new();
                    let mut err_string = String::new();
                    if !&stdout.is_empty(){
//                        let stdout = strip_ansi_escapes::strip(stdout.clone()).unwrap_or(stdout);
                        let out_st = String::from_utf8(stdout);
                        match out_st {
                            Ok(out_st) => {
                                out_string = out_st
                            }
                            Err(err) => {
                                println!("{}",err)
                            }
                        }
                    }
                    if !&stderr.is_empty(){
                        let stderr = strip_ansi_escapes::strip(stderr.clone()).unwrap_or(stderr);
                        let err_st = String::from_utf8(stderr);
                        match err_st {
                            Ok(err_st)=>{
                                err_string=err_st;
                            }
                            Err(err)=>{
                                println!("{}",err)
                            }
                        }
                    }
                    if !out_string.is_empty() || !err_string.is_empty(){
                        println!("Received out {} {}",out_string,err_string);
                        let msg = TransferData{
                            command:"exec".to_string(),
                            value:process.0.to_string(),
                            args:vec![out_string,err_string]
                        };
                        let msg_json = serde_json::to_string(&msg);
                        match msg_json {
                            Ok(msg_str) => {
                                self.out.send(Message::from(msg_str));
                            }
                            Err(err)=>{
                                println!("{}",err);
                            }
                        }
                    }
                }
                self.out.timeout(100,PROCESS_TICK);
            }
            _ => {}
        }
        Ok(())
    }

}



fn main(){
    let addr = format!("{}:{}",std::env::var("HOST").unwrap_or("0.0.0.0".to_owned()),std::env::var("PORT").unwrap_or("3012".to_owned()));
    println!("Running on address {} ",addr);

    let ws_ser = Builder::new().with_settings(Settings{
        tcp_nodelay:true,
        ..Settings::default()
    }).build(|out| {
        Server{out, shells:HashMap::new()}}).unwrap();

    if let Err(error) = ws_ser.listen(&addr) {
        // Inform the user of failure
        println!("Failed to create WebSocket due to {:?}", error);
    }
}