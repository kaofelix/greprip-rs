use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    
    let rg_args = greprip::translate_grep_args(&args_ref);
    
    let result = Command::new("rg")
        .args(&rg_args)
        .status()
        .expect("failed to execute rg");
    
    std::process::exit(result.code().unwrap_or(1));
}
