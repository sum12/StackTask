pub mod tasks;
use chrono::{DateTime, Local};
pub use tasks::Task;

pub mod work_times;
pub use work_times::WorkTimes;

use std::fs::read_to_string;

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
        t.subs.iter().map(|sub| (sub, indent)).collect(),
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

    let buf = read_to_string(filepath).expect("unable to read file");
    let mut tasks: Vec<_> = buf
        .lines()
        .map(|line| {
            let splits = line.split(",");
            // splits = dbg!(splits);
            let text = splits
                .clone()
                .last()
                .expect(&format!("missing text for line {line}"))
                .to_owned();
            let times = splits.clone().fold(vec![], |mut acc, entry| {
                let mut number = entry;
                if entry.as_bytes()[0] == b'#' {
                    acc.clear();
                    number = &entry[1..];
                }
                acc.push(number);
                acc
            });
            let wktimes: Vec<WorkTimes> = times
                .chunks_exact(2)
                .filter_map(
                    |chunk| match (chunk[0].parse::<i32>(), chunk[1].parse::<i32>()) {
                        (Ok(end), Ok(start)) => Some(WorkTimes {
                            end,
                            start: -1 * start,
                        }),
                        _ => None,
                    },
                )
                .collect();
            Task {
                text,
                subs: vec![],
                work_times: wktimes,
            }
        })
        .collect();

    let mut root = Task {
        text: "ROOT".to_owned(),
        subs: vec![],
        work_times: vec![WorkTimes {
            start: i32::MIN,
            end: i32::MAX,
        }],
    };
    tasks.sort_by_key(|t| t.start());

    for task in tasks.iter() {
        push_subtask(&mut root, task);
    }
    let with_idents = indents(&root, 0);
    let mut better: Vec<_> = with_idents
        .iter()
        .flat_map(|(ref task, ref indent)| {
            task.work_times.iter().map(move |wktime| {
                (
                    Task {
                        text: task.text.to_string(),
                        subs: vec![],
                        work_times: vec![wktime.clone()],
                    },
                    indent,
                )
            })
        })
        .filter(|(task, _)| (task.start() as i64) >= from_secs)
        .collect();
    better.sort_by_key(|(task, _)| task.start());

    let mut prev: chrono::NaiveDate = Default::default();
    for (task, indent) in better {
        let (hours, remaining) = (task.duration() / 3600, task.duration() % 3600);
        let (minutes, seconds) = (remaining / 60, task.duration() % 60);
        let date = DateTime::from_timestamp(task.start().into(), 0)
            .unwrap()
            .with_timezone(&Local)
            .date_naive();
        if prev < date {
            println!("{date}");
            prev = date;
        }
        println!(
            "   {hours:02}:{minutes:02}:{seconds:02}{blank: >long$} {text: <50}",
            blank = "",
            long = indent * 2,
            text = task.text,
        );
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
