#![cfg(test)]

use crate::{
    condition::*,
    consideration::*,
    decision_makers::{
        machinery::*, parallelizer::*, planner::*, reasoner::*, selector::*, sequencer::*,
    },
    task::*,
    Scalar,
};
use std::collections::{HashMap, HashSet};

macro_rules! map {
    ( $type:ty : $( $key:expr => $value:expr, )* ) => {
        {
            let mut result = HashMap::<_, $type>::new();
            $(
                result.insert($key, $value);
            )*
            result
        }
    };
}

macro_rules! set {
    ( $( $value:expr, )* ) => {
        {
            let mut result = HashSet::new();
            $(
                result.insert($value);
            )*
            result
        }
    };
}

struct Memory {
    pub mood: Scalar,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Mood {
    Happy,
    Sad,
}

#[derive(Debug, Clone)]
struct MoodStateTask {
    pub name: String,
}

impl MoodStateTask {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Task<Memory> for MoodStateTask {
    fn on_enter(&mut self, memory: &mut Memory) {
        println!("ENTER | state: {} | mood: {}", self.name, memory.mood);
    }

    fn on_exit(&mut self, memory: &mut Memory) {
        println!("EXIT | state: {} | mood: {}", self.name, memory.mood);
    }
}

#[test]
fn test_reasoner() {
    #[derive(Debug, Copy, Clone)]
    struct MoodConsideration {
        pub desired: Scalar,
    }

    impl MoodConsideration {
        pub fn new(desired: Scalar) -> Self {
            Self { desired }
        }
    }

    impl Consideration<Memory> for MoodConsideration {
        fn score(&self, memory: &Memory) -> Scalar {
            1.0 - (self.desired - memory.mood).abs()
        }
    }

    let mut memory = Memory { mood: 0.0 };
    let mut reasoner = Reasoner::new(map! {
        _ :
        Mood::Happy => ReasonerState::new(
            MoodConsideration::new(1.0),
            MoodStateTask::new("Happy"),
        ),
        Mood::Sad => ReasonerState::new(
            MoodConsideration::new(0.0),
            MoodStateTask::new("Sad"),
        ),
    });

    assert_eq!(reasoner.active_state(), None);
    assert_eq!(reasoner.process(&mut memory), true);
    assert_eq!(reasoner.active_state(), Some(&Mood::Sad));
    memory.mood = 1.0;
    assert_eq!(reasoner.process(&mut memory), true);
    assert_eq!(reasoner.active_state(), Some(&Mood::Happy));
    assert_eq!(reasoner.process(&mut memory), false);
    assert_eq!(
        reasoner.change_active_state(None, &mut memory, true),
        Ok(true)
    );
    assert_eq!(reasoner.active_state(), None);
}

#[test]
fn test_machinery() {
    #[derive(Debug, Copy, Clone)]
    enum MoodCondition {
        LessThan(Scalar),
        GreaterThan(Scalar),
    }

    impl Condition<Memory> for MoodCondition {
        fn validate(&self, memory: &Memory) -> bool {
            match self {
                Self::LessThan(v) => memory.mood < *v,
                Self::GreaterThan(v) => memory.mood > *v,
            }
        }
    }

    let mut memory = Memory { mood: 0.0 };
    let mut machinery = Machinery::new(map! {
        _ :
        Mood::Happy => MachineryState::new(
            MoodStateTask::new("Happy"),
            vec![MachineryChange::new(Mood::Sad, MoodCondition::LessThan(0.5))],
        ),
        Mood::Sad => MachineryState::new(
            MoodStateTask::new("Sad"),
            vec![MachineryChange::new(Mood::Happy, MoodCondition::GreaterThan(0.5))],
        ),
    });

    assert_eq!(machinery.active_state(), None);
    assert_eq!(
        machinery.change_active_state(Some(Mood::Sad), &mut memory, true),
        Ok(true)
    );
    assert_eq!(machinery.process(&mut memory), false);
    assert_eq!(machinery.active_state(), Some(&Mood::Sad));
    memory.mood = 1.0;
    assert_eq!(machinery.process(&mut memory), true);
    assert_eq!(machinery.active_state(), Some(&Mood::Happy));
    assert_eq!(machinery.process(&mut memory), false);
    assert_eq!(
        machinery.change_active_state(None, &mut memory, true),
        Ok(true)
    );
    assert_eq!(machinery.active_state(), None);
}

#[test]
fn test_planner() {
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum Time {
        Day,
        Night,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum Location {
        Home,
        Workplace,
    }

    struct Memory {
        pub time: Time,
        pub location: Location,
        pub traffic: bool,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum Action {
        Sleep,
        Work,
        DriveCarToHome,
        DriveCarToWorkplace,
        DriveTramToHome,
        DriveTramToWorkplace,
    }

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum ActionCondition {
        IsDay,
        IsNight,
        AtHome,
        AtWorkplace,
        Traffic,
        NoTraffic,
    }

    #[derive(Debug, Copy, Clone)]
    struct TimeCondition(pub Time);

    impl Condition<Memory> for TimeCondition {
        fn validate(&self, memory: &Memory) -> bool {
            memory.time == self.0
        }
    }

    #[derive(Debug, Copy, Clone)]
    struct LocationCondition(pub Location);

    impl Condition<Memory> for LocationCondition {
        fn validate(&self, memory: &Memory) -> bool {
            memory.location == self.0
        }
    }

    #[derive(Debug, Copy, Clone)]
    struct TrafficCondition(pub bool);

    impl Condition<Memory> for TrafficCondition {
        fn validate(&self, memory: &Memory) -> bool {
            memory.traffic == self.0
        }
    }

    let mut memory = Memory {
        time: Time::Day,
        location: Location::Home,
        traffic: true,
    };

    let mut machinery = Machinery::new(map! {
        _ :
        Action::Work => MachineryState::new(
            NoTask::default(),
            vec![MachineryChange::new(Action::Sleep, TimeCondition(Time::Night))],
        ),
        Action::Sleep => MachineryState::new(
            NoTask::default(),
            vec![MachineryChange::new(Action::Work, TimeCondition(Time::Day))],
        ),
    });
    assert_eq!(
        machinery.change_active_state(Some(Action::Sleep), &mut memory, true),
        Ok(true)
    );

    let mut planner = Planner::new(
        map! {
            Box<dyn Condition<_>> :
            ActionCondition::IsDay => Box::new(TimeCondition(Time::Day)),
            ActionCondition::IsNight => Box::new(TimeCondition(Time::Night)),
            ActionCondition::AtHome => Box::new(LocationCondition(Location::Home)),
            ActionCondition::AtWorkplace => Box::new(LocationCondition(Location::Workplace)),
            ActionCondition::Traffic => Box::new(TrafficCondition(true)),
            ActionCondition::NoTraffic => Box::new(TrafficCondition(false)),
        },
        map! {
            _ :
            Action::Sleep => PlannerAction::new(
                set![
                    ActionCondition::AtHome,
                    ActionCondition::IsNight,
                ],
                set![
                    ActionCondition::AtHome,
                    ActionCondition::IsDay,
                ],
                1.0,
                ClosureTask::default().on_enter(|m: &mut Memory| m.time = Time::Day),
            ),
            Action::Work => PlannerAction::new(
                set![
                    ActionCondition::AtWorkplace,
                    ActionCondition::IsDay,
                ],
                set![
                    ActionCondition::AtWorkplace,
                    ActionCondition::IsNight,
                ],
                1.0,
                ClosureTask::default().on_enter(|m: &mut Memory| m.time = Time::Night),
            ),
            Action::DriveCarToHome => PlannerAction::new(
                set![
                    ActionCondition::AtWorkplace,
                    ActionCondition::NoTraffic,
                ],
                set![
                    ActionCondition::AtHome,
                ],
                1.0,
                ClosureTask::default().on_enter(|m: &mut Memory| m.location = Location::Home),
            ),
            Action::DriveCarToWorkplace => PlannerAction::new(
                set![
                    ActionCondition::AtHome,
                    ActionCondition::NoTraffic,
                ],
                set![
                    ActionCondition::AtWorkplace,
                ],
                1.0,
                ClosureTask::default().on_enter(|m: &mut Memory| m.location = Location::Workplace),
            ),
            Action::DriveTramToHome => PlannerAction::new(
                set![
                    ActionCondition::AtWorkplace,
                    ActionCondition::Traffic,
                ],
                set![
                    ActionCondition::AtHome,
                ],
                1.0,
                ClosureTask::default().on_enter(|m: &mut Memory| m.location = Location::Home),
            ),
            Action::DriveTramToWorkplace => PlannerAction::new(
                set![
                    ActionCondition::AtHome,
                    ActionCondition::Traffic,
                ],
                set![
                    ActionCondition::AtWorkplace,
                ],
                1.0,
                ClosureTask::default().on_enter(|m: &mut Memory| m.location = Location::Workplace),
            ),
        },
        machinery,
        true,
    )
    .unwrap();

    assert_eq!(planner.process(&mut memory), true);
    assert_eq!(
        planner.active_plan(),
        Some(vec![Action::DriveTramToWorkplace, Action::Work].as_slice())
    );

    memory = Memory {
        time: Time::Night,
        location: Location::Workplace,
        traffic: false,
    };

    assert_eq!(planner.process(&mut memory), true);
    assert_eq!(
        planner.active_plan(),
        Some(vec![Action::DriveCarToHome, Action::Sleep].as_slice())
    );
}

#[test]
fn test_sequencer() {
    let mut memory = false;

    let mut sequencer = Sequencer::new(
        vec![
            SequencerState::new(
                ClosureCondition::new(|m| *m),
                ClosureTask::default().on_enter(|m| *m = false),
            ),
            SequencerState::new(
                ClosureCondition::new(|m| !m),
                ClosureTask::default().on_enter(|m| *m = true),
            ),
        ],
        true,
        true,
    );

    assert_eq!(sequencer.process(&mut memory), true);
    assert_eq!(memory, true);
    assert_eq!(sequencer.process(&mut memory), true);
    assert_eq!(memory, false);
    assert_eq!(sequencer.process(&mut memory), true);
    assert_eq!(memory, true);
}

#[test]
fn test_selector() {
    let mut memory = false;

    let mut selector = Selector::new(
        OrderedSelectorStatePicker::First,
        map! {
            _ :
            true => SelectorState::new(
                ClosureCondition::new(|m| *m),
                ClosureTask::default().on_enter(|m| *m = false),
            ),
            false => SelectorState::new(
                ClosureCondition::new(|m| !m),
                ClosureTask::default().on_enter(|m| *m = true),
            ),
        },
    );

    assert_eq!(selector.process(&mut memory), true);
    assert_eq!(memory, true);
    assert_eq!(selector.process(&mut memory), true);
    assert_eq!(memory, false);
    assert_eq!(selector.process(&mut memory), true);
    assert_eq!(memory, true);
}

#[test]
fn test_parallelizer() {
    let mut memory = false;

    let mut parallelizer = Parallelizer::new(vec![
        ParallelizerState::new(
            ClosureCondition::new(|m| *m),
            ClosureTask::default().on_enter(|m| *m = false),
        ),
        ParallelizerState::new(
            ClosureCondition::new(|m| !m),
            ClosureTask::default().on_enter(|m| *m = true),
        ),
    ]);

    assert_eq!(parallelizer.process(&mut memory), true);
    assert_eq!(memory, true);
    assert_eq!(parallelizer.process(&mut memory), true);
    assert_eq!(memory, false);
    assert_eq!(parallelizer.process(&mut memory), true);
    assert_eq!(memory, true);
}
