use ws::{listen, Sender, Handler, Handshake, CloseCode, Message};
use crate::process_shell::ProcessShell;
use ws::util::Token;
//use serde::{Serialize,Deserialize};
use serde::{Serialize,Deserialize};

mod process_shell;

#[derive(Serialize,Deserialize,Debug)]
struct TransferData{
    command:String,
    value:String,
    args:Vec<String>
}

const PING :Token = Token(1);
const READ_PROCESS_OUT:Token = Token(2);

struct Server{
    out:Sender,
    shell:Vec<ProcessShell>
}
impl Handler for Server{

    fn on_open(&mut self,shake:Handshake) -> ws::Result<()>{
        if let Some(ip_addr) = shake.remote_addr()? {
            println!("Connection opened from {}. connection_id {} ", ip_addr,self.out.connection_id())
        } else {
            println!("Unable to obtain client's IP address.")
        }
        self.out.timeout(100,Token(1));
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
//        println!("receive msg ");
        if let Ok(string_msg) = String::from_utf8(msg.into_data()){
            match serde_json::from_str::<TransferData>(&string_msg){
                Ok(data)=>{
                    if data.command=="ping"{
                        if let Some(time)= data.args.get(0){
                            if let Ok(time)=time.parse::<i64>(){
                                let now_time = chrono::Utc::now().timestamp_millis();
                                println!("connection id {} : ping one way time : {}",self.out.connection_id(),now_time-time);
                                let pong = TransferData{
                                    command: "ping".to_string(),
                                    value: data.value,
                                    args: vec![format!("{}",now_time)]
                                };
                                if let Ok(sendstr)=serde_json::to_string(&pong){
                                    self.out.send(Message::from(sendstr));
                                }
                            }
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
        for mut process in &mut self.shell{
            process.kill();
        }
        println!("Connection closing due to ({:?}) {}, connection_id {} ", code, reason,self.out.connection_id());
    }

    fn on_timeout(&mut self, event: Token) -> ws::Result<()> {
        Ok(())
    }

}



fn main(){
    let addr = format!("{}:{}",std::env::var("HOST").unwrap_or("0.0.0.0".to_owned()),std::env::var("PORT").unwrap_or("3012".to_owned()));
    println!("Running on address {} ",addr);
    if let Err(error) = listen(&addr, |out| {
        Server{out,shell:Vec::new()}}) {
        // Inform the user of failure
        println!("Failed to create WebSocket due to {:?}", error);
    }
}