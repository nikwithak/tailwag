fn main() {
    let mut args = std::env::args();
    let _app_name = args.next().expect("No command name found in Args.");
    match args.next().as_deref() {
        Some("new") => println!("new not currently supported"),
        Some("migrate") => println!("Migrate not currenlty supported."),
        Some("deploy") => println!("deploy not currently supported"),
        // Some("connect") => todo!(),
        // Some("run") => todo!(), // Thinkign: `tw run gui` or `tw run webservice` or `tw run tasks` / `tw run all`
        Some(_) | None => print_help(),
    }
}

fn print_help() {
    println!("Usage: tailwag <action> <filepath>");
}
