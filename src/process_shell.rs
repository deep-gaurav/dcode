use std::sync::{Mutex, Arc};
use std::io::{Read, BufReader, Write};
use std::thread;
use std::process::{Child, Command, Stdio, ChildStdin};

pub struct ProcessShell{
    child:Option<Child>,
    stdout:Option<Arc<Mutex<Vec<u8>>>>,
    stderr:Option<Arc<Mutex<Vec<u8>>>>,
    stdin:Option<ChildStdin>
}
impl ProcessShell{
    
    pub fn new()->ProcessShell{
        ProcessShell{
            child: None,
            stdout: None,
            stderr:None,
            stdin: None
        }
    }

    pub fn new_proc(&mut self,comm:&str){
        let com_args = comm.trim().split_ascii_whitespace().collect::<Vec<_>>();
        if let Some(com) = com_args.get(0).to_owned() {
            let mut child_shell = Command::new(com.trim())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .args(&com_args[1..])
                .spawn();
            match child_shell {
                Ok(mut child_shell) => {
                    let child_in = child_shell.stdin.take().expect("Cant get child's stdin");
                    let child_out = BufReader::new(child_shell.stdout.take().expect("Cant get child's stdout"));
                    let child_err = BufReader::new(child_shell.stderr.take().expect("Cant get child's error"));
                    let out = child_stream_to_vec(child_out);
                    let err = child_stream_to_vec(child_err);
                    self.stdout = Some(out);
                    self.stderr = Some(err);
                    self.child = Some(child_shell);
                    self.stdin = Some(child_in);
                }
                Err(err) => {
                    println!("Error {}", err);
                }
            }
        }
    }

    pub fn read(&mut self)->(Vec<u8>,Vec<u8>){
        let out_vec = self.read_stream( &self.stdout.clone());
        let err_vec = self.read_stream( &self.stderr.clone());
        (out_vec,err_vec)
    }

    pub fn read_stream(&mut self, stream:&Option<Arc<Mutex<Vec<u8>>>>) ->Vec<u8>{
        match stream {
            Some(out)=>{
                let outstr = out.clone().lock().expect("!lock").to_vec();
                out.clone().lock().expect("!lock").clear();
                match &mut self.child {
                    Some(child)=>{
                        match child.try_wait(){
                            Ok(result) =>{
                                match result {
                                    Some(result)=>{
                                        println!("process exited {}",result);
                                        self.stdin=None;
                                        self.stdout =None;
                                        self.child=None;
                                    },
                                    None=>{
//                                        println!("process running");
                                    }
                                }
                            }
                            Err(err)=>{
                                println!("Cant wait {}",err);
                            }
                        }
                    }
                    None =>{
                        self.stdout =None;
                        self.stdin=None;
                    }
                }
                outstr
            }
            None => {
                vec![]
            }
        }
    }

    pub fn write(&mut self,bytes:&Vec<u8>){
        match &mut self.child {
            Some(child)=>{
                match child.try_wait(){
                    Ok(child) => {
                        match child {
                            Some(status)=>{
                                println!("Old process dead starting new process");
                                let string_cmd = String::from_utf8(bytes.clone());
                                match string_cmd {
                                    Ok(cmd)=>self.new_proc(&cmd),
                                    Err(err) => {
                                        println!("{}",err)
                                    }
                                }
                            }
                            None =>{
                                println!("Old process not dead, writing in it");
                                match &mut self.stdin {
                                    Some(stdin)=>{
                                        stdin.write(bytes);
                                    }
                                    None => {
                                        println!("NO STDIN");
                                    }
                                }
                            }
                        }
                    }
                    Err(err) =>{
                        println!("Error attempting to wait")
                    }
                }
            }
            None => {
                println!("No process Starting new");
                let string_cmd = String::from_utf8(bytes.clone());
                match string_cmd {
                    Ok(cmd)=>self.new_proc(&cmd),
                    Err(err) => {
                        println!("{}",err)
                    }
                }
            }
        }
    }

    pub fn kill(&mut self){
        match &mut self.child {
            Some(child )=>{
                child.kill();
            },
            None=>{

            }
        }
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
