#![feature(question_mark)]

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate clap;
extern crate rusqlite;
extern crate chrono;
extern crate ansi_term;

use std::{fs, env, io, cmp, iter};
use std::path::Path;
use std::convert::From;
use std::process::Command;
use clap::{Arg, App, SubCommand};
use rusqlite::Connection;
use chrono::*;
use ansi_term::Style;

#[derive(Debug)]
enum Error {
    ProjectNotFound(String),
    SqliteError(rusqlite::Error),
    Git,
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Error {
        Error::SqliteError(e)
    }
}

fn open_connection() -> Result<Connection, Error> {
    let home = env::var("HOME").unwrap_or("./".into());
    let path = Path::new(&home).join(".timmy");
    if !path.exists() {
        fs::create_dir(&path).unwrap();
    }
    let path = path.join("db.sqlite3");
    let conn = Connection::open(path)?;

    conn.execute_batch("CREATE TABLE IF NOT EXISTS projects (
                            id       INTEGER PRIMARY KEY,
                            name     TEXT NOT NULL UNIQUE,
                            customer TEXT
                        );
                        CREATE TABLE IF NOT EXISTS tags_projects_join (
                            tag_name   TEXT NOT NULL,
                            project_id INTEGER NOT NULL,
                            UNIQUE(tag_name, project_id)
                        );
                        CREATE TABLE IF NOT EXISTS timeperiods (
                            id           INTEGER PRIMARY KEY,
                            project_id   INTEGER NOT NULL,
                            description  TEXT,
                            start        DATETIME NOT NULL,
                            end          DATETIME NOT NULL
                        );
                        CREATE TABLE IF NOT EXISTS commits (
                            sha           TEXT NOT NULL UNIQUE,
                            summary       TEXT NOT NULL,
                            project_id    INTEGER NOT NULL,
                            timeperiod_id INTEGER NOT NULL);")?;
    return Ok(conn);
}


