#![feature(question_mark)]

extern crate timmy;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate clap;
extern crate rusqlite;
extern crate chrono;
extern crate ansi_term;
extern crate regex;

use std::{fs, env, io, time, thread};
use std::process::Stdio;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};
use std::path::Path;
use std::convert::From;
use std::process::Command;
use clap::{Arg, App, SubCommand};
use rusqlite::{Connection, Statement};
use regex::Regex;
use chrono::*;
use ansi_term::Style;
use timmy::tables::*;
use timmy::chronny;

#[derive(Debug)]
enum Error {
    ProjectNotFound(String),
    ProjectAlreadyExists(String),
    Sqlite(rusqlite::Error),
    Git,
    InvalidDateTime(String),
    InactiveProject(String),
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Error {
        Error::Sqlite(e)
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
                            timeperiod_id INTEGER NOT NULL);

                       CREATE TABLE IF NOT EXISTS program_usage (
                            project_id    INTEGER NOT NULL,
                            program       TEXT NOT NULL,
                            time          INTEGER NOT NULL);")?;

    let _ = conn.execute("ALTER TABLE projects ADD COLUMN active BOOLEAN NOT NULL DEFAULT 1;", &[]);
    Ok(conn)
}

fn format_time(time: f64) -> String {
    if time > 1.0 {
        format!("{}hrs {}mins",
                time.floor(),
                (60.0 * (time - time.floor())).floor())
    } else if time > 0.0 {
        format!("{}mins", (time * 60.0).floor())
    } else {
        format!("None")
    }
}

fn create_project(conn: &mut Connection,
                  name: &str,
                  customer: Option<&str>,
                  tags: &str)
                  -> Result<(), Error> {
    match find_project(conn, name) {
        Ok(_) => return Err(Error::ProjectAlreadyExists(name.into())),
        _ => {},
    };
    let tx = conn.transaction()?;
    tx.execute("INSERT INTO projects(name, customer) VALUES (?,?)",
               &[&name, &customer])?;
    let proj_id = tx.last_insert_rowid();
    if tags != "" {
        for tag in tags.split(',') {
            let _ = tx.execute("INSERT INTO tags_projects_join VALUES (?, ?)",
                               &[&tag, &proj_id]);
        }
    }
    tx.commit()?;
    Ok(())
}

fn finish_project(conn: &mut Connection, name: &str) -> Result<(), Error> {
    let (proj_id, _) = find_project(conn, name)?;
    conn.execute("UPDATE projects SET active=0 WHERE id=?", &[&proj_id])?;
    Ok(())
}

fn find_project(conn: &mut Connection, name: &str) -> Result<(i64, bool), Error> {
    match conn.query_row("SELECT id, active FROM projects WHERE name=?",
                         &[&name],
                         |row| (row.get(0), row.get(1))) {
        Ok((id, active)) => Ok((id, active)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::ProjectNotFound(name.into())),
        Err(e) => Err(Error::from(e)),
    }
}

