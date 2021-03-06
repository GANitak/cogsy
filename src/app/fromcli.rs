use std::io;
use std::process;
use clap::{
    App as Clap,
    SubCommand,
    Arg,
    ArgMatches,
};
use crate::utils::{self, Config};
use crate::app::{
    ListenLogEntry,
    Release,
    App,
    update,
    database::{
        query::{self, QueryType},
        update as dbupdate,
    },
    message::{Message, MessageKind},
};
use crate::screens::popup::format_vec;

pub fn init<'a>() -> Clap<'a, 'a> {
    Clap::new("cogsy")
        .author("cartoon.raccoon")
        .version("0.1.2")
        .about("A command line Discogs client written in Rust")
        .subcommand(SubCommand::with_name("update")
            .about("Updates the cogsy database.")
            .arg(Arg::with_name("username")
                .short("u")
                .long("username")
                .takes_value(true)
                .value_name("USERNAME")
                .help("Updates the username"))
            .arg(Arg::with_name("token")
                .short("t")
                .long("token")
                .takes_value(true)
                .value_name("token")
                .help("Updates the token")
            )
        )
        .subcommand(SubCommand::with_name("random")
            .about("Select a random song to play.")
            .arg(Arg::with_name("nolog")
                .short("n")
                .long("nolog")
                .takes_value(false)
                .required(false)
                .help("The album won't be logged to the listening log.")
            )
        )
        .subcommand(SubCommand::with_name("listen")
            .about("Log an album that you're listening to.")
            .arg(Arg::with_name("albumname")
                .required(true)
                .help("The name of the album you want to play.")
            )
        )
        .subcommand(SubCommand::with_name("query")
            .about("Query the database for an album.")
            .arg(Arg::with_name("wantlist")
                .short("w")
                .long("wantlist")
                .takes_value(true)
                .help("Use this switch to query from the wantlist.")
            )
            .arg(Arg::with_name("albumname")
                .help("The name of the album you want to query.")
        )
    )
}

pub fn parse_and_execute(clapapp: ArgMatches) -> Option<()> {
    let app = App::initialize();
    if let Some(sub_m) = clapapp.subcommand_matches("update") {
        if sub_m.is_present("username") {
            println!("Sorry, in-app username updates are unsupported at this time.");
        } else if sub_m.is_present("token") {
            println!("Sorry, in-app token updates are unsupported at this time.");
        } else {
            println!("{}",
                Message::set("Beginning full database update.", MessageKind::Info)
            );
            match update::full(&app.user_id, &app.token, true, false) {
                Ok(()) => {}
                Err(e) => {eprintln!("{}", e)}
            }
        }
        Some(())
    } else if let Some(sub_m) = clapapp.subcommand_matches("random") {
        if sub_m.is_present("nolog") {
            println!("{}", 
                Message::set("Selecting random album without logging.", MessageKind::Info)
            );
            match query::random() {
                Ok(random) => {
                    println!("You should play `{}.`", random.title);
                }
                Err(e) => {
                    eprintln!("Oops: {}", e);
                }
            }
        } else {
            println!("{}", 
                Message::set("Selecting random album with logging.", MessageKind::Info)
            );
            match query::random() {
                Ok(random) => {
                    let time_now = utils::get_utc_now();
                    let entry = ListenLogEntry {
                        id: random.id,
                        title: random.title.clone(),
                        time: time_now,
                    };
                    match dbupdate::listenlog(entry) {
                        Ok(()) => {
                            println!("You should play `{}`.", random.title);
                        }
                        Err(e) => {
                            eprintln!("Oops: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Oops: {}", e);
                }
            }
        }
        Some(())
    } else if let Some(sub_m) = clapapp.subcommand_matches("listen") {
        let album = sub_m.value_of("albumname").unwrap().to_string()
        .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], "");

        match query::release(album.clone(), QueryType::Collection) {
            Ok(results) => {
                if results.len() > 1 {
                    println!("{}",
                        Message::set(
                            &format!("Multiple results for `{}`, pick one:", album),
                            MessageKind::Info
                        )
                    );
                    for (i, release) in results.iter().enumerate() {
                        println!(
                            "[{}]: {} - {} ({})",
                            i + 1,
                            release.artist,
                            release.title,
                            format_vec(&release.formats),
                        );
                    }
                    loop {
                        let mut answer = String::new();
                        io::stdin().read_line(&mut answer)
                            .expect("Oops, could not read line.");
                        let choice: usize = match answer.trim().parse() {
                            Ok(num) => num,
                            Err(_) => {println!("{}",
                                Message::set("Invalid input!", MessageKind::Error)
                            ); continue}
                        };
                        if choice <= results.len() {
                            let time_now = utils::get_utc_now();
                            let entry = ListenLogEntry {
                                id: results[choice - 1].id,
                                title: results[choice - 1].title.clone(),
                                time: time_now,
                            };
                            match dbupdate::listenlog(entry) {
                                Ok(()) => {println!("Listening to `{}` by {}", 
                                    results[choice - 1].title, 
                                    results[choice - 1].artist);
                                }
                                Err(e) => {eprintln!("{}", e);}
                            }
                            break;
                            } else {
                            println!("{}",
                                Message::set("Please select a valid choice.", MessageKind::Error)
                            );
                        }
                    }
                } else if results.len() == 1 {
                    let time_now = utils::get_utc_now();
                    let entry = ListenLogEntry {
                        id: results[0].id,
                        title: results[0].title.clone(),
                        time: time_now,
                    };
                    match dbupdate::listenlog(entry) {
                        Ok(()) => {println!("Listening to `{}` by {}", 
                            results[0].title, results[0].artist);}
                        Err(e) => {eprintln!("{}", e);}
                    }
                } else {
                    println!("Unable to find results for `{}`", album);
                }  
            },
            Err(e) => {eprintln!("{}", e);}
        }  
        Some(())
    } else if let Some(sub_m) = clapapp.subcommand_matches("query") {
        let results: Vec<Release>;
        let query: String;
        let querytype: QueryType;
        //TODO: Streamline this wet-ass code
        if sub_m.is_present("wantlist") {
            query = sub_m.value_of("wantlist")
            .unwrap_or_else(|| {
                println!("{} {}", Message::set("Error:", MessageKind::Error), "Album name is required.");
                process::exit(1);
            }).to_string()
            .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], "");

            querytype = QueryType::Wantlist;
            println!("Querying wantlist for: {}", query);
        } else {
            query = sub_m.value_of("albumname")
            .unwrap_or_else(|| {
                println!("{} {}", Message::set("Error:", MessageKind::Error), "Album name is required.");
                process::exit(1);
            }).to_string()
            .replace(&['(', ')', ',', '*', '\"', '.', ':', '!', '?', ';', '\''][..], "");

            querytype = QueryType::Collection;
            println!("Querying collection for: {}\n", query);
        }
        results = query::release(
            query.clone(), querytype
        ).unwrap_or_else(|e| {eprintln!("Oops: {}", e); Vec::new()});
        if results.len() > 1 {
            println!("Multiple results for `{}`:\n", query)
        }
        if results.is_empty() {
            println!("Nothing found for `{}`.", query);
        }
        for release in results {
            let display_time = release.date_added
            .with_timezone(&Config::timezone());

            println!(
                "{} by {}:\nReleased: {}\nLabels: {}\nFormats: {}\nAdded: {}\n",
                release.title,
                release.artist,
                release.year,
                format_vec(&release.labels),
                format_vec(&release.formats),
                display_time.format("%A %d %m %Y %R"),
            )
        }
        Some(())
    } else {
        None
    }
}