use crate::types::{Birthday, Event, Person, Subtask, Task};
use chrono::naive::{NaiveDate, NaiveTime};

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, ErrorKind, Read};
use std::mem;
use std::path::Path;
use std::str::FromStr;
use std::usize;

pub mod tag {
    pub const ITEM: &str = "- ";
    pub const DAY: &str = "## ";
    pub const WEEK: &str = "# Week ";
    pub const TOPLEVEL: &str = "";
    pub const SUB: &str = "  ";
    pub const TODO: &str = "TODO";
    pub const DONE: &str = "DONE";
    pub const EVT: &str = "EVT";
    // event without
    pub const EVT_PLAIN: &str = "EVT: ";
}

const LINE_END_LINUX: &str = "\n";
const LINE_END_WINDOWS: &str = "\r\n";

struct Parser {
    line_end: String,
    unit_ends: [String; 4],
    task_tag_todo: String,
    task_tag_done: String,
    event_tag: String,
    day_tag: String,
}

impl Parser {
    fn from_line_end(line_end: &str) -> Parser {
        let le = line_end.to_owned();
        let task_tag_todo = le.clone() + tag::ITEM + tag::TODO;
        let task_tag_done = le.clone() + tag::ITEM + tag::DONE;
        let event_tag = le.clone() + tag::ITEM + tag::EVT;
        let day_tag = le.clone() + tag::DAY;

        let unit_ends = [
            // terminated with the next top-level list item
            le.clone() + tag::TOPLEVEL + tag::ITEM,
            // terminated with an empty line
            // FIXME: this will break if there is now trailing newline
            le.clone() + line_end,
            // terminated  by the next day
            // FIXME: might go wrong if there is a codeblock in between which contains ##
            day_tag.clone(),
            // terminated at the begin of a week
            le.clone() + tag::WEEK,
        ];

        Parser {
            line_end: le,
            unit_ends,
            task_tag_done,
            task_tag_todo,
            event_tag,
            day_tag,
        }
    }

    pub fn parse_events(&self, log_data: &str) -> io::Result<Vec<Event>> {
        let mut events = vec![];
        for (start, _) in log_data.match_indices(&self.event_tag) {
            // isolate line and skip the leading CRLF
            let start = start + self.line_end.len();
            let (_eol, line) = slice(log_data, start, &self.line_end);

            let date = self.lookup_date(log_data, start)?;

            // parse time (if any)
            let (msg, time) = match line.find("EVT:") {
                // straightforward event (e.g. - EVT:) (skip the first 6 chars)
                Some(_pos) => (line["- EVT:".len()..].trim_start().to_string(), None),
                // event with time ( e.g.- EVT 16:49:)
                // skip to the first number
                None => {
                    let mut hm = line["- EVT ".len()..]
                        .split(|c: char| c.eq(&':'))
                        .map(|n| u32::from_str(n).unwrap());
                    let h = hm.next().unwrap();
                    let m = hm.next().unwrap();

                    let time = NaiveTime::from_hms(h, m, 0);

                    let msg = line
                        .match_indices(':')
                        .skip(1)
                        .map(|(pos, _)| &line[pos + ":".len()..])
                        .map(|msg| msg.trim_start().to_string())
                        .next()
                        .unwrap();

                    (msg, Some(time))
                }
            };

            let start_of_unit = start + self.line_end.len();

            let end_of_unit = start_of_unit + self.lookup_end_of_unit(&log_data[start_of_unit..]);

            let notes = log_data[start_of_unit..end_of_unit]
                .lines()
                .skip(1)
                .filter(|l| !l.is_empty())
                .map(|l| l.trim_start())
                .map(|l| &l[2..])
                .map(|l| l.to_string())
                .collect();

            let event = Event {
                msg,
                notes,
                date,
                time,
            };

            events.push(event);
        }

        Ok(events)
    }

