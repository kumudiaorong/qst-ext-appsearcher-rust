use xlog_rs::log;
pub struct Flags {
    pub id: String,
    pub endpoint: tonic::transport::Endpoint,
}
fn show_help(code: i32) {
    println!("Usage: qst [options]");
    println!("Options:");
    println!("  --uri <uri>    set uri");
    println!("  --id <id>      set id");
    println!("  --help         show help");
    std::process::exit(code);
}
impl Flags {
    pub fn new(args: Vec<String>) -> Self {
        let mut id = String::new();
        let mut addr = String::new();
        for (i, arg) in args.iter().enumerate() {
            match arg.as_str() {
                "--help" => {
                    show_help(0);
                }
                "--id" => {
                    if i + 1 < args.len() {
                        id = args[i + 1].clone();
                    }
                }
                "--uri" => {
                    if i + 1 < args.len() {
                        addr = args[i + 1].clone();
                    }
                }
                _ => {}
            }
        }
        if id.is_empty() {
            log::error("The id is empty");
            show_help(1);
        }
        if addr.is_empty() {
            log::error("The uri is empty");
            show_help(2);
        }
        if let Ok(ep) = tonic::transport::Channel::from_shared(addr.clone()) {
            log::info(format!("Flags: id={}, uri={}", id, addr).as_str());
            Self { id, endpoint: ep }
        } else {
            log::error(format!("invalid uri: {}", addr).as_str());
            show_help(3);
            unreachable!()
        }
    }
}
