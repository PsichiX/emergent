use emergent::{
    condition::Condition,
    decision_makers::machinery::{Machinery, MachineryBuilder, MachineryChange, MachineryState},
    task::NoTask,
};
use std::io::Write;

/// This example demonstrates reactive quest progression implemented as
/// a finite state machine (Machinery).
///
/// Compared to planner, transitions are explicit and hand-authored:
/// each state lists all possible jumps and their conditions.
fn main() {
    let mut machinery = make_machinery();
    let mut world = World::default();

    machinery
        .change_active_state(Some(QuestState::Scout), &mut world, true)
        .expect("initial state should exist");

    println!("Interactive quest machinery example");
    println!("Goal: deliver sword to quest giver.");
    println!("Use listed available actions to progress.\n");

    print_status(&mut machinery, &mut world);

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

        let is_reset = command == "reset";

        match command {
            "quit" | "exit" => break,
            _ => match apply_user_action(command, &mut world) {
                Ok(changed_world) => {
                    if changed_world {
                        if is_reset {
                            let _ = machinery.change_active_state(
                                Some(QuestState::Scout),
                                &mut world,
                                true,
                            );
                        }
                        let _ = machinery.process(&mut world);
                    }
                }
                Err(err) => println!("error: {err}"),
            },
        }

        print_status(&mut machinery, &mut world);
    }

    println!("bye");
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum QuestState {
    Scout,
    WaitDragonLeaves,
    AcquireByBuy,
    AcquireBySteal,
    Deliver,
    Completed,
}

#[derive(Debug, Copy, Clone)]
struct CanBuySword;

impl Condition<World> for CanBuySword {
    fn validate(&self, world: &World) -> bool {
        !world.has_sword && world.gold >= 1000
    }
}

#[derive(Debug, Copy, Clone)]
struct CanWaitDragonLeaves;

impl Condition<World> for CanWaitDragonLeaves {
    fn validate(&self, world: &World) -> bool {
        !world.has_sword && world.found_dragon && !world.dragon_away
    }
}

#[derive(Debug, Copy, Clone)]
struct CanStealSword;

impl Condition<World> for CanStealSword {
    fn validate(&self, world: &World) -> bool {
        !world.has_sword && world.found_dragon && world.dragon_away
    }
}

#[derive(Debug, Copy, Clone)]
struct CanDeliverSword;

impl Condition<World> for CanDeliverSword {
    fn validate(&self, world: &World) -> bool {
        world.has_sword && !world.quest_completed
    }
}

#[derive(Debug, Copy, Clone)]
struct IsQuestCompleted;

impl Condition<World> for IsQuestCompleted {
    fn validate(&self, world: &World) -> bool {
        world.quest_completed
    }
}

#[derive(Debug, Copy, Clone)]
struct NeedsSword;

impl Condition<World> for NeedsSword {
    fn validate(&self, world: &World) -> bool {
        !world.quest_completed && !world.has_sword
    }
}

fn make_machinery() -> Machinery<World, QuestState> {
    MachineryBuilder::default()
        .state(
            QuestState::Scout,
            MachineryState::task(NoTask)
                .change(MachineryChange::new(QuestState::Deliver, CanDeliverSword))
                .change(MachineryChange::new(QuestState::AcquireByBuy, CanBuySword))
                .change(MachineryChange::new(
                    QuestState::WaitDragonLeaves,
                    CanWaitDragonLeaves,
                ))
                .change(MachineryChange::new(
                    QuestState::AcquireBySteal,
                    CanStealSword,
                )),
        )
        .state(
            QuestState::WaitDragonLeaves,
            MachineryState::task(NoTask)
                .change(MachineryChange::new(QuestState::Deliver, CanDeliverSword))
                .change(MachineryChange::new(QuestState::AcquireByBuy, CanBuySword))
                .change(MachineryChange::new(
                    QuestState::AcquireBySteal,
                    CanStealSword,
                )),
        )
        .state(
            QuestState::AcquireByBuy,
            MachineryState::task(NoTask)
                .change(MachineryChange::new(QuestState::Deliver, CanDeliverSword))
                .change(MachineryChange::new(
                    QuestState::AcquireBySteal,
                    CanStealSword,
                ))
                .change(MachineryChange::new(
                    QuestState::WaitDragonLeaves,
                    CanWaitDragonLeaves,
                ))
                .change(MachineryChange::new(QuestState::Scout, NeedsSword)),
        )
        .state(
            QuestState::AcquireBySteal,
            MachineryState::task(NoTask)
                .change(MachineryChange::new(QuestState::Deliver, CanDeliverSword))
                .change(MachineryChange::new(QuestState::AcquireByBuy, CanBuySword))
                .change(MachineryChange::new(
                    QuestState::WaitDragonLeaves,
                    CanWaitDragonLeaves,
                ))
                .change(MachineryChange::new(QuestState::Scout, NeedsSword)),
        )
        .state(
            QuestState::Deliver,
            MachineryState::task(NoTask)
                .change(MachineryChange::new(
                    QuestState::Completed,
                    IsQuestCompleted,
                ))
                .change(MachineryChange::new(QuestState::AcquireByBuy, CanBuySword))
                .change(MachineryChange::new(
                    QuestState::AcquireBySteal,
                    CanStealSword,
                ))
                .change(MachineryChange::new(QuestState::Scout, NeedsSword)),
        )
        .state(QuestState::Completed, MachineryState::task(NoTask))
        .build()
}

fn suggested_actions(world: &World) -> Vec<&'static str> {
    let mut actions = Vec::new();

    if !world.quest_completed {
        actions.push("earn_gold <amount>");
        if !world.found_dragon {
            actions.push("find_dragon");
        }
        if world.found_dragon && !world.dragon_away && !world.has_sword {
            actions.push("wait_dragon_leaves");
        }
        if world.found_dragon && world.dragon_away && !world.has_sword {
            actions.push("steal_sword");
        }
        if world.gold >= 1000 && !world.has_sword {
            actions.push("buy_sword");
        }
        if world.has_sword {
            actions.push("deliver");
        }
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

fn print_status(machinery: &mut Machinery<World, QuestState>, world: &mut World) {
    println!("\n=== WORLD ===");
    println!("gold: {}", world.gold);
    println!("found_dragon: {}", world.found_dragon);
    println!("dragon_away: {}", world.dragon_away);
    println!("has_sword: {}", world.has_sword);
    println!("quest_completed: {}", world.quest_completed);

    let active_state = machinery.active_state().copied();
    println!("\n=== MACHINERY ===");
    println!("active quest state: {:?}", active_state);

    println!("\n=== AVAILABLE USER ACTIONS ===");
    for action in suggested_actions(world) {
        println!("- {action}");
    }
    println!();
}
