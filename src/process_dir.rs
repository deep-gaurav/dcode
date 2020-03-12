
use std::fs::{self};
use std::path::Path;
use serde::{Serialize,Deserialize};

use std::io::Write;
use std::io::Read;

use super::{Server,TransferData,Message};

#[derive(Serialize,Deserialize,Debug)]
struct FsEntry {
    is_dir:bool,
    name:String,
    path:String
}

impl Server{

    fn handle_list_dir(&mut self,dirpath:&str){
        let mut lists:Vec<_> = list_dir(Path::new(dirpath));
        let path = Path::new(dirpath);
        if let Some(parent) = path.parent(){
            lists.push(FsEntry{
                is_dir:parent.is_dir(),
                name:"..".to_string(),
                path:parent.to_str().unwrap_or_default().to_string()
            });
        }
        let lists=lists.iter().filter_map(|f|serde_json::to_string(f).ok()).collect();
        let out_data = TransferData{
            args:lists,
            command:"fs".to_string(),
            value:"list".to_string()
        };
        if let Ok(out_str)= serde_json::to_string(&out_data){

            match self.out.send(
                Ok(Message::text(out_str))
            ){
                Ok(_msg)=>{

                },
                Err(err)=>{
                    eprintln!("Error sending {:?}",err);
                }
            }
        }
    }

    pub fn create_file(&mut self,data:&TransferData)->Option<fs::File>{
        if let Some(filepath) = data.args.get(0) {
            match serde_json::from_str::<FsEntry>(filepath){
                Ok(entry)=>{

                    let path = Path::new(&entry.path);
                    match entry.is_dir{
                        true => {
                            match fs::create_dir(path){
                                Ok(dir)=>{
                                    self.handle_list_dir(&entry.path);
                                }
                                Err(err)=>{
                                    eprintln!("Failed to create dir");

                                }
                            }
                        }
                        false =>{
                            let file = fs::File::create(path);
                            match file {
                                Ok(_file)=>{
                                    // self.handle_list_dir(&path.parent().expect("no parent?").to_str().unwrap_or_default());
                                    return Some(_file)
                                }
                                Err(err)=>{
                                    eprintln!("Cant create file {:?}",err);
                                }
                            }
                        }
                    }
                }
                Err(err)=>{
                    eprintln!("Not fs Entry {:?}",err );
                }
            }
        };
        None
    }

    pub fn handle_fs(&mut self, data: &TransferData){
        match data.value.as_str(){
            "list" => {
                if let Some(dirpath) = data.args.get(0) {
                    self.handle_list_dir(dirpath);
                }
            }

            "new" => {
                if let Some(file)=self.create_file(data){
                    if let Ok(data_str) = serde_json::to_string(data){
                        self.out.send(
                            Ok(
                                Message::text(data_str)
                            )
                        );
                    }
                }
            }
            "new_dir" =>{
                if let Some(path)=data.args.get(0){
                    if let Ok(dir) = fs::create_dir_all(path){
                        self.out.send(
                            Ok(
                                Message::text(serde_json::to_string(data).unwrap())
                            )
                        );
                    }
                }
            }
            "delete" => {
                if let Some(file)=data.args.get(0){
                    let path = Path::new(file);
                    if path.is_dir(){
                        if let Ok(_)=fs::remove_dir_all(path){
                            self.out.send(
                                Ok(
                                    Message::text(serde_json::to_string(data).unwrap())
                                )
                            );
                        }
                    }else{
                        if let Ok(_)=fs::remove_file(path){
                            self.out.send(
                                Ok(
                                    Message::text(serde_json::to_string(data).unwrap())
                                )
                            );
                        }
                    }
                }
            }


            "save" => {
                if let Some(mut file)=self.create_file(data){
                    if  let Ok(_)=file.write_all(data.args[1].as_bytes()){
                        // let send_data = data.clone();
                        if let Ok(data_str) = serde_json::to_string(data){
                            self.out.send(
                                Ok(
                                    Message::text(data_str)
                                )
                            );
                        }
                    }

                }
            }

            "open" => {
                println!("openn file {:?}",data );
                if let Some(path) = data.args.get(0){
                    if let Ok(mut file)= fs::File::open(path){
                        let mut buf = String::new();
                        let read_data = file.read_to_string(&mut buf);
                        if let Ok(read_data)=read_data{
                            let mut send_data = data.clone();
                            send_data.args.push(buf);
                            if let Ok(data_str) = serde_json::to_string(&send_data){
                                self.out.send(
                                    Ok(
                                        Message::text(data_str)
                                    )
                                );
                            }
                        }

                    }
                }
            }

            any => eprintln!("unknown fs command {}",any)
        }
    }

}

fn list_dir(dir: &Path) -> Vec<FsEntry> {
    if dir.is_dir(){
        match fs::read_dir(dir){
            Ok(dir_iter)=>{
                return dir_iter
                .filter_map(
                    |f| match f{
                        Ok(entry)=>{
                            Some(FsEntry{
                                is_dir:entry.path().is_dir(),
                                name:entry.path().file_name().unwrap_or_default().to_str().unwrap_or_default().to_string(),
                                path:entry.path().canonicalize().unwrap_or_default().to_str().unwrap_or_default().to_string()
                            })

                        }
                        Err(err)=>{
                            eprintln!("{:?}",err);
                            None
                        }
                    }
                )
                .collect();
            }
            Err(err)=>{
                eprintln!("Error reading dir {:?}",err);
            }
        }
    }
    vec![]
}
