extern crate clap;
extern crate rusqlite;
extern crate chrono;

use std::{fs, env, io};
use std::path::Path;
use clap::{Arg, App, SubCommand};
use rusqlite::Connection;
use chrono::*;

fn open_connection() -> Result<Connection, rusqlite::Error> {
    let home = env::var("HOME").unwrap_or("./".into());
    let path = Path::new(&home).join(".timmy");
    if !path.exists() {
        fs::create_dir(&path).unwrap();
    }
    let path = path.join("db.sqlite3");
    let conn = try!(Connection::open(path));

    try!(conn.execute_batch("CREATE TABLE IF NOT EXISTS projects (
                               id       INTEGER PRIMARY KEY,
                               name     TEXT NOT NULL UNIQUE,
                               customer TEXT
                             );
                             CREATE TABLE IF NOT EXISTS tags_projects_join (
                               tag_name   TEXT NOT NULL,
                               project_id INTEGER NOT NULL,
                               UNIQUE(tag_name, project_id)
                             );
                             CREATE TABLE IF NOT EXISTS timeperiod (
                               id           INTEGER PRIMARY KEY,
                               project_id   INTEGER NOT NULL,
                               start        DATETIME NOT NULL,
                               end          DATETIME NOT NULL
                             );"));
    return Ok(conn);
}


fn create_project(conn: &mut Connection, name: &str, customer: Option<&str>, tags: &str) -> Result<(), rusqlite::Error> {
    let tx = try!(conn.transaction());
    let proj_id = try!(tx.execute("INSERT INTO projects(name, customer) VALUES (?,?)", &[&name, &customer]));
    if tags != "" {
        for tag in tags.split(",") {
            try!(tx.execute("INSERT INTO tags_projects_join VALUES (?, ?)", &[&tag, &proj_id]));
        }
    }
    tx.commit()
}

fn track(conn: &mut Connection, name: &str) -> Result<(), rusqlite::Error> {
    let start = Local::now();
    let proj_id: i32 = try!(conn.query_row("SELECT id FROM projects WHERE name=?", &[&name], |row| row.get(0)));
    println!("When you are finished with the task press ENTER");

    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();

    let end = Local::now();
    try!(conn.execute("INSERT INTO timeperiod(project_id, start, end) VALUES (?,?,?)", &[&proj_id, &start, &end]));
    Ok(())
}

fn main() {
    let mut conn = open_connection().unwrap();
    let matches =
        App::new("Timmy")
            .version("0.1")
            .author("Matthew Hall")
            .about("Time tracker")
            .subcommand(SubCommand::with_name("new")
                        .about("Creates a new project")
                        .arg(Arg::with_name("NAME")
                             .help("the project name")
                             .required(true))
                        .arg(Arg::with_name("customer")
                             .short("c")
                             .long("customer")
                             .takes_value(true))
                        .arg(Arg::with_name("tags")
                             .short("t")
                             .long("tags")
                             .help("comma separated list of tags")
                             .takes_value(true)))
        .subcommand(SubCommand::with_name("track")
                    .about("Start tracking a time period")
                    .arg(Arg::with_name("PROJECT")
                         .help("the project to start tracking time for")
                         .required(true)))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        create_project(&mut conn, matches.value_of("NAME").unwrap(), matches.value_of("customer"), matches.value_of("tags").unwrap_or("".into())).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("track") {
        track(&mut conn, matches.value_of("PROJECT").unwrap()).unwrap();
    }
}
