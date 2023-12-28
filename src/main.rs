use std::{fs::read_to_string, os};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub struct WorkTimes {
    pub start: i32,
    pub end: i32,
}

impl WorkTimes {
    pub fn duration(&self) -> i32 {
        self.end - self.start
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Task {
    pub work_times: Vec<WorkTimes>,
    pub text: String,
    pub subs: Vec<Task>,
}

impl Task {
    pub fn start(&self) -> i32 {
        self.work_times.iter().fold(i32::MAX, |min, wktime| {
            if wktime.start <= min {
                wktime.start
            } else {
                min
            }
        })
    }

    pub fn finish(&self) -> i32 {
        self.work_times.iter().fold(
            i32::MIN,
            |max, wktime| if wktime.end > max { wktime.end } else { max },
        )
    }

    pub fn duration(&self) -> i32 {
        self.work_times.iter().map(|wktime| wktime.duration()).sum()
    }

    // function to check if a task is a subtask of another task
    // a task(T1) is a subtask of another task(T2) if all the work times of T1 are within the work times of T2
    pub fn is_subtask_of(&self, other: &Task) -> bool {
        self.work_times.iter().all(|wktime| {
            other.work_times.iter().any(|other_wktime| {
                wktime.start >= other_wktime.start && wktime.end <= other_wktime.end
            })
        })
    }
}

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

    let buf = read_to_string(filepath).expect("unable to read file");
    // println!("{}", format!("read {buf} from file"));
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
            let mut times = vec![];
            let _: Vec<()> = splits
                .clone()
                .map(|entry| {
                    let mut number = entry;
                    if entry.as_bytes()[0] == b'#' {
                        times.clear();
                        number = &entry[1..];
                    }
                    match number.parse::<i32>() {
                        Ok(v) => times.push(v),
                        _ => (),
                    }
                })
                .collect();
            let wktimes: Vec<WorkTimes> = times
                .chunks(2)
                .map(|chunk| WorkTimes {
                    end: chunk[0],
                    start: -1 * chunk[1],
                })
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
    let mut with_idents = indents(&root, 0);
    with_idents.sort_by_key(|(task, _)| task.start());

    let mut prev_date = i32::MIN;
    for (task, indent) in with_idents {
        let (hours, remaining) = (task.duration() / 3600, task.duration() % 3600);
        let (minutes, seconds) = (remaining / 60, task.duration() % 60);
        let date = task.start() / (3600 * 24);
        if prev_date < date {
            println!("");
            prev_date = date
        }
        println!(
            "{hours:02}:{minutes:02}:{seconds:02}{blank: >long$} {text}",
            blank = "",
            long = indent * 2,
            text = task.text
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
