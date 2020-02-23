use std::sync::{Mutex, Arc};
use std::io::{Read, BufReader, Write};
use std::thread;
use portable_pty::{PtySystemSelection, PtySize, PtyPair, CommandBuilder, Child};
use vt100::{Parser, Screen};

pub struct ProcessShell{
    child:Box<dyn Child>,
    pair:PtyPair,
    stdout:Arc<Mutex<Vec<u8>>>,
    vt100:vt100::Parser,
    last_content:Option<Screen>
}
impl ProcessShell{
    
    pub fn new()->Option<ProcessShell>{
        let pty_system = PtySystemSelection::default().get();
        match pty_system {
            Ok(pty_system)=>{
                let pair = pty_system.openpty(PtySize{
                    rows:200,
                    cols:80,
                    pixel_height:0,
                    pixel_width:0
                });
                match pair {
                    Ok(pair) => {
                        let cmd = CommandBuilder::new("bash");
                        let child = pair.slave.spawn_command(cmd);
                        match child {
                            Ok(child)=>{
                                let reader = pair.master.try_clone_reader();
                                match reader {
                                    Ok(reader)=>{
                                        let out = child_stream_to_vec(reader);
                                        Some(ProcessShell{
                                            child,
                                            pair,
                                            stdout: out,
                                            vt100:Parser::new(200,80,0),
                                            last_content:None
                                        })
                                    },
                                    Err(err)=>None
                                }
                            },
                            Err(err)=>None
                        }
                    },
                    Err(err) => {
                        None
                    },
                }
            }
            Err(err)=>{
                None
            }
        }
    }

    pub fn read(&mut self)->(Vec<u8>,Vec<u8>){
        let out_vec = self.stdout.clone().lock().expect("!lock").to_vec();
        self.stdout.clone().lock().expect("!lock").clear();
        self.vt100.process(out_vec.as_slice());
        match &self.last_content {
            Some(content)=>{
                let new_screen = self.vt100.screen().clone();
                let content_diff = new_screen.contents_diff(content).to_vec();
                self.last_content=Some(new_screen.clone());
                if content_diff.is_empty(){
                    return (vec![],vec![]);
                }
                (new_screen.contents().into_bytes(),vec![])
            }
            None=>{
                let screen = self.vt100.screen().clone();
                self.last_content = Some(screen);
                (self.vt100.screen().contents().into_bytes(),vec![])
            }
        }
    }


    pub fn write(&mut self,bytes:&Vec<u8>){
        self.pair.master.write(bytes.as_slice());
    }

    pub fn kill(&mut self){
        self.child.kill();
    }

}


fn child_stream_to_vec<R>(mut stream: R) -> Arc<Mutex<Vec<u8>>>
    where
        R: Read + Send + 'static,
{
    let out = Arc::new(Mutex::new(Vec::new()));
    let vec = out.clone();
    let thread = thread::Builder::new()
        .name("child_stream_to_vec".into())
        .spawn(move || loop {
            let mut buf = [0];
            match stream.read(&mut buf) {
                Err(err) => {
                    println!("{}] Error reading from stream: {}", line!(), err);
                    break;
                }
                Ok(got) => {
                    if got == 0 {
                        break;
                    } else if got == 1 {
                        vec.lock().expect("!lock").push(buf[0])
                    } else {
                        println!("{}] Unexpected number of bytes: {}", line!(), got);
                        break;
                    }
                }
            }
        })
        .expect("!thread");

    out
}
