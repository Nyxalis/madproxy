fn main() {
    if let Err(e) = launch_sequence() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn launch_sequence() -> Result<(), Box<dyn std::error::Error>> {
    const LAUNCH_ASCII: &str = r#"
 __  __           _ ____                      
|  \/  | __ _  __| |  _ \ _ __ _____  ___   _ 
| |\/| |/ _` |/ _` | |_) | '__/ _ \ \/ / | | |
| |  | | (_| | (_| |  __/| | | (_) >  <| |_| |
|_|  |_|\__,_|\__,_|_|   |_|  \___/_/\_\\__, |
                                        |___/ "#;

    println!("{}", LAUNCH_ASCII);

    Ok(())
}