    pub fn parse_tasks(&self, log_data: &str) -> io::Result<Vec<Task>> {
        // find toplevel TODOS
        let mut tasks = vec![];

        let todos = log_data
            .match_indices(&self.task_tag_todo)
            .map(|(idx, _)| (idx, false));
        let dones = log_data
            .match_indices(&self.task_tag_done)
            .map(|(idx, _)| (idx, true));

        for (idx, is_done) in todos.chain(dones) {
            let (todo_start, todo_line, eol) = {
                let ip = idx + self.line_end.len();
                let eol = log_data[ip..].find(&self.line_end).unwrap();
                (ip, &log_data[ip..ip + eol], eol)
            };

            // search backwards from the TODO to find the day
            let date = self.lookup_date(log_data, todo_start)?;

            // search forward from the task
            // to identify the end of the task
            let end_of_todo = self.lookup_end_of_unit(&log_data[todo_start..]);

            let todo_body = &log_data[todo_start + eol..todo_start + end_of_todo];

            let (subtasks, notes) =
                todo_body
                    .lines()
                    .fold((vec![], vec![]), |(mut st, mut n), l| {
                        let l = {
                            let pos = l
                                .find(tag::ITEM)
                                .map(|pos| pos + tag::ITEM.len())
                                .unwrap_or(0);
                            &l[pos..]
                        };

                        if l.is_empty() {
                            return (st, n);
                        }

                        match (l.find(tag::TODO), l.find(tag::DONE)) {
                            (Some(_todo), Some(_done)) => eprintln!(
                                "Found TODO and DONE in {}. A task can either be done or todo.",
                                l
                            ),
                            (Some(_todo), None) => {
                                let s = Subtask {
                                    msg: slice_from(&l, ": ").into(),
                                    is_done: false,
                                };
                                st.push(s);
                            }
                            (None, Some(_done)) => {
                                let s = Subtask {
                                    msg: slice_from(&l, ": ").into(),
                                    is_done: true,
                                };
                                st.push(s);
                            }
                            (None, None) => n.push(l.to_string()),
                        };

                        (st, n)
                    });

            // drop the TODO at the front
            let msg = slice_from(todo_line, ": ").to_owned();

            // check if there are any undone subtasks
            let all_subtasks_done = !subtasks.iter().any(|st| !st.is_done);
            let is_done = is_done && all_subtasks_done;

            let task = Task {
                msg,
                subtasks,
                notes,
                date,
                is_done,
            };

            tasks.push(task);
        }

        Ok(tasks)
    }

    // helpers
    fn lookup_date(&self, s: &str, lookup_from: usize) -> io::Result<NaiveDate> {
        let day_line = {
            let day = s[..lookup_from].rfind(&self.day_tag).unwrap() + 1;
            let eol = s[day..].find(&self.line_end).unwrap();
            &s[day..day + eol]
        };

        // strip out all shit including control characters and delimiters
        // since we always use dd.mm.yyyy
        let dmy: String = day_line.chars().filter(|c| char::is_numeric(*c)).collect();

        NaiveDate::parse_from_str(&dmy, "%d%m%Y").map_err(|e| {
            io::Error::new(
                ErrorKind::InvalidInput,
                format!("Parsing '{}' failed with {}", day_line, e),
            )
        })
    }

    /// A unit is a number of lines with higher level of indentation than the preceding line
    fn lookup_end_of_unit(&self, s: &str) -> usize {
        self.unit_ends
            .iter()
            .filter_map(|unit_end| s.find(unit_end))
            .min()
            .unwrap_or_else(|| panic!("Failed to find unit delimiter in: {}", s))
    }
}

/// conveniently load the birthday file to get a list of people and their birthdays
/// see [mdlog::parser::parse_people] for details on the actual format of the file
pub fn load_birthday_file(path: &Path) -> io::Result<Vec<Person>> {
    let s = {
        let mut s = String::new();
        let mut f = File::open(path)?;

        f.read_to_string(&mut s)?;
        s
    };

    parse_people(&s)
}

/// The birthday file contains people, their birthday (with or without year) and present suggestions.  
/// The first, mandatory part is dict of ```<name of the person>: <birthday as dd.mm.yyyy>```.  
/// The second, optional part is separated by a ```# Presents``` and contains a dict of ```<name of the person>: list<presents>```
///
/// # Example:
/// ```
/// # extern crate chrono;
/// # fn main(){
/// use mdlog::types::{Person, Birthday};
/// use mdlog::parser::parse_people;
/// use std::collections::HashSet;
/// use chrono::naive::NaiveDate;
///
/// let file_content = "
///
///Alex: 19.01.2001
///Bob Smith: 20.12.?
///John Johnson: 21.12.1947
///
///### Presents
///Alex:
///- Salad
///- Moar Salad
///
///Bob Smith:
///- Bazooka
///";
/// // the yaml parser does not guarantee ordering of the output so we need to compare
/// // this via a set
/// let peops : HashSet<Person> = parse_people(&file_content).unwrap().into_iter().collect();
/// let correct : HashSet<Person> = [
///     Person{
///         name: "Alex".into(),
///         birthday: Birthday::KnownYear(NaiveDate::from_ymd(2001,01,19)),
///         presents : Some(vec!["Salad".into(), "Moar Salad".into()])
///     },
///     Person{
///         name: "Bob Smith".into(),
///         birthday: Birthday::UnknownYear(12,20),
///         presents : Some(vec!["Bazooka".into()])
///     },
///     Person{
///         name: "John Johnson".into(),
///         birthday: Birthday::KnownYear(NaiveDate::from_ymd(1947,12,21)),
///         presents : None
///     }
/// ].iter().cloned().collect();
/// assert_eq!(&peops, &correct);
/// # }
/// ```
pub fn parse_people(s: &str) -> io::Result<Vec<Person>> {
    let begin_presents = s.find("# Presents");

    let birthdays = {
        let bd_entries_end = begin_presents.unwrap_or(s.len());
        &s[..bd_entries_end]
    };

    let birthdays: HashMap<String, String> =
        serde_yaml::from_str(birthdays).expect("deserialize failed");

    let mut people: Vec<Person> = birthdays
        .into_iter()
        .map(|(name, birthdate)| {
            // happy path
            match birthdate.rfind('?') {
                None => {
                    let date =
                        NaiveDate::parse_from_str(&birthdate, "%d.%m.%Y").unwrap_or_else(|_| {
                            panic!(
                                "Failed to parse date for {}:{} please check the entry!",
                                name, birthdate
                            )
                        });
                    (name, Birthday::KnownYear(date))
                }
                Some(_pos) => {
                    let mut dm = birthdate
                        .split('.')
                        .map(|s| u32::from_str(s).expect("Failed to parse day or month"));
                    let d = dm.next().unwrap();
                    let m = dm.next().unwrap();
                    (name, Birthday::UnknownYear(m, d))
                }
            }
        })
        .map(|(name, birthday)| Person {
            name,
            birthday,
            presents: None,
        })
        .collect();

    // tack on present suggestions if there are any for this person
    if let Some(split_pos) = begin_presents {
        let presents = &s[split_pos..];
        let mut presents: HashMap<String, Vec<String>> =
            serde_yaml::from_str(presents).expect("Deserialize failed for presents");
        people.iter_mut().for_each(|p| {
            let x = presents.remove(&p.name);
            mem::replace(&mut p.presents, x);
        });
    }

    Ok(people)
}

