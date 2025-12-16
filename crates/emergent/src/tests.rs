use crate::{
    DecisionMakingTask, Scalar,
    builders::{
        behavior_tree::{BehaviorTree, BehaviorTreeTask},
        lod::{Lod, LodMemory},
    },
    condition::{ClosureCondition, Condition},
    consideration::Consideration,
    decision_makers::{
        machinery::{Machinery, MachineryChange, MachineryState},
        parallelizer::{Parallelizer, ParallelizerState},
        planner::{Planner, PlannerAction},
        reasoner::{Reasoner, ReasonerState},
        selector::{Selector, SelectorState},
        sequencer::{Sequencer, SequencerState},
    },
    memory::{blackboard::Blackboard, datatable::DataTable},
    task::{ClosureTask, NoTask, Task},
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

fn check_send_sync<T>()
where
    T: Send + Sync,
{
    println!("{} is Send + Sync!", std::any::type_name::<T>());
}

#[test]
fn test_decision_making_task() {
    fn foo<T>()
    where
        T: DecisionMakingTask,
    {
        println!("{} is DecisionMakingTask!", std::any::type_name::<T>());
    }

    foo::<Machinery>();
    foo::<Parallelizer>();
    foo::<Planner>();
    foo::<Reasoner>();
    foo::<Selector>();
    foo::<Sequencer>();
}

#[test]
fn test_reasoner() {
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
    assert!(reasoner.process(&mut memory));
    assert_eq!(reasoner.active_state(), Some(&Mood::Sad));
    memory.mood = 1.0;
    assert!(reasoner.process(&mut memory));
    assert_eq!(reasoner.active_state(), Some(&Mood::Happy));
    assert!(!reasoner.process(&mut memory));
    assert_eq!(
        reasoner.change_active_state(None, &mut memory, true),
        Ok(true)
    );
    assert_eq!(reasoner.active_state(), None);
}

#[test]
fn test_machinery() {
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
    assert!(!machinery.process(&mut memory));
    assert_eq!(machinery.active_state(), Some(&Mood::Sad));
    memory.mood = 1.0;
    assert!(machinery.process(&mut memory));
    assert_eq!(machinery.active_state(), Some(&Mood::Happy));
    assert!(!machinery.process(&mut memory));
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
            NoTask,
            vec![MachineryChange::new(Action::Sleep, TimeCondition(Time::Night))],
        ),
        Action::Sleep => MachineryState::new(
            NoTask,
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
                ClosureTask::default().enter(|m: &mut Memory| m.time = Time::Day),
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
                ClosureTask::default().enter(|m: &mut Memory| m.time = Time::Night),
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
                ClosureTask::default().enter(|m: &mut Memory| m.location = Location::Home),
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
                ClosureTask::default().enter(|m: &mut Memory| m.location = Location::Workplace),
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
                ClosureTask::default().enter(|m: &mut Memory| m.location = Location::Home),
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
                ClosureTask::default().enter(|m: &mut Memory| m.location = Location::Workplace),
            ),
        },
        machinery,
        true,
    )
    .unwrap();

    assert!(planner.process(&mut memory));
    assert_eq!(
        planner.active_plan(),
        Some(vec![Action::DriveTramToWorkplace, Action::Work].as_slice())
    );

    memory = Memory {
        time: Time::Night,
        location: Location::Workplace,
        traffic: false,
    };

    assert!(planner.process(&mut memory));
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
                ClosureTask::default().enter(|m| *m = false),
            ),
            SequencerState::new(
                ClosureCondition::new(|m| !m),
                ClosureTask::default().enter(|m| *m = true),
            ),
        ],
        true,
        true,
    );

    assert!(sequencer.process(&mut memory));
    assert!(memory);
    assert!(sequencer.process(&mut memory));
    assert!(!memory);
    assert!(sequencer.process(&mut memory));
    assert!(memory);
}

#[test]
fn test_selector() {
    let mut memory = false;

    let mut selector = Selector::new(vec![
        SelectorState::new(
            ClosureCondition::new(|m| *m),
            ClosureTask::default().enter(|m| *m = false),
        ),
        SelectorState::new(
            ClosureCondition::new(|m| !m),
            ClosureTask::default().enter(|m| *m = true),
        ),
    ]);

    assert!(selector.process(&mut memory));
    assert!(memory);
    assert!(selector.process(&mut memory));
    assert!(!memory);
    assert!(selector.process(&mut memory));
    assert!(memory);
}

#[test]
fn test_parallelizer() {
    let mut memory = false;

    let mut parallelizer = Parallelizer::new(vec![
        ParallelizerState::new(
            ClosureCondition::new(|m| *m),
            ClosureTask::default().enter(|m| *m = false),
        ),
        ParallelizerState::new(
            ClosureCondition::new(|m| !m),
            ClosureTask::default().enter(|m| *m = true),
        ),
    ]);

    assert!(parallelizer.process(&mut memory));
    assert!(memory);
    assert!(parallelizer.process(&mut memory));
    assert!(!memory);
    assert!(parallelizer.process(&mut memory));
    assert!(memory);
}

