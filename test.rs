use std::env;
use std::path::PathBuf;


fn main(){
    let mut working_path=String::new();
    match env::current_dir(){
        Ok(path)=>working_path=path.display().to_string(),
        Err(e)=>panic!("Failed to load working dir"),
    }
    println!("{:?}",working_path);
}