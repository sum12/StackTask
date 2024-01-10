use crate::work_times::WorkTimes;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
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

impl IntoIterator for &Task {
    type Item = Task;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.work_times
            .iter()
            .map(|wktime| Task {
                text: self.text.to_string(),
                subs: vec![],
                work_times: vec![wktime.clone()],
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}
