use ws::{listen, Sender, Handler, Handshake, CloseCode, Message};
use crate::process_shell::ProcessShell;
use ws::util::Token;

mod process_shell;

struct Server{
    out:Sender,
    shell:ProcessShell
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
        let data =msg.into_data();
        println!("Shell write {:?}",data);
        self.shell.write(&data);
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        self.shell.kill();
        println!("Connection closing due to ({:?}) {}, connection_id {} ", code, reason,self.out.connection_id());
    }

    fn on_timeout(&mut self, event: Token) -> ws::Result<()> {
        match event{
            Token(1) => {
                self.out.timeout(100,Token(1));

                let data = self.shell.read();
                if !data.is_empty(){
                    println!("Shell read {:?}", String::from_utf8(data.clone()));
                    self.out.ping(data.clone());
                    self.out.send(Message::from(String::from_utf8(data).unwrap()));
                }
            },
            _ => {

            }
        }
        Ok(())
    }

}



fn main(){
    let addr = format!("{}:{}",std::env::var("HOST").unwrap_or("0.0.0.0".to_owned()),std::env::var("PORT").unwrap_or("3012".to_owned()));
    println!("Running on address {} ",addr);
    if let Err(error) = listen(&addr, |out| {
        Server{out,shell:ProcessShell::new()}}) {
        // Inform the user of failure
        println!("Failed to create WebSocket due to {:?}", error);
    }
}