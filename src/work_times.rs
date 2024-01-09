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
