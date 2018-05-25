use clap::App;
use clap::Arg;

use std::path::Path;

use color::palette::Palette;
use commands::common::Command;
use commands::common::exit_codes;
use config::read::read_workspace_file;
use crate_info::crate_author;
use crate_info::crate_description;
use crate_info::crate_name;
use crate_info::crate_version;


pub fn main() -> i32 {

    let chdir_arg = Arg::with_name("dir")
        .short("C")
        .long("chdir")
        .help("Change to <dir> before doing anything")
        .takes_value(true)
    ;

    let matches = App::new(crate_name())
        .version(crate_version())
        .about(crate_description())
        .author(crate_author())

        .arg(chdir_arg)

        .subcommand(super::clone::subcommand_def())
        .subcommand(super::status::subcommand_def())
        .subcommand(super::update::subcommand_def())

        .get_matches();

    if let Some(chdir_arg) = matches.args.get("dir") {
        ::std::env::set_current_dir(
            Path::new(
                chdir_arg.vals[0]
                    .to_str()
                    .expect("Did not understand <dir> argument")
            )
        ).unwrap();
    }

    let palette = Palette::default();

    let subcommand: Box<Command> = match &matches.subcommand {
        None => Box::new(super::status::make_command(&matches)),
        Some(sc) => match sc.name.as_ref() {
            "clone" => Box::new(super::clone::make_command(&sc.matches)),
            "status" => Box::new(super::status::make_command(&sc.matches)),
            "update" => Box::new(super::update::make_command(&sc.matches)),
            _ => panic!("Unknown subcommand: {}", sc.name),
        },
    };

    let ws_file_path = Path::new(".projects.gws");
    if ws_file_path.exists() {
        match read_workspace_file(ws_file_path) {
            Ok(ws) => {
                match subcommand.run(ws, &palette) {
                    Ok(status) => status,
                    Err(_) => exit_codes::UNKNOWN_ERROR,
                }
            },
            Err(_) => exit_codes::BAD_PROJECTS_FILE,
        }
    } else {
        exit_codes::NO_PROJECTS_FILE
    }
}