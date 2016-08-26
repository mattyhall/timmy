# Timmy
[Timmy](https://crates.io/crates/timmy) is a time tracker. At the moment it is not particularly stable or even well written! Things like parsing dates in english (eg. "yesterday 12:00") may be buggy. If you do want to install it run:

```
matt@box:~/$ cargo install timmy
```

and make sure `~/.cargo/bin` or equivalent is in your PATH.

## Example usage

Creating a new project can be done with the new command. Tags are comma separated.

```
matt@box:~/$ # Creating a new project called timmy with the customer 'me' and tags 'rust,cli'
matt@box:~/$ timmy new timmy -c me -t rust,cli
```

You can get a list of projects like so:

```
matt@box:~/$ timmy projects
┌───────┬──────────┬──────────┐
│ Name  │ Customer │ Tags     │
├───────┼──────────┼──────────┤
│ timmy │ me       │ cli,rust │
└───────┴──────────┴──────────┘
```

You can start tracking using timmy track. You can optionally add a start point or a start and end point to add some time that you forgot to track.

```
matt@box:~/$ timmy track <project>
matt@box:~/$ # Starting time 
matt@box:~/$ timmy track <project> -s "12:00" 
matt@box:~/$ # Starting time and end
matt@box:~/$ timmy track <project> -s "12:00" -e "13:00"
```

At the end of a session timmy will automatically look for any git commits in the repo in the current directory. If you edit history (eg. reverting a commit) you can run:

```
matt@box:~/$ timmy git <project>
```

and it will repopulate the commits for the project.

To get all information on a particular project use the project command:

```
matt@box:~/$ timmy project timmy
timmy
Tags: rust,cli
Total time spent: 14hrs 31mins

Activity
Thu 25 August 2016 17:55-18:02 6mins
    * Print what commits have been found
    * Add total time to activity printout
Thu 25 August 2016 17:44-17:54 9mins
    * Add week flag to show activity in last week
Thu 25 August 2016 17:18-17:27 9mins
    * Add support for dates like 20/08 to chronny
    * Fix 20/08 date not being parsed
Thu 25 August 2016 16:32-17:15 42mins
    * Add filtering activity by dates
    * Add absolute dates to chronny
    * Remove id from projects table
Wed 24 August 2016 14:09-14:30 21mins
    * Add start and end options to track
    * Add kitchen sink test for chronny
Tue 23 August 2016 20:35-21:24 48mins
    * Add relative times
Tue 23 August 2016 15:47-17:12 1hrs 25mins
    * Start writing a human datetime parsing lib
    * Use i64 for row ids
    * Automatically look for commits at the end of track
Tue 23 August 2016 15:28-15:46 18mins
Mon 22 August 2016 21:18-21:27 8mins: Example usage
    * Extend readme with example usage
Mon 22 August 2016 20:30-21:04 34mins
    * Add short weeks view
    * Use debug! instead of error!
    * Rename week command to weeks
    * Stop projects panicing when project is not found
Mon 22 August 2016 15:38-17:47 2hrs 9mins
    * Add total separator to week view
    * Use tables lib in timmy
    * Remove double bordering from tables
    * First attempt at a tables lib
Mon 22 August 2016 14:50-15:33 42mins
    * Add total to weeks view
    * Move formatting a time difference into a function
    * First version of weeks view
Mon 22 August 2016 14:20-14:41 20mins
Sun 21 August 2016 18:22-18:25 2mins: bugfixing
    * Make total time display correct
Sun 21 August 2016 17:04-18:20 1hrs 15mins: Upload to github
    * Add readme
    * Make project view work with no timeperiods or tags
Sun 21 August 2016 15:49-16:03 13mins
    * Remove abc
    * Refactor project view
    * Fix clippy warnings
    * Rename SqliteError to Sqlite
Sun 21 August 2016 15:23-15:48 24mins: Clean up
Sun 21 August 2016 14:50-15:03 12mins: Total time on project vew
    * Add total time to project view
Sun 21 August 2016 14:35-14:46 11mins: Fix projects list
    * Fix projects view
Sun 21 August 2016 13:48-14:24 35mins: Project view
    * Add tags to project view
    * Move getting tags into query
    * Add project view
Sat 20 August 2016 21:28-21:42 13mins: Project view
Sat 20 August 2016 20:06-20:47 41mins: Views
    * Refactor printing a row into a function
    * Implement projects list
Sat 20 August 2016 18:39-19:21 42mins: Views
Sat 20 August 2016 14:50-14:57 6mins: Test sqlites support for times
Sat 20 August 2016 13:45-13:59 13mins
    * Use question mark instead of try!
    * Add description to timeperiods
    * Rename timeperiod to timeperiods for consistency
Sat 20 August 2016 13:40-13:41 0mins
Sat 20 August 2016 12:56-13:38 41mins
Fri 19 August 2016 19:18-19:58 39mins
Fri 19 August 2016 18:08-18:23 15mins
    * Use own error type
Total: 14hrs 31mins
```

This is quite long so you can get the activity between certain times or dates like so:

```
matt@box:~/$ timmy project timmy -s 22/08/16 -u 24/08/16
timmy
Tags: rust,cli
Total time spent: 14hrs 31mins

Activity
Wed 24 August 2016 14:09-14:30 21mins
    * Add start and end options to track
    * Add kitchen sink test for chronny
Tue 23 August 2016 20:35-21:24 48mins
    * Add relative times
Tue 23 August 2016 15:47-17:12 1hrs 25mins
    * Start writing a human datetime parsing lib
    * Use i64 for row ids
    * Automatically look for commits at the end of track
Tue 23 August 2016 15:28-15:46 18mins
Mon 22 August 2016 21:18-21:27 8mins: Example usage
    * Extend readme with example usage
Mon 22 August 2016 20:30-21:04 34mins
    * Add short weeks view
    * Use debug! instead of error!
    * Rename week command to weeks
    * Stop projects panicing when project is not found
Total: 3hrs 37mins
```

You can even use English date descriptions like so:

```
matt@box:~/$ timmy project timmy -s "yesterday 12:00"
timmy 
Tags: rust,cli
Total time spent: 14hrs 31mins

Activity
Thu 25 August 2016 17:55-18:02 6mins
    * Print what commits have been found
    * Add total time to activity printout
Thu 25 August 2016 17:44-17:54 9mins
    * Add week flag to show activity in last week
Thu 25 August 2016 17:18-17:27 9mins
    * Add support for dates like 20/08 to chronny
    * Fix 20/08 date not being parsed
Thu 25 August 2016 16:32-17:15 42mins
    * Add filtering activity by dates
    * Add absolute dates to chronny
    * Remove id from projects table
Total: 1hrs 8mins
```

You can get a week by week view of a project as well:

```
matt@box:~/$ timmy weeks timmy
┌──────────┬───────┬─────────────┐
│ Week     │ Day   │ Time        │
├──────────┼───────┼─────────────┤
│ 22/08/16 │ Mon   │ 3hrs 56mins │
│          │ Tue   │ 2hrs 32mins │
│          │ Wed   │ 21mins      │
│          │ Thu   │ 1hrs 8mins  │
│          ├───────┼─────────────┤
│          │ Total │ 7hrs 58mins │
├──────────┼───────┼─────────────┤
│ 15/08/16 │ Fri   │ 55mins      │
│          │ Sat   │ 2hrs 40mins │
│          │ Sun   │ 2hrs 57mins │
│          ├───────┼─────────────┤
│          │ Total │ 6hrs 33mins │
└──────────┴───────┴─────────────┘
```
