use std::sync::{Mutex, Arc};
use std::io::{Read, BufReader, Write};
use std::thread;
//use std::process::{Child, Command, Stdio, ChildStdin};
use portable_pty::{PtySystemSelection, PtySize, PtyPair, CommandBuilder, Child};
use vt100::{Parser, Screen};

pub struct ProcessShell{
    child:Box<dyn Child>,
    pair:PtyPair,
    stdout:Arc<Mutex<Vec<u8>>>,
    vt100:vt100::Parser,
    last_content:Option<Screen>
//    stderr:Option<Arc<Mutex<Vec<u8>>>>,
//    stdin:Option<ChildStdin>
}
impl ProcessShell{
    
    pub fn new()->Option<ProcessShell>{
        let pty_system = PtySystemSelection::default().get();
        match pty_system {
            Ok(pty_system)=>{
                let pair = pty_system.openpty(PtySize{
                    rows:24,
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
                                            vt100:Parser::new(24,80,0),
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

//    pub fn new_proc(&mut self,comm:&str){
//        let com_args = comm.trim().split_ascii_whitespace().collect::<Vec<_>>();
//        if let Some(com) = com_args.get(0).to_owned() {
//            let mut child_shell = Command::new(com.trim())
//                .stdin(Stdio::piped())
//                .stdout(Stdio::piped())
//                .stderr(Stdio::piped())
//                .args(&com_args[1..])
//                .spawn();
//            match child_shell {
//                Ok(mut child_shell) => {
//                    let child_in = child_shell.stdin.take().expect("Cant get child's stdin");
//                    let child_out = BufReader::new(child_shell.stdout.take().expect("Cant get child's stdout"));
//                    let child_err = BufReader::new(child_shell.stderr.take().expect("Cant get child's error"));
//                    let out = child_stream_to_vec(child_out);
//                    let err = child_stream_to_vec(child_err);
//                    self.stdout = Some(out);
//                    self.stderr = Some(err);
//                    self.child = Some(child_shell);
//                    self.stdin = Some(child_in);
//                }
//                Err(err) => {
//                    println!("Error {}", err);
//                }
//            }
//        }
//    }

    pub fn read(&mut self)->(Vec<u8>,Vec<u8>){
        let out_vec = self.stdout.clone().lock().expect("!lock").to_vec();
        self.stdout.clone().lock().expect("!lock").clear();
        self.vt100.process(out_vec.as_slice());
//        print!("{}",self.vt100.screen().contents());
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

//    pub fn read_stream(&mut self, stream:&Option<Arc<Mutex<Vec<u8>>>>) ->Vec<u8>{
//        match stream {
//            Some(out)=>{
//                let outstr = out.clone().lock().expect("!lock").to_vec();
//                out.clone().lock().expect("!lock").clear();
//                match &mut self.child {
//                    Some(child)=>{
//                        match child.try_wait(){
//                            Ok(result) =>{
//                                match result {
//                                    Some(result)=>{
//                                        println!("process exited {}",result);
//                                        self.stdin=None;
//                                        self.stdout =None;
//                                        self.child=None;
//                                    },
//                                    None=>{
////                                        println!("process running");
//                                    }
//                                }
//                            }
//                            Err(err)=>{
//                                println!("Cant wait {}",err);
//                            }
//                        }
//                    }
//                    None =>{
//                        self.stdout =None;
//                        self.stdin=None;
//                    }
//                }
//                outstr
//            }
//            None => {
//                vec![]
//            }
//        }
//    }

    pub fn write(&mut self,bytes:&Vec<u8>){
//        let str_s = String::from_utf8(bytes.to_vec());
//        match str_s {
//            Ok(str_s)=>{
//                write!(self.pair.master,"{}",&str_s.trim());
//            },
//            Err(err)=>{
//                println!("{}",err)
//            }
//        }
//        writeln!(self.pair.master,format!("{}"));

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