#[test]
fn test_lod() {
    const DELTA_TIME: Scalar = 1.0;

    struct Memory {
        time_since_last_meal: Scalar,
        hunger: Scalar,
    }

    let mut lod = Lod::default()
        // level 0 means agent is not in area and we optimize AI processing by not doing any work,
        // so on task exit we just estimate how much more hungry agent can get during task time.
        .level(ClosureTask::default().exit(|m: &mut LodMemory<Memory>| {
            m.memory.hunger -= m.memory.time_since_last_meal;
            println!("* Background hunger estimation: {}", m.memory.hunger);
        }))
        // level 1 means agent is in area and we want to accurately change its hunger level.
        .level(ClosureTask::default().update(|m: &mut LodMemory<Memory>| {
            m.memory.hunger -= DELTA_TIME;
            println!("* Foreground hunger calculation: {}", m.memory.hunger);
        }))
        .build();

    let mut memory = LodMemory {
        lod_level: 0,
        memory: Memory {
            time_since_last_meal: 0.0,
            hunger: 10.0,
        },
    };

    // we start with agent running in the background.
    assert_eq!(lod.active_index(), None);
    assert!(lod.process(&mut memory));
    assert_eq!(lod.active_index(), Some(0));
    // agent will now run in foreground and we assume 5 seconds have passed since last meal.
    memory.lod_level = 1;
    memory.memory.time_since_last_meal = 5.0;
    assert!(lod.process(&mut memory));
    assert_eq!(lod.active_index(), Some(1));
    assert_eq!(memory.memory.hunger, 5.0);
    lod.update(&mut memory);
    assert_eq!(memory.memory.hunger, 4.0);
}

#[test]
fn test_behavior_tree() {
    struct Memory {
        mode: bool,
        counter: usize,
    }

    struct Countdown(pub usize);

    impl Task<Memory> for Countdown {
        fn is_locked(&self, memory: &Memory) -> bool {
            memory.counter > 0
        }

        fn on_enter(&mut self, memory: &mut Memory) {
            memory.counter = self.0;
        }

        fn on_update(&mut self, memory: &mut Memory) {
            memory.counter = memory.counter.max(1) - 1;
        }
    }

    struct FlipMode;

    impl Task<Memory> for FlipMode {
        fn on_enter(&mut self, memory: &mut Memory) {
            memory.mode = !memory.mode;
        }
    }

    struct IsMode(pub bool);

    impl Condition<Memory> for IsMode {
        fn validate(&self, memory: &Memory) -> bool {
            memory.mode == self.0
        }
    }

    // we define a tree that will perform ping-pong with delay:
    // first we wait 2 turns, flip memory state, then wait 1 turn and flip back memory state.
    let mut tree = BehaviorTree::selector(true)
        .node(
            BehaviorTree::sequence(IsMode(true))
                .node(BehaviorTree::state(true, Countdown(2)))
                .node(BehaviorTree::state(true, FlipMode)),
        )
        .node(
            BehaviorTree::sequence(IsMode(false))
                .node(BehaviorTree::state(true, Countdown(1)))
                .node(BehaviorTree::state(true, FlipMode)),
        )
        .build();

    let mut memory = Memory {
        mode: true,
        counter: 0,
    };

    assert!(tree.on_process(&mut memory));
    assert!(memory.mode);
    assert_eq!(memory.counter, 2);

    assert!(!tree.on_process(&mut memory));
    tree.on_update(&mut memory);
    assert!(memory.mode);
    assert_eq!(memory.counter, 1);

    assert!(!tree.on_process(&mut memory));
    tree.on_update(&mut memory);
    assert!(memory.mode);
    assert_eq!(memory.counter, 0);

    assert!(tree.on_process(&mut memory));

    assert!(tree.on_process(&mut memory));
    assert!(!memory.mode);
    assert_eq!(memory.counter, 1);

    assert!(!tree.on_process(&mut memory));
    tree.on_update(&mut memory);
    assert!(!memory.mode);
    assert_eq!(memory.counter, 0);

    assert!(tree.on_process(&mut memory));
    assert!(memory.mode);
}

#[test]
fn test_send_sync() {
    check_send_sync::<Blackboard>();
    check_send_sync::<DataTable<()>>();
    check_send_sync::<Reasoner<()>>();
    check_send_sync::<Machinery<()>>();
    check_send_sync::<Planner<()>>();
    check_send_sync::<Sequencer<()>>();
    check_send_sync::<Selector<()>>();
    check_send_sync::<Parallelizer<()>>();
    check_send_sync::<Lod<()>>();
    check_send_sync::<BehaviorTreeTask<()>>();
}
