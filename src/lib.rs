extern crate chrono;
extern crate serde;
extern crate serde_yaml;

pub mod parser;

pub mod types {
    use chrono::naive::{NaiveDate, NaiveTime};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct Person {
        pub name: String,
        pub birthdate: NaiveDate,
        pub presents: Option<Vec<String>>,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct Task {
        pub msg: String,
        pub subtasks: Vec<Subtask>,
        pub notes: Vec<String>,
        pub date: NaiveDate,
        pub is_done: bool,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct Subtask {
        pub msg: String,
        pub is_done: bool,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    pub struct Event {
        pub msg: String,
        pub notes: Vec<String>,
        pub date: NaiveDate,
        pub time: Option<NaiveTime>,
    }

}
