use std::fs::read_to_string;

#[derive(Debug)]
pub struct WorkTimes {
    pub start: i32,
    pub end: i32,
}

#[derive(Debug)]
pub struct Task {
    pub text: String,
    pub subs: Vec<Task>,
    pub work_times: Vec<WorkTimes>,
}

pub fn main() {
    let buf = read_to_string("/home/sum12/sync/toofc/tasks/done").expect("unable to read file");
    // println!("{}", format!("read {buf} from file"));
    for line in buf.lines() {
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
                start: chunk[0],
                end: -1 * chunk[1],
            })
            .collect();
        let task = Task {
            text,
            subs: vec![],
            work_times: wktimes,
        };
        println!("{:?} {:#?}", task.text, task.work_times);
    }
    ()
}
