# Timmy
Timmy is a time tracker.

## Example usage

```
matt@box:~/$ # Creating a new project called timmy with the customer 'me' and tags 'rust,cli'
matt@box:~/$ timmy new timmy -c me -t rust,cli
matt@box:~/$
matt@box:~/$
matt@box:~/$ # View all projects
matt@box:~/$ timmy projects
┌────┬───────┬──────────┬──────────┐
│ Id │ Name  │ Customer │ Tags     │
├────┼───────┼──────────┼──────────┤
│ 1  │ timmy │ me       │ cli,rust │
└────┴───────┴──────────┴──────────┘
matt@box:~/$
matt@box:~/$
matt@box:~/$ # Project view showing total time and activity taken from git commits
matt@box:~/$ timmy project timmy
timmy
Tags: rust,cli
Time spent: 10hrs 20mins

Activity
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
matt@box:~/$
matt@box:~/$
matt@box:~/$ # Weeks view
matt@box:~/$ timmy weeks timmy
┌──────────┬───────┬─────────────┐
│ Week     │ Day   │ Time        │
├──────────┼───────┼─────────────┤
│ 22/08/16 │ Mon   │ 3hrs 47mins │
│          ├───────┼─────────────┤
│          │ Total │ 3hrs 47mins │
├──────────┼───────┼─────────────┤
│ 15/08/16 │ Fri   │ 55mins      │
│          │ Sat   │ 2hrs 40mins │
│          │ Sun   │ 2hrs 57mins │
│          ├───────┼─────────────┤
│          │ Total │ 6hrs 33mins │
└──────────┴───────┴─────────────┘
```


## Screencast
[![asciicast](https://asciinema.org/a/83425.png)](https://asciinema.org/a/83425)
