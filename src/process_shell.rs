use std::sync::{Mutex, Arc};
use std::io::{Read, BufReader, Write};
use std::thread;
use std::process::{Child, Command, Stdio, ChildStdin};

pub struct ProcessShell{
    child:Child,
    out:Arc<Mutex<Vec<u8>>>,
    stdin:ChildStdin
}
impl ProcessShell{

    pub fn new()->ProcessShell{
        let mut child_shell = Command::new("bash")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Cant spawn bash");
        let child_in = child_shell.stdin.take().expect("Cant get child's stdin");
        let mut child_out = BufReader::new(child_shell.stdout.take().expect("Cant get child's stdout"));
        let out = child_stream_to_vec(child_out);
        ProcessShell{
            child:child_shell,
            out,
            stdin:child_in
        }
    }

    pub fn read(&self)->Vec<u8>{
        let outstr = self.out.clone().lock().expect("!lock").to_vec();
        self.out.clone().lock().expect("!lock").clear();
        outstr
    }

    pub fn write(&mut self,bytes:&Vec<u8>){
        self.stdin.write(bytes).expect("Cant Write to stdin");
    }

    pub fn kill(&mut self){
        self.child.kill().expect("Cant close child");
    }

}


fn child_stream_to_vec<R>(mut stream: R) -> Arc<Mutex<Vec<u8>>>
    where
        R: Read + Send + 'static,
{
    let out = Arc::new(Mutex::new(Vec::new()));
    let vec = out.clone();
    thread::Builder::new()
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
