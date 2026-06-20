use emergent::{
    condition::Condition,
    decision_makers::{
        SingleDecisionMaker,
        planner::{Planner, PlannerAction},
    },
    task::NoTask,
};
use std::{
    collections::{HashMap, HashSet},
    io::Write,
};

/// This example demonstrates how to use `Planner` for interactive quest planning.
/// The player has a goal to deliver a sword to a quest giver, and can get sword
/// by either buying it for 1000 gold or stealing it from a dragon cave while
/// dragon is away. Quest planner will adapt it's suggestions based on player's
/// actions and current world state.
fn main() {
    let mut planner = make_planner();
    let mut world = World::default();

    println!("Interactive quest planner example");
    println!("Goal: deliver sword to quest giver.");
    println!("Use listed available actions to progress.\n");

    print_status(&mut planner, &mut world);

    let stdin = std::io::stdin();
    loop {
        print!("> ");
        if std::io::stdout().flush().is_err() {
            break;
        }

        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }

        let command = line.trim();
        if command.is_empty() {
            continue;
        }

        match command {
            "quit" | "exit" => break,
            _ => match apply_user_action(command, &mut world) {
                Ok(changed_world) => {
                    if changed_world {
                        let _ = planner.find_plan(Some(Action::DeliverSword), &mut world, true);
                    }
                }
                Err(err) => println!("error: {err}"),
            },
        }

        print_status(&mut planner, &mut world);
    }

    println!("bye");
}