fn get_current_program() -> String {
    // Get the X id for the currently displayed window
    let output = Command::new("xprop").args(&["-root", "_NET_ACTIVE_WINDOW"]).output().unwrap();
    // parse: _NET_ACTIVE_WINDOW(WINDOW): window id # 0x3e0000a
    let output = String::from_utf8_lossy(&output.stdout);
    let win_id = output.split(' ').last().unwrap();
    // Get the pid
    let output = Command::new("xprop").args(&["-id", &format!("{}", win_id)]).output().unwrap();
    let output = String::from_utf8_lossy(&output.stdout);
    let regex = Regex::new(r"_NET_WM_PID\(CARDINAL\) = (\d+)").unwrap();
    let caps = regex.captures(&output).unwrap();
    let pid = caps.at(1).unwrap();
    let output = Command::new("ps").args(&["-ocomm=", &format!("-p{}", pid)]).output().unwrap();
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn program_tracker_thread(rx: Receiver<bool>)
                          -> Option<thread::JoinHandle<HashMap<String, i64>>>
{
    let status = Command::new("xprop").arg("-root").stdout(Stdio::null()).status();
    match status {
        Ok(_) => {},
        Err(_) => return None
    }
    let handle = thread::spawn(move || {
        let mut hm = HashMap::new();
        let mut current_program = get_current_program().trim().into();
        let mut program_change_time = Local::now();
        while rx.try_recv().is_err() {
            let new_program = get_current_program().trim().into();
            if new_program != current_program {
                let diff = Local::now() - program_change_time;
                program_change_time = Local::now();
                let counter = hm.entry(current_program).or_insert(0);
                *counter += diff.num_seconds();
                current_program = new_program;
            }
            thread::sleep(time::Duration::from_millis(100));
        }
        return hm;
    });
    Some(handle)
}

fn track(conn: &mut Connection,
         name: &str,
         description: Option<&str>,
         start: Option<&str>,
         end: Option<&str>,
         no_program: bool) -> Result<(), Error> {
    let (proj_id, active) = find_project(conn, name)?;
    if !active {
        return Err(Error::InactiveProject(name.into()));
    }
    let start = if let Some(start) = start {
        chronny::parse_datetime(start, Local::now()).ok_or(Error::InvalidDateTime(start.into()))?
    } else {
        Local::now()
    };
    println!("Starting at {}", start.format("%d/%m/%y %H:%M"));
    let (end, times) = if let Some(end) = end {
        (chronny::parse_datetime(end, Local::now()).ok_or(Error::InvalidDateTime(end.into()))?,
         HashMap::new())
    } else {
        let (tx, rx) = channel();
        let handle = program_tracker_thread(rx);
        println!("When you are finished with the task press ENTER");
        let mut s = String::new();
        io::stdin().read_line(&mut s).unwrap();
        let _ = tx.send(true);
        let mut times = HashMap::new();
        if let Some(handle) = handle {
            if !no_program {
                times = handle.join().expect("couldn't join program tracker thread");
            }
        }
        debug!("program times: {:?}", times);
        (Local::now(), times)
    };
    println!("Ending at {}", end.format("%d/%m/%y %H:%M"));

    let tx = conn.transaction()?;
    tx.execute("INSERT INTO timeperiods(project_id, start, end, description) VALUES (?,?,?,?)",
                &[&proj_id, &start, &end, &description])?;
    let period_id = tx.last_insert_rowid();
    {
        let mut stmnt = tx.prepare("INSERT INTO commits (sha, summary, project_id, timeperiod_id) \
                                    values(?,?,?,?)")?;
        match get_commits(&mut stmnt, proj_id, period_id, &start, &end) {
            Ok(()) => {},
            Err(Error::Git) => println!("Git either isn't installed or there is no repo in the \
                                         current working directory. To associate commits with this \
                                         project run `timmy git <project>` in a directory with a \
                                         git repo."),
            e => return e,
        };
        let mut stmnt = tx.prepare("INSERT INTO program_usage(project_id, program, time) VALUES (?,?,?)")?;
        for (program, time) in &times {
            let old_time: i64 = tx.query_row("SELECT time FROM program_usage WHERE project_id=? AND program=?",
                                             &[&proj_id, program],
                                             |row| row.get(0))
                                  .unwrap_or(0);
            tx.execute("DELETE FROM program_usage WHERE project_id=? AND program=?", &[&proj_id, program])?;
            let time = time + old_time;
            stmnt.execute(&[&proj_id, program, &time])?;
        }
    }
    tx.commit()?;
    Ok(())
}

fn get_commits(insert_stmnt: &mut Statement, proj_id: i64, period_id: i64, start: &DateTime<Local>, end: &DateTime<Local>) -> Result<(), Error> {
    let mut cmd = Command::new("git");
    cmd.arg("whatchanged")
        .arg(format!("--since={}", start.to_rfc3339()))
        .arg(format!("--until={}", end.to_rfc3339()))
        .arg("-q");
    debug!("executing {:?}", cmd);
    let output = cmd.output()
        .map_err(|e| {
            debug!("{:?}", e);
            Error::Git
        })?;

    if !output.status.success() {
        debug!("Git error: {}", String::from_utf8_lossy(&output.stderr));
        return Err(Error::Git);
    }
    let s: String = String::from_utf8_lossy(&output.stdout).into_owned();

    let mut lines = s.lines();

    while let Some(line) = lines.next() {
        // parses the following:

        // commit f04a366b0da4377b2f1e87dc9ec68bdf68c24cee
        // Author: Matthew Hall <matthew@quickbeam.me.uk>
        // Date:   Sun Aug 21 15:00:43 2016 +0100
        //
        //     Add total time to project view

        if line.starts_with("commit") {
            let sha = line.split(' ').nth(1).unwrap();
            debug!("{}", sha);
            // skip author
            lines.next();
            // skip date
            lines.next();
            // skip newline
            lines.next();
            // parse summary
            let summary = lines.next().unwrap().trim();
            println!("Found commit {}: {}", sha, summary);
            insert_stmnt.execute(&[&sha, &summary, &proj_id, &period_id])?;
        }
    }
    Ok(())
}

fn git(conn: &mut Connection, project: &str) -> Result<(), Error> {
    let (proj_id, active) = find_project(conn, project)?;
    if !active {
        return Err(Error::InactiveProject(project.into()));
    }
    let tx = conn.transaction()?;

    tx.execute("DELETE FROM commits WHERE project_id=?", &[&proj_id])?;

    // tx.prepare borrows tx so to call commit stmnt must be dropped
    {
        let mut stmnt = tx.prepare("SELECT id, start, end FROM timeperiods WHERE project_id=?")?;
        let mut rows = stmnt.query(&[&proj_id])?;
        let mut insert_stmnt = tx.prepare("INSERT INTO commits (sha, summary, project_id, timeperiod_id) \
                                           values(?,?,?,?)")?;
        while let Some(row) = rows.next() {
            let row = row?;
            let period_id: i64 = row.get(0);
            let start: DateTime<Local> = row.get(1);
            let end: DateTime<Local> = row.get(2);
            get_commits(&mut insert_stmnt, proj_id, period_id, &start, &end)?;
        }
    }

    tx.commit()?;
    Ok(())
}

fn projects(conn: &mut Connection, all: bool) -> Result<(), Error> {
    let mut projects_stmnt = if all {
        conn.prepare("SELECT name, customer, group_concat(tag_name), active FROM projects
                      LEFT JOIN tags_projects_join on project_id=projects.id
                      GROUP BY id;")?
    } else {
        conn.prepare("SELECT name, customer, group_concat(tag_name), active FROM projects
                      LEFT JOIN tags_projects_join on project_id=projects.id
                      WHERE active=1
                      GROUP BY id;")?
    };
    let rows =
        projects_stmnt.query_map(&[], |row| (row.get(0), row.get(1), row.get(2), row.get(3)))?;
    let mut headers = vec!["Name".into(), "Customer".into(), "Tags".into()];
    if all { headers.push("Active".into()); }
    let mut table = Table::with_headers(headers);
    for row in rows {
        let (name, customer, tags, active): (String, Option<String>, Option<String>, bool) = row?;
        let mut row = vec![name, customer.unwrap_or("".into()), tags.unwrap_or("".into())];
        if all { row.push(format!("{}", active)); }
        table.add_simple(row);
    }
    table.add_border_bottom();
    table.print();
    Ok(())
}

fn print_activity(conn: &mut Connection, id: i64, week: bool, since: Option<&str>, until: Option<&str>) -> Result<(), Error> {
    let mut since = if let Some(since) = since {
        debug!("{}", since);
        chronny::parse_datetime(since, Local::now()).ok_or(Error::InvalidDateTime(since.into()))?
    } else {
        Local::now().with_year(1).unwrap()
    };
    let until = if let Some(until) = until {
        debug!("{}", until);
        chronny::parse_datetime(until, Local::now()).ok_or(Error::InvalidDateTime(until.into()))?
    } else {
        Local::now()
    };
    if week {
        since = Local::now() - Duration::days(7);
    }
    debug!("printing activity between {:?} and {:?}", since, until);
    let mut periods_stmnt =
        conn.prepare("SELECT id, start, end, description,
                             CAST((julianday(end)-julianday(start))*24 AS REAL)
                      FROM timeperiods
                      WHERE project_id=? AND start > ? AND start < ?
                      ORDER BY start DESC")?;
    let rows = periods_stmnt.query_map(&[&id, &since, &until],
                   |row| (row.get(0), row.get(1), row.get(2), row.get(3), row.get(4)))?;

    let subtitle_style = Style::new().underline();
    println!("{}", subtitle_style.paint("Activity"));

    let mut total = 0.0f64;
    for row in rows {
        let (timeperiod_id, start, end, description, time): (i64,
                                                             DateTime<Local>,
                                                             DateTime<Local>,
                                                             Option<String>,
                                                             f64) = row?;
        total += time;
        let time_string = format_time(time);
        let description_string = if let Some(desc) = description {
            format!(": {}", desc)
        } else {
            "".into()
        };
        let time_fmt = "%H:%M";
        println!("{} {}-{} {}{}",
                 start.format("%a %d %B %Y"),
                 start.format(time_fmt),
                 end.format(time_fmt),
                 time_string,
                 description_string);

        let mut commits_stmnt = conn.prepare("SELECT summary FROM commits WHERE timeperiod_id=?")?;
        let commits = commits_stmnt.query_map(&[&timeperiod_id], |row| (row.get(0)))?;
        for commit in commits {
            let msg: String = commit?;
            println!("    * {}", msg);
        }
    }
    println!("Total: {}", format_time(total));
    Ok(())
}

fn print_project_summary(conn: &mut Connection,
                         id: i64,
                         name: &str,
                         customer: Option<String>,
                         tags: Option<String>)
                         -> Result<(), Error>
{
    let title_style = Style::new().underline().bold();
    print!("{}", title_style.paint(name));

    if let Some(customer) = customer {
        print!("{}",
               title_style.paint(format!("for {}", customer)));
    }
    println!("");

    if let Some(tags) = tags {
        println!("Tags: {}", tags);
    }

    let total_time: Option<f64> =
        conn.query_row("SELECT SUM(CAST((julianday(end)-julianday(start))*24 as REAL))
                        FROM timeperiods WHERE project_id=?",
                       &[&id],
                       |row| row.get(0))?;
    let total_time = total_time.unwrap_or(0.0);
    let total_time_str = format_time(total_time);
    println!("Total time spent: {}", total_time_str);
    println!("");
    Ok(())
}

fn print_program_usage(conn: &mut Connection, id: i64, total_time: Option<i64>) -> Result<(), Error> {
    if let Some(total_time) = total_time {
        let subtitle_style = Style::new().underline();
        println!("{}", subtitle_style.paint("Program usage"));
        let mut stmnt = conn.prepare("SELECT program, time FROM program_usage WHERE project_id=?")?;
        let rows = stmnt.query_map(&[&id], |row| (row.get(0), row.get(1)))?;
        for row in rows {
            let (program, time): (String, i64) = row?;
            let pc: f32 = (time as f32) / (total_time as f32) * 100f32;
            debug!("pc, time, total_time: {} {} {}", pc, time, total_time);
            println!("{:>5.2}% {}", pc, program);
        }
        println!("");
    }
    Ok(())
}

fn project(conn: &mut Connection,
           name: &str,
           week: bool,
           since: Option<&str>,
           until: Option<&str>)
           -> Result<(), Error>
{
    let (id, customer, tags, time): (i64, Option<String>, Option<String>, Option<i64>) =
        conn.query_row("SELECT id, customer, group_concat(tag_name),
                               (SELECT SUM(time) FROM program_usage WHERE project_id=id)
                        FROM projects
                        LEFT JOIN tags_projects_join ON tags_projects_join.project_id=projects.id
                        WHERE name=?",
                       &[&name],
                       |row| {
                           let id: Option<i64> = row.get(0);
                           if let None = id {
                               return Err(Error::ProjectNotFound(name.into()));
                           }
                           Ok((row.get(0), row.get(1), row.get(2), row.get(3)))
                       })??;
    print_project_summary(conn, id, name, customer, tags)?;
    print_program_usage(conn, id, time)?;
    print_activity(conn, id, week, since, until)
}

fn weeks(conn: &mut Connection, name: &str) -> Result<(), Error> {
    let (project_id, _) = find_project(conn, name)?;
    let mut day_stmnt =
        conn.prepare("SELECT start,
                             SUM(CAST((julianday(end)-julianday(start))*24 AS REAL))
                      FROM timeperiods
                      WHERE project_id=?
                      GROUP BY strftime('%j', start)
                      ORDER BY strftime('%Y%W', start) DESC, start")?;
    let rows = day_stmnt.query_map(&[&project_id], |row| (row.get(0), row.get(1)))?;
    let mut week = 0;
    let mut year = 0;
    let mut start_of_week = NaiveDate::from_isoywd(1, 1, Weekday::Mon);
    let mut table = Table::with_headers(vec!["Week".into(), "Day".into(), "Time".into()]);
    let mut total_time = -1.0;
    let total_separator = vec![Cell::new_left_bordered(CellType::Data("".into()), "│"),
                               Cell::new_left_bordered(CellType::Separator, "├"),
                               Cell::new_both_bordered(CellType::Separator, "┼", "┤")];
    for row in rows {
        let (start, time): (DateTime<Local>, f64) = row?;
        let (y,w,_) = start.isoweekdate();
        let time_str = format_time(time);
        let week_str = if w != week || y != year {
            week = w;
            year = y;
            start_of_week = NaiveDate::from_isoywd(y, w, Weekday::Mon);
            if total_time >= 0.0 {
                table.add_row(total_separator.clone());
                table.add_simple(vec!["".into(), "Total".into(), format_time(total_time)]);
                table.add_full_separator();
            }
            total_time = 0.0;
            format!("{}", start_of_week.format("%d/%m/%y"))
        } else {
            "".into()
        };
        total_time += time;
        table.add_simple(vec![week_str, format!("{}", start.format("%a")), time_str]);
    }
    table.add_row(total_separator.clone());
    table.add_simple(vec!["".into(), "Total".into(), format_time(total_time)]);
    table.add_border_bottom();
    table.print();
    Ok(())
}

fn short_weeks(conn: &mut Connection, name: &str) -> Result<(), Error> {
    let (project_id, _) = find_project(conn, name)?;
    let mut weeks_stmnt =
        conn.prepare("SELECT start,
                             SUM(CAST((julianday(end)-julianday(start))*24 AS REAL))
                      FROM timeperiods
                      WHERE project_id=?
                      GROUP BY strftime('%W', start)
                      ORDER BY strftime('%Y%W', start) DESC")?;
    let rows = weeks_stmnt.query_map(&[&project_id], |row| (row.get(0), row.get(1)))?;
    for row in rows {
        let (start, time): (DateTime<Local>, f64) = row?;
        let (y,w,_) = start.isoweekdate();
        let start_of_week = NaiveDate::from_isoywd(y, w, Weekday::Mon);
        let end_of_week = NaiveDate::from_isoywd(y, w, Weekday::Sun);
        let time_str = format_time(time);
        println!("{}-{}\t{}", start_of_week.format("%d/%m/%y"), end_of_week.format("%d/%m/%y"), time_str);
    }
    Ok(())
}

fn main() {
    env_logger::init().unwrap();

    let mut conn = open_connection().unwrap();
    let matches = App::new("Timmy")
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
        .subcommand(SubCommand::with_name("finish")
           .about("Makes a project inactive")
            .arg(Arg::with_name("NAME")
                    .help("the project name")
                    .required(true)))
        .subcommand(SubCommand::with_name("track")
            .about("Start tracking a time period")
            .arg(Arg::with_name("PROJECT")
                .help("the project to start tracking time for")
                .required(true))
            .arg(Arg::with_name("description")
                .short("d")
                .long("description")
                .help("a description of what you will do in the timeperiod")
                .takes_value(true))
            .arg(Arg::with_name("start")
                 .short("s")
                 .long("start")
                 .help("When to track from")
                 .takes_value(true))
            .arg(Arg::with_name("end")
                 .short("e")
                 .long("end")
                 .help("When to end")
                 .takes_value(true)
                 .requires("start"))
            .arg(Arg::with_name("no program")
                 .short("n")
                 .long("noprogram")
                 .help("Don't track program usage")))
        .subcommand(SubCommand::with_name("git")
            .about("go through each time period and store the commits that happened during that \
                    time. timmy track automatically does this when you quit it for that \
                    time period. This command is useful if you've modified your git history \
                    in some way or you ran timmy track in the wrong directory.")
            .arg(Arg::with_name("PROJECT")
                .help("the project to assign the commits to")
                .required(true)))
        .subcommand(SubCommand::with_name("projects")
            .about("List the projects")
            .arg(Arg::with_name("all")
                .help("Show all projects, including inactive ones")
                .short("a")
                .long("all")))
        .subcommand(SubCommand::with_name("project")
            .about("Show a project")
            .arg(Arg::with_name("NAME")
                .help("the project to show")
                .required(true))
            .arg(Arg::with_name("since")
                 .short("s")
                 .long("since")
                 .help("the date and time from which to show activity")
                 .takes_value(true))
            .arg(Arg::with_name("until")
                 .short("u")
                 .long("until")
                 .help("the date and time until which to show activity")
                 .takes_value(true))
            .arg(Arg::with_name("week")
                 .short("w")
                 .long("week")
                 .help("show activity in the past week")
                 .conflicts_with_all(&["since", "until"])))
        .subcommand(SubCommand::with_name("weeks")
            .about("show time spent per week")
            .arg(Arg::with_name("PROJECT")
                .help("the project to show")
                .required(true))
            .arg(Arg::with_name("short")
                 .long("short")
                 .help("show the short view")))
        .get_matches();

    let res = if let Some(matches) = matches.subcommand_matches("new") {
        create_project(&mut conn,
                       matches.value_of("NAME").unwrap(),
                       matches.value_of("customer"),
                       matches.value_of("tags").unwrap_or("".into()))
    } else if let Some(matches) = matches.subcommand_matches("finish") {
        finish_project(&mut conn, matches.value_of("NAME").unwrap())
    } else if let Some(matches) = matches.subcommand_matches("track") {
        track(&mut conn,
              matches.value_of("PROJECT").unwrap(),
              matches.value_of("description"),
              matches.value_of("start"),
              matches.value_of("end"),
              matches.is_present("no program"))
    } else if let Some(matches) = matches.subcommand_matches("git") {
        git(&mut conn, matches.value_of("PROJECT").unwrap())
    } else if let Some(matches) = matches.subcommand_matches("projects") {
        projects(&mut conn, matches.is_present("all"))
    } else if let Some(matches) = matches.subcommand_matches("project") {
        project(&mut conn,
                matches.value_of("NAME").unwrap(),
                matches.is_present("week"),
                matches.value_of("since"),
                matches.value_of("until"))
    } else if let Some(matches) = matches.subcommand_matches("weeks") {
        if matches.is_present("short") {
            short_weeks(&mut conn, matches.value_of("PROJECT").unwrap())
        } else {
            weeks(&mut conn, matches.value_of("PROJECT").unwrap())
        }
    } else {
        unreachable!();
    };
    match res {
        Ok(()) => {}
        Err(Error::ProjectNotFound(p)) => println!("Project {} not found", p),
        Err(Error::ProjectAlreadyExists(p)) => println!("Project {} already exists", p),
        Err(Error::Git) => println!("No git repository found"),
        Err(Error::Sqlite(e)) => {
            println!("There was a problem with the database");
            debug!("{:?}", e);
        },
        Err(Error::InvalidDateTime(s)) => println!("Could not parse {}", s),
        Err(Error::InactiveProject(p)) => println!("Project {} is inactive", p),
    }
}
