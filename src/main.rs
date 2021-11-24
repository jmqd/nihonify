use nihonify;
use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("nihonify")
        .version("0.1")
        .author("Jordan McQueen <j@jm.dev>")
        .subcommand(
            SubCommand::with_name("convert-date").arg(
                Arg::with_name("date")
        .long("date") // allow --name
        .takes_value(true)
        .help("A YYYY-mm-dd gregorian date to convert to nengou.")
        .required(true),
            ),
        )
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("convert-date") {
        println!(
            "{}",
            nihonify::Era::to_jp_nenkou_string(nihonify::utc_dt(matches.value_of("date").unwrap()))
                .unwrap()
        );
    }
}