fn create_project(conn: &mut Connection, name: &str, customer: Option<&str>, tags: &str) -> Result<(), Error> {
    let tx = conn.transaction()?;
    let proj_id = tx.execute("INSERT INTO projects(name, customer) VALUES (?,?)", &[&name, &customer])?;
    if tags != "" {
        for tag in tags.split(",") {
            tx.execute("INSERT INTO tags_projects_join VALUES (?, ?)", &[&tag, &proj_id])?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn find_project(conn: &mut Connection, name: &str) -> Result<i32, Error> {
    match conn.query_row("SELECT id FROM projects WHERE name=?", &[&name], |row| row.get(0)) {
        Ok(id) => Ok(id),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::ProjectNotFound(name.into())),
        Err(e) => Err(Error::from(e))
    }
}

fn track(conn: &mut Connection, name: &str, description: Option<&str>) -> Result<(), Error> {
    let proj_id = find_project(conn, name)?;
    let start = Local::now();
    println!("When you are finished with the task press ENTER");

    let mut s = String::new();
    io::stdin().read_line(&mut s).unwrap();

    let end = Local::now();
    conn.execute("INSERT INTO timeperiods(project_id, start, end, description) VALUES (?,?,?,?)", &[&proj_id, &start, &end, &description])?;
    Ok(())
}

fn git(conn: &mut Connection, project: &str) -> Result<(), Error> {
    let proj_id = find_project(conn, project)?;
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM commits WHERE project_id=?", &[&proj_id])?;
    // tx.prepare borrows tx so to call commit stmnt must be dropped
    {
        let mut stmnt = tx.prepare("SELECT id, start, end FROM timeperiods WHERE project_id=?")?;
        let mut rows = stmnt.query(&[&proj_id])?;
        while let Some(row) = rows.next() {
            let row = row?;
            let period_id: i32 = row.get(0);
            let start: DateTime<Local> = row.get(1);
            let end: DateTime<Local> = row.get(2);

            let mut cmd = Command::new("git");
            cmd.arg("whatchanged")
               .arg(format!("--since={}", start.to_rfc3339()))
               .arg(format!("--until={}", end.to_rfc3339()))
               .arg("-q");
            debug!("executing {:?}", cmd);
            let output = cmd.output().map_err(|e|{ error!("{:?}", e); Error::Git})?;

            if !output.status.success() {
                error!("Git error: {}", String::from_utf8_lossy(&output.stderr));
                return Err(Error::Git);
            }
            let s: String = String::from_utf8_lossy(&output.stdout).into_owned();

            let mut lines = s.lines();
            let mut insert_stmnt = tx.prepare("INSERT INTO commits (sha, summary, project_id, timeperiod_id) values(?,?,?,?)")?;

            while let Some(line) = lines.next() {
                if line.starts_with("commit") {
                    let sha = line.split(" ").nth(1).unwrap();
                    lines.next();
                    lines.next();
                    lines.next();
                    let summary = lines.next().unwrap().trim();
                    insert_stmnt.execute(&[&sha, &summary, &proj_id, &period_id])?;
                }
            }
        };
    }

    tx.commit()?;
    Ok(())
}

fn print_border(max_lengths: &[usize], top: bool, joined: bool) {
    let left = match (top, joined) {
        (true, true) => "┌",
        (true, false) => "",
        (false, true) => "├",
        (false, false) => "└"
    };
    let right = match (top, joined) {
        (true, true) => "┐",
        (true, false) => "",
        (false, true) => "┤",
        (false, false) => "┘"

    };
    let middle = match (top, joined) {
        (true, true) => "┬",
        (true, false) => "",
        (false, true) => "┼",
        (false, false) => "┴"
    };
    print!("{}", left);
    for (i, len) in max_lengths.iter().enumerate() {
        let bars: String = iter::repeat("─").take(len+2).collect();
        print!("{}", bars);
        if i == max_lengths.len() - 1 { print!("{}", right); } else { print!("{}", middle); }
    }
    println!("");
}

fn print_row<T>(max_lengths: &[usize], row: T) where T: AsRef<[String]> {
    print!("│");
    for (i, len) in max_lengths.iter().enumerate() {
        let ref cell = row.as_ref()[i];
        let to_pad = len - cell.len();
        let spaces: String = iter::repeat(" ").take(to_pad).collect();
        print!(" {}{} │", cell, spaces);
    }
    println!("");
}

fn print_table<T>(headers: &[String], rows: &[T]) where T: AsRef<[String]> {
    let max_lengths: Vec<usize> = headers.iter().enumerate().map(|(i,v)| {
        let lengths = rows.iter().map(|row| row.as_ref()[i].len());
        cmp::max(lengths.max().unwrap(), v.len())
    }).collect();


    print_border(&max_lengths, true, true);
    print_row(&max_lengths, headers);
    print_border(&max_lengths, false, true);

    for row in rows {
        print_row(&max_lengths, row);
    }
    print_border(&max_lengths, false, false);
}

fn projects(conn: &mut Connection) -> Result<(), Error> {
    let mut projects_stmnt = conn.prepare("SELECT id, name, customer, group_concat(tag_name) FROM projects LEFT JOIN tags_projects_join on project_id=projects.id GROUP BY id;")?;
    let rows = projects_stmnt.query_map(&[], |row| (row.get(0), row.get(1), row.get(2), row.get(3)))?;
    let headers = ["Id".into(), "Name".into(), "Customer".into(), "Tags".into()];
    let mut table = vec![];
    for row in rows {
        let (id, name, customer, tags): (i32, String, Option<String>, Option<String>) = row?;
        table.push([format!("{}", id), name, customer.unwrap_or("".into()), tags.unwrap_or("".into())]);
    }
    print_table(&headers, &table);
    Ok(())
}

fn project(conn: &mut Connection, name: &str) -> Result<(), Error> {
    let (id, customer, tags): (i32, Option<String>, Option<String>) =
        conn.query_row("SELECT id, customer, group_concat(tag_name) FROM projects JOIN tags_projects_join ON project_id=projects.id WHERE name=?",
                       &[&name], |row| (row.get(0), row.get(1), row.get(2)))?;
    let title_style = Style::new().underline().bold();
    print!("{}", title_style.paint(name));
    if customer.is_some() {
        print!("{}", title_style.paint(format!("for {}", customer.unwrap())));
    }
    println!("");
    if let Some(tags) = tags {
        println!("Tags: {}", tags);
    }
    println!("");
    let subtitle_style = Style::new().underline();
    println!("{}", subtitle_style.paint("Recent activity"));

    let mut periods_stmnt = conn.prepare("SELECT id, start, end, description FROM timeperiods WHERE project_id=? ORDER BY start DESC")?;
    let rows = periods_stmnt.query_map(&[&id], |row| (row.get(0), row.get(1), row.get(2), row.get(3)))?;
    for row in rows {
        let (timeperiod_id, start, end, description): (i32, DateTime<Local>, DateTime<Local>, Option<String>) = row?;
        let diff = end - start;
        let time_string = if diff.num_hours() > 0 {
            format!("{}hrs {}mins", diff.num_hours(), diff.num_minutes())
        } else if diff.num_minutes() > 0 {
            format!("{}mins", diff.num_minutes())
        } else {
            format!("{}secs", diff.num_seconds())
        };
        let description_string = if let Some(desc) = description {
            format!(": {}", desc)
        } else {
            "".into()
        };
        let time_fmt = "%H:%M";
        println!("{} {}-{} {}{}", start.format("%a %d %B %Y"), start.format(time_fmt), end.format(time_fmt), time_string, description_string);
        let mut commits_stmnt = conn.prepare("SELECT summary FROM commits WHERE timeperiod_id=?")?;
        let commits = commits_stmnt.query_map(&[&timeperiod_id], |row| (row.get(0)))?;
        for commit in commits {
            let msg: String = commit?;
            println!("    * {}", msg);
        }
    }
    Ok(())
}

fn main() {
    env_logger::init().unwrap();

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
                         .required(true))
                    .arg(Arg::with_name("description")
                         .short("d")
                         .long("description")
                         .help("a description of what you will do in the timeperiod")
                         .takes_value(true)))
        .subcommand(SubCommand::with_name("git")
                    .about("go through each time period and store the commits that happened during that time")
                    .arg(Arg::with_name("PROJECT")
                         .help("the project to assign the commits to")
                         .required(true)))
        .subcommand(SubCommand::with_name("projects")
                    .about("List the projects"))
        .subcommand(SubCommand::with_name("project")
                    .about("Show a project")
                    .arg(Arg::with_name("NAME")
                         .help("the project to show")
                         .required(true)))
        .get_matches();

    let res = if let Some(matches) = matches.subcommand_matches("new") {
        create_project(&mut conn, matches.value_of("NAME").unwrap(), matches.value_of("customer"), matches.value_of("tags").unwrap_or("".into()))
    } else if let Some(matches) = matches.subcommand_matches("track") {
        track(&mut conn, matches.value_of("PROJECT").unwrap(), matches.value_of("description"))
    } else if let Some(matches) = matches.subcommand_matches("git") {
        git(&mut conn, matches.value_of("PROJECT").unwrap())
    } else if let Some(matches) = matches.subcommand_matches("projects") {
        projects(&mut conn)
    } else if let Some(matches) = matches.subcommand_matches("project") {
        project(&mut conn, matches.value_of("NAME").unwrap())
    } else {
        unreachable!();
    };
    match res {
        Ok(()) => {},
        Err(Error::ProjectNotFound(p)) => println!("Project {} not found", p),
        Err(Error::Git) => println!("No git repository found"),
        Err(Error::SqliteError(e)) => {
            println!("There was a problem with the database");
            error!("{:?}", e);
        },
    }
}