macro_rules! map {
	( $type:ty : $( $key:expr => $value:expr, )* ) => {
		{
            #[allow(unused_mut)]
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
            #[allow(unused_mut)]
			let mut result = HashSet::new();
			$(
				result.insert($value);
			)*
			result
		}
	};
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Action {
    BuySword,
    FindDragon,
    WaitDragonLeavesCave,
    StealSword,
    DeliverSword,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Fact {
    Has1000Gold,
    FoundDragon,
    DragonInCave,
    DragonAway,
    HasSword,
    QuestCompleted,
}

#[derive(Debug, Clone)]
struct World {
    gold: usize,
    found_dragon: bool,
    dragon_away: bool,
    has_sword: bool,
    quest_completed: bool,
}

impl Default for World {
    fn default() -> Self {
        Self {
            gold: 250,
            found_dragon: false,
            dragon_away: false,
            has_sword: false,
            quest_completed: false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct HasAtLeastGold(usize);

impl Condition<World> for HasAtLeastGold {
    fn validate(&self, world: &World) -> bool {
        world.gold >= self.0
    }
}

#[derive(Debug, Copy, Clone)]
struct FoundDragon;

impl Condition<World> for FoundDragon {
    fn validate(&self, world: &World) -> bool {
        world.found_dragon
    }
}

#[derive(Debug, Copy, Clone)]
struct DragonInCave;

impl Condition<World> for DragonInCave {
    fn validate(&self, world: &World) -> bool {
        !world.dragon_away
    }
}

#[derive(Debug, Copy, Clone)]
struct DragonAway;

impl Condition<World> for DragonAway {
    fn validate(&self, world: &World) -> bool {
        world.dragon_away
    }
}

#[derive(Debug, Copy, Clone)]
struct HasSword;

impl Condition<World> for HasSword {
    fn validate(&self, world: &World) -> bool {
        world.has_sword
    }
}

#[derive(Debug, Copy, Clone)]
struct QuestCompleted;

impl Condition<World> for QuestCompleted {
    fn validate(&self, world: &World) -> bool {
        world.quest_completed
    }
}

fn make_planner() -> Planner<World, Fact, Action> {
    Planner::new(
        map! {
            Box<dyn Condition<_>> :
            Fact::Has1000Gold => Box::new(HasAtLeastGold(1000)),
            Fact::FoundDragon => Box::new(FoundDragon),
            Fact::DragonInCave => Box::new(DragonInCave),
            Fact::DragonAway => Box::new(DragonAway),
            Fact::HasSword => Box::new(HasSword),
            Fact::QuestCompleted => Box::new(QuestCompleted),
        },
        map! {
            _ :
            Action::BuySword => PlannerAction::new(
                set![Fact::Has1000Gold,],
                set![Fact::HasSword,],
                1.0,
                NoTask,
            ),
            Action::FindDragon => PlannerAction::new(
                set![],
                set![Fact::FoundDragon,],
                2.0,
                NoTask,
            ),
            Action::WaitDragonLeavesCave => PlannerAction::new(
                set![Fact::FoundDragon, Fact::DragonInCave,],
                set![Fact::DragonAway,],
                5.0,
                NoTask,
            ),
            Action::StealSword => PlannerAction::new(
                set![Fact::FoundDragon, Fact::DragonAway,],
                set![Fact::HasSword,],
                1.0,
                NoTask,
            ),
            Action::DeliverSword => PlannerAction::new(
                set![Fact::HasSword,],
                set![Fact::QuestCompleted,],
                1.0,
                NoTask,
            ),
        },
        SingleDecisionMaker::new(Action::DeliverSword),
        false,
    )
    .expect("planner should build")
}

fn available_actions(world: &World) -> Vec<&'static str> {
    let mut actions = Vec::new();

    if world.gold < 1000 {
        actions.push("earn_gold <amount>");
    }
    if !world.found_dragon {
        actions.push("find_dragon");
    }
    if world.found_dragon && !world.dragon_away {
        actions.push("wait_dragon_leaves");
    }
    if world.found_dragon && world.dragon_away && !world.has_sword {
        actions.push("steal_sword");
    }
    if world.gold >= 1000 && !world.has_sword {
        actions.push("buy_sword");
    }
    if world.has_sword && !world.quest_completed {
        actions.push("deliver");
    }

    actions.push("reset");
    actions.push("quit");
    actions
}

fn apply_user_action(command: &str, world: &mut World) -> Result<bool, String> {
    let mut parts = command.split_whitespace();
    let Some(cmd) = parts.next() else {
        return Ok(false);
    };

    match cmd {
        "earn_gold" => {
            let amount = parts
                .next()
                .ok_or_else(|| "usage: earn_gold <amount>".to_string())?
                .parse::<usize>()
                .map_err(|_| "amount must be unsigned integer".to_string())?;
            world.gold = world.gold.saturating_add(amount);
            println!("You earned {amount} gold.");
            Ok(true)
        }
        "find_dragon" => {
            world.found_dragon = true;
            println!("You found the dragon cave.");
            Ok(true)
        }
        "wait_dragon_leaves" => {
            if world.found_dragon && !world.dragon_away {
                world.dragon_away = true;
                println!("Dragon left the cave for now.");
                Ok(true)
            } else {
                Err("cannot wait for dragon leaves right now".to_string())
            }
        }
        "steal_sword" => {
            if world.found_dragon && world.dragon_away {
                world.has_sword = true;
                println!("You stole the sword from the cave.");
                Ok(true)
            } else {
                Err("cannot steal sword now (need: found_dragon + dragon_away)".to_string())
            }
        }
        "buy_sword" => {
            if world.gold >= 1000 {
                world.gold = world.gold.saturating_sub(1000);
                world.has_sword = true;
                println!("You bought the sword for 1000 gold.");
                Ok(true)
            } else {
                Err("not enough gold to buy sword".to_string())
            }
        }
        "deliver" => {
            if world.has_sword && !world.quest_completed {
                world.quest_completed = true;
                println!("You delivered the sword to quest giver. Quest complete.");
                Ok(true)
            } else {
                Err("cannot deliver (need: has_sword and quest not completed)".to_string())
            }
        }
        "reset" => {
            *world = World::default();
            println!("World reset.");
            Ok(true)
        }
        "quit" => Ok(false),
        _ => Err("unknown command".to_string()),
    }
}

fn print_status(planner: &mut Planner<World, Fact, Action>, world: &mut World) {
    let _ = planner.process(world);
    println!("\n=== WORLD ===");
    println!("gold: {}", world.gold);
    println!("found_dragon: {}", world.found_dragon);
    println!("dragon_away: {}", world.dragon_away);
    println!("has_sword: {}", world.has_sword);
    println!("quest_completed: {}", world.quest_completed);

    println!("\n=== PLANNER ===");
    let suggested = planner.active_action().copied();
    println!("suggested next quest step: {:?}", suggested);
    println!("current plan: {:?}", planner.active_plan());

    println!("\n=== AVAILABLE USER ACTIONS ===");
    for action in available_actions(world) {
        println!("- {action}");
    }
    println!();
}
