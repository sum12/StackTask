#!/usr/bin/env python


import csv
from dataclasses import dataclass
from pathlib import Path
import datetime
import os
from functools import cached_property

ADDITIONAL_DAYS = int(os.getenv("ADDITIONAL_DAYS", 0))
TFDEBUG = int(os.getenv("TFDEBUG", 0))
TASKPATH = Path(os.getenv("TASKPATH"))

@dataclass(frozen=True, order=True)
class WorkingTimes:
    start: int
    end: int

    def __str__(self):
        # return a string representation of the working times
        # where start and end are shown in the format HH:MM
        # and the duration is shown in the format HH:MM:SS
        start = datetime.datetime.fromtimestamp(self.start)
        end = datetime.datetime.fromtimestamp(self.end)
        duration = end-start
        return f"{start.hour:02d}:{start.minute:02d}-{end.hour:02d}:{end.minute:02d}({duration})"

@dataclass
class Task:
    text: str
    subs: list["Task"]
    working_times: list[WorkingTimes]

    def is_subtask_of(self, other: "Task"):
        # check if self is a subtask of other
        # if self is a subtask of other, then all working times of self must be within the either the start or end of other's working times
        if not(other.start <= self.start <= other.finish and other.start <= self.finish <= other.finish):
            return False
        for wt in self.working_times:
            if not any(wt2.start <= wt.start and wt.end <= wt2.end for wt2 in other.working_times):
                for wt3 in other.working_times:
                    if not(wt3.start <= wt.start and wt.end <= wt3.end) :
                        pass
                        #print(f"{self.text} -> {wt3} does not fit in {other.text}, {wt3.start <= wt.start} and {wt.end <= wt3.end}")
                return False
        return True

    @cached_property
    def duration(self):
        return sum(map(lambda x: x.end-x.start, self.working_times))

    @cached_property
    def start(self):
        return min(map(lambda x: x.start, self.working_times))

    @cached_property
    def finish(self):
        return max(map(lambda x: x.end, self.working_times))

def push_or_append(t1: Task, t2:Task):
    # if t2 is a subtask of any of the subtasks of t1, try to push it down into that subtasks
    # otherwise append it to the list of subtasks of t1
    possible = [sub for sub in t1.subs if t2.is_subtask_of(sub)]
    if possible:
        push_or_append(possible[0], t2)
    else:
        t1.subs.append(t2)
        t1.subs = sorted(t1.subs, key=lambda x: x.start)

def dumptree(root: Task, indent=0):
    for task in root.subs:
        yield (task.start, task, indent)
        yield from dumptree(task, indent+1)

def main():
    file_path = TASKPATH

    with open(file_path) as csv_file:
        csv_reader = list(csv.reader(csv_file, delimiter=','))
        if not csv_reader:
            return

        raw_tasks = list(csv_reader)[::-1]
        tasks = []
        # create a list of tasks
        root = Task(text="Root", subs=[], working_times=[WorkingTimes(0, 2**32-1)])
        for *pauses, text in raw_tasks:
            collect = []
            for t in pauses:
                collect.append(t)
                if "#" in t:
                    collect = [t[1:]]
            remaining = collect[::-1]
            working_times = sorted([WorkingTimes(start=-1*int(t[0]), end=int(t[1])) for t in zip(remaining[::2], remaining[1::2])])
            tasks.append(Task(text=text, subs=[], working_times=working_times))
        tasks = sorted(tasks, key=lambda x: x.start)

        # filter out tasks that are not in the current week
        dt =  datetime.datetime.now().replace(hour=0, minute=0, second=0)
#         dt = dt - datetime.timedelta(days=dt.weekday())
        dt = dt - datetime.timedelta(days=ADDITIONAL_DAYS)
        for task in tasks:
            if task.start  < dt.timestamp():
                continue
            push_or_append(root, task)

    # print the tree in a format where the duration is shown and each level is indented based on the subtask level
    prev_day = None
    for (_, task, indent) in sorted(dumptree(root)):
        hours, remainder = divmod(task.duration, 3600)
        minutes, seconds = divmod(remainder, 60)
        text = '  ' * indent + task.text
        wk_times = ""
        if TFDEBUG:
            wk_times = " ".join(map(str, task.working_times))
            curr_day = datetime.datetime.fromtimestamp(task.start).strftime('%a %d %b %Y')
            if prev_day != curr_day:
                prev_day = curr_day
                print(f"{prev_day}")
        print(f"{hours:02d}:{minutes:02d}:{seconds:02d} {text: <80} {wk_times}")

if __name__ == "__main__":
    main()