fn slice<'a>(s: &'a str, start: usize, delim: &'a str) -> (usize, &'a str) {
    let pos = s[start..].find(delim).unwrap();

    (start, &s[start..start + pos])
}

// slice from after the delim  onwards
// panics if the the token is not in the given str
fn slice_from<'a>(s: &'a str, delim: &str) -> &'a str {
    &s[s.find(delim).unwrap() + delim.len()..]
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_DATA: &str = "
# Week 42, 14.10.2019 - 20.10.2019

## Mon, 14.10.2019
- a
- EVT 16:25: b
  - b1
  - b2
- TODO: c

## Tue, 15.10.2019
- TODO: d
    - DONE: d1

## Wed, 16.10.2019
- EVT: e

## Thu, 17.10.2019
- TODO A1: f
    - TODO: f1
    - TODO C3: f2

## Fri, 18.10.2019
- some code
```
# code
```
## Sat, 19.10.2019
- DONE: g
## Sun, 20.10.2019
- EVT 06:01: h

# Week 43, 21.10.2019 - 27.10.2019";

    #[test]
    fn events() {
        let correct = {
            let mon = Event {
                msg: "b".into(),
                notes: vec!["b1".into(), "b2".into()],
                date: NaiveDate::from_ymd(2019, 10, 14),
                time: Some(NaiveTime::from_hms(16, 25, 0)),
            };

            let wed = Event {
                msg: "e".into(),
                notes: vec![],
                date: NaiveDate::from_ymd(2019, 10, 16),
                time: None,
            };

            let sun = Event {
                msg: "h".into(),
                notes: vec![],
                date: NaiveDate::from_ymd(2019, 10, 20),
                time: Some(NaiveTime::from_hms(6, 1, 0)),
            };

            [mon, wed, sun]
        };

        let p = Parser::from_line_end(LINE_END_LINUX);

        let parsed = p.parse_events(EXAMPLE_DATA).unwrap();

        assert_eq!(&parsed, &correct);
    }

    #[test]
    fn tasks() {
        let correct = {
            let mon = Task {
                msg: "c".into(),
                subtasks: vec![],
                notes: vec![],
                date: NaiveDate::from_ymd(2019, 10, 14),
                is_done: false,
            };
            let tue = Task {
                msg: "d".into(),
                subtasks: vec![Subtask {
                    msg: "d1".into(),
                    is_done: true,
                }],
                notes: vec![],
                date: NaiveDate::from_ymd(2019, 10, 15),
                is_done: false,
            };
            let thu = Task {
                msg: "f".into(),
                subtasks: vec![
                    Subtask {
                        msg: "f1".into(),
                        is_done: false,
                    },
                    Subtask {
                        msg: "f2".into(),
                        is_done: false,
                    },
                ],
                notes: vec![],
                date: NaiveDate::from_ymd(2019, 10, 17),
                is_done: false,
            };
            let sat = Task {
                msg: "g".into(),
                subtasks: vec![],
                notes: vec![],
                date: NaiveDate::from_ymd(2019, 10, 19),
                is_done: true,
            };
            [mon, tue, thu, sat]
        };

        let p = Parser::from_line_end(LINE_END_LINUX);

        let tasks = p.parse_tasks(&EXAMPLE_DATA).unwrap();

        assert_eq!(&tasks, &correct);
    }

}
