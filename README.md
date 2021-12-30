howdy
-----
howdy is a simple command line tool for your mood monitoring.
Basically, it is capable of two things:
1) asking you "how are you today?", and expecting you to "rate" current day;
2) providing you with reports about your mood dynamics.

### Why?

While this is a very simple (and somewhat stupid) idea, after a year of
rating your days, you can get a decent amount of data. This will let you
to answer some important and not-so-important questions, like:
- what is your favorite weekday/month/season?
- what is your favorite weather?
- how sports activities/gaming/cooking/anything affect your well-being?
- is there anything in your life constantly dropping your "mood balance" below zero?

All this may be obvious, but sometimes I want to verify if my
assumptions about myself are correct. This is where this tool may be handy.

So it is something like maintaining a "mood calendar", or "mood ledger",
with some tools to analyze the data about yourself.

### How?

All records currently maintained in a readable text file (`./howdy.journal` by default).
To add a new record, run:
```
howdy add 1 -c nice day
```
this will add a record into journal file that you rated your day with 1,
and will leave a comment "nice day" to your rate.
Technically you can use any number between -128..127, but the ideal way to use it
is to use just -1,0,1 - try to keep it as simple as possible.

To get a mood report for last month, use:
```
howdy mood
```

More strictly, command syntax looks like this:
```
howdy [-f FILEPATH] add SCORE [TAG ...] [-c COMMENT]
howdy [-f FILEPATH] mood [TAG ...] [-t REPORT_TYPE]
```
Here:

- `FILEPATH` is a path to journal file, defaults to `./howdy.journal`;
- `SCORE` is signed int from -128 to 127;
- `TAG` is a string that can be assigned to a day rate, which can be used
  to filter scores making input in the report;
- `COMMENT` is a string that will be added to a journal to a day rate;
  currently, there is no particular use of it other than making a note for yourself;
- `REPORT_TYPE` is one of the possible report types:
  - `w` or `weekly`: sum up daily scores in Mon-Sun intervals and display those;
  - `7d` or `7 days`: sum up daily scores in 7 days intervals (including today) and display those;
  - `m` or `monthly`: sum up daily scores for monthly intervals and display those;
  - `30d` or `30 days`: sum up daily scores in 30 days intervals (including today) and display those;
  - `lm` or `last month`: sum up daily scores for last 30 days and display it;
(if no report type is specified, the `monthly` option is considered);
  - `y` or `yearly`: sum up daily score for last 365 days and display it;
  - `mm` or `moving`: display 30 monthly reports for last 30 days.

#### Examples:

Rate the current day with `1`, mark it with `sports` and `friends` tags,
and attach a note `"good party with friends"` to this record:

```
howdy -f ./howdy.journal add 1 sports friends --comment good party with friends
```

Rate the current day with `-1`, attach a note `"42"` to this record:

```
howdy add -1 -c 42
```

Read records from `./howdy.journal` over the last month,
and display total summary score:

```
howdy mood
```

Read records from `./howdy.journal` over the last year that have a tag `sports`,
and display total summary score:

```
howdy mood sports -t yearly
```
  
### Potential enhancements?

- add exporting to xlsx;
- add exporting to gnu plot;
- add GUI (let's be honest, no one wants to type a command with args in terminal
just record a digit);
- work on a more advanced querying syntax;

