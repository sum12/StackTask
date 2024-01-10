pub mod tasks;
use chrono::{DateTime, Local};
pub use tasks::Task;

pub mod work_times;
pub use work_times::WorkTimes;

use std::{fs::read_to_string, iter::repeat};

// a method that takes two tasks (t1 and t2) and pushes the second task as a subtask of the first task
// if the second task is a subtask of the first task but not a subtask of any of the subtasks of the first task
// then the second task is pushed as a subtask of the first task
// if the second task is a subtask of any of the subtasks of the first task
//
//

fn push_subtask(t1: &mut Task, t2: &Task) {
    for sub in t1.subs.iter_mut() {
        if t2.is_subtask_of(sub) {
            push_subtask(sub, t2);
            return;
        }
    }
    t1.subs.push(t2.clone());
    t1.subs.sort();
}

fn indents(t: &Task, indent: usize) -> Vec<(&Task, usize)> {
    t.subs.iter().fold(
        t.subs.iter().zip(repeat(indent)).collect(),
        |mut acc: Vec<_>, sub| {
            acc.append(&mut indents(sub, indent + 1));
            acc
        },
    )
}

// a method to read a file and return a vector of tasks
// each line in the file is a task
// each task has a text and a list of work times
// each work time is a pair of numbers
// the first number is the start time and the second number is the end time
pub fn main() {
    let filepath = std::env::var("TASKPATH").expect("TASKPATH not set");
    let today = chrono::offset::Local::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("could not get today's date")
        .timestamp();
    let from_secs = today
        - std::env::var("ADDITIONAL_DAYS")
            .unwrap_or("1".to_string())
            .parse::<i64>()
            .unwrap_or(1_i64)
            * 24
            * 3600;

    let mut root = Task {
        text: "ROOT".to_owned(),
        subs: vec![],
        work_times: vec![WorkTimes {
            start: i32::MIN,
            end: i32::MAX,
        }],
    };
    let mut tasks: Vec<_> = read_to_string(filepath)
        .expect("unable to read file")
        .lines()
        .map(|line| {
            let mut splits = line.split(',').rev();
            let text = splits
                .next()
                .expect(&format!("missing text for line {line}"))
                .to_owned();
            let mut wktimes = vec![];
            let mut done = false;
            loop {
                match (splits.next(), splits.next()) {
                    (Some(start_entry), Some(end_entry)) => {
                        let end_entry = if end_entry.as_bytes()[0] == b'#' {
                            done = true;
                            &end_entry[1..]
                        } else {
                            &end_entry[..]
                        };
                        match (start_entry.parse::<i32>(), end_entry.parse::<i32>()) {
                            (Ok(start), Ok(end)) => wktimes.push(WorkTimes {
                                end,
                                start: -1 * start,
                            }),
                            _ => break,
                        };
                        if done {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            Task {
                text,
                work_times: wktimes,
                ..Default::default()
            }
        })
        .filter(|task| (task.start() as i64) >= from_secs)
        .collect();

    tasks.sort_by_key(|t| t.start());

    for task in tasks.iter() {
        push_subtask(&mut root, task);
    }
    let mut with_idents: Vec<(&Task, usize)> = indents(&root, 0);
    with_idents.sort_by_key(|(task, _)| task.start());
    let mut better: Vec<_> = with_idents
        .iter()
        .flat_map(|(task, indent)| task.into_iter().zip(repeat(indent)))
        .collect();
    better.sort_by_key(|(task, _)| task.start());

    let mut prev: chrono::NaiveDate = Default::default();
    for (task, indent) in better {
        let (hours, minutes) = (task.duration() / 3600, (task.duration() % 3600) / 60);
        let date = DateTime::from_timestamp(task.start().into(), 0)
            .unwrap()
            .with_timezone(&Local)
            .date_naive();
        if prev < date {
            println!("{date}");
            prev = date;
        }
        println!(
            "   - {hours: >2}:{minutes: >2}{blank: >long$} {text: <50}",
            hours = zero_to_space(hours),
            minutes = zero_to_space(minutes),
            blank = "",
            long = indent * 2,
            text = task.text,
        );
    }
}

fn zero_to_space(number: i32) -> String {
    if number == 0 {
        " ".to_string()
    } else {
        number.to_string()
    }
}

#[cfg(test)]
mod test {
    // a test to two work times can be orderd by start time and those with same start time can be ordered by end time
    use super::*;
    #[test]
    fn test_work_times_ordering() {
        let wktimes = vec![
            WorkTimes { start: 2, end: 3 },
            WorkTimes { start: 2, end: 4 },
            WorkTimes { start: 1, end: 3 },
            WorkTimes { start: 1, end: 2 },
        ];
        let mut sorted = wktimes.clone();
        sorted.sort();
        assert_eq!(
            sorted,
            vec![
                WorkTimes { start: 1, end: 2 },
                WorkTimes { start: 1, end: 3 },
                WorkTimes { start: 2, end: 3 },
                WorkTimes { start: 2, end: 4 },
            ]
        );
    }

    // a test to check if a task's start time is the minimum of all its work times
    // and finish time is the maximum of all its work times
    #[test]
    fn test_task_start_finish() {
        let task = Task {
            text: "test".to_owned(),
            subs: vec![],
            work_times: vec![
                WorkTimes { start: 3, end: 4 },
                WorkTimes { start: 1, end: 2 },
            ],
        };
        assert_eq!(task.start(), 1);
        assert_eq!(task.finish(), 4);
        assert_eq!(task.duration(), 2);
    }

    // a test to check if a task t1 is a subtask of another task t2
    // where t1 is a subtask of t2 if all the work times of t1 are within the work times of t2
    #[test]
    fn test_task_is_subtask() {
        let t1 = Task {
            text: "t1".to_owned(),
            subs: vec![],
            work_times: vec![
                WorkTimes { start: 1, end: 2 },
                WorkTimes { start: 3, end: 4 },
            ],
        };
        let t2 = Task {
            text: "t2".to_owned(),
            subs: vec![],
            work_times: vec![
                WorkTimes { start: 0, end: 5 },
                WorkTimes { start: 6, end: 7 },
            ],
        };
        assert!(t1.is_subtask_of(&t2));
    }

    // a test to check if a task t1 is not a subtask of another task t2
    // where t1 is a subtask of t2 if all the work times of t1 are within the work times of t2
    // in this case t1 has a work time that is not within the work times of t2
    // so t1 is not a subtask of t2
    // t1 = [1, 2], [3, 4]
    // t2 = [0, 5], [6, 7]
    // t1 is not a subtask of t2
    #[test]
    fn test_task_is_not_subtask() {
        let t1 = Task {
            text: "t1".to_owned(),
            subs: vec![],
            work_times: vec![
                WorkTimes { start: 1, end: 2 },
                WorkTimes { start: 3, end: 4 },
            ],
        };
        let t2 = Task {
            text: "t2".to_owned(),
            subs: vec![],
            work_times: vec![
                WorkTimes { start: 0, end: 1 },
                WorkTimes { start: 6, end: 7 },
            ],
        };
        assert!(!t1.is_subtask_of(&t2));
    }
}
