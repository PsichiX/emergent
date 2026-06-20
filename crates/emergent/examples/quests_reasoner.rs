use emergent::{
    Scalar,
    consideration::Consideration,
    decision_makers::reasoner::{Reasoner, ReasonerBuilder, ReasonerState},
    task::NoTask,
};
use std::io::Write;

/// This example demonstrates utility-based quest progression using `Reasoner`.
/// Each quest state gets a score based on world facts. On each update, reasoner
/// picks the state with the highest score and exposes it as suggested quest step.
fn main() {
    let mut reasoner = make_reasoner();
    let mut world = World::default();

    let _ = reasoner.process(&mut world);

    println!("Interactive quest reasoner example");
    println!("Goal: deliver sword to quest giver.");
    println!("Use listed available actions to progress.\n");

    print_status(&mut reasoner, &mut world);

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
                        let _ = reasoner.process(&mut world);
                    }
                }
                Err(err) => println!("error: {err}"),
            },
        }

        print_status(&mut reasoner, &mut world);
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
struct ScoutScore;

impl Consideration<World> for ScoutScore {
    fn score(&self, world: &World) -> Scalar {
        if world.quest_completed || world.has_sword || world.found_dragon || world.gold >= 1000 {
            0.0
        } else {
            0.8
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct WaitDragonLeavesScore;

impl Consideration<World> for WaitDragonLeavesScore {
    fn score(&self, world: &World) -> Scalar {
        if !world.quest_completed && !world.has_sword && world.found_dragon && !world.dragon_away {
            0.9
        } else {
            0.0
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct AcquireByBuyScore;

impl Consideration<World> for AcquireByBuyScore {
    fn score(&self, world: &World) -> Scalar {
        if world.quest_completed || world.has_sword || world.gold < 1000 {
            0.0
        } else if world.found_dragon && world.dragon_away {
            0.6
        } else {
            0.85
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct AcquireByStealScore;

impl Consideration<World> for AcquireByStealScore {
    fn score(&self, world: &World) -> Scalar {
        if !world.quest_completed && !world.has_sword && world.found_dragon && world.dragon_away {
            0.95
        } else {
            0.0
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct DeliverScore;

impl Consideration<World> for DeliverScore {
    fn score(&self, world: &World) -> Scalar {
        if world.has_sword && !world.quest_completed {
            1.0
        } else {
            0.0
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct CompletedScore;

impl Consideration<World> for CompletedScore {
    fn score(&self, world: &World) -> Scalar {
        if world.quest_completed { 1.0 } else { 0.0 }
    }
}

fn make_reasoner() -> Reasoner<World, QuestState> {
    ReasonerBuilder::default()
        .state(QuestState::Scout, ReasonerState::new(ScoutScore, NoTask))
        .state(
            QuestState::WaitDragonLeaves,
            ReasonerState::new(WaitDragonLeavesScore, NoTask),
        )
        .state(
            QuestState::AcquireByBuy,
            ReasonerState::new(AcquireByBuyScore, NoTask),
        )
        .state(
            QuestState::AcquireBySteal,
            ReasonerState::new(AcquireByStealScore, NoTask),
        )
        .state(
            QuestState::Deliver,
            ReasonerState::new(DeliverScore, NoTask),
        )
        .state(
            QuestState::Completed,
            ReasonerState::new(CompletedScore, NoTask),
        )
        .build()
}

fn state_scores(world: &World) -> [(QuestState, Scalar); 6] {
    [
        (QuestState::Scout, ScoutScore.score(world)),
        (
            QuestState::WaitDragonLeaves,
            WaitDragonLeavesScore.score(world),
        ),
        (QuestState::AcquireByBuy, AcquireByBuyScore.score(world)),
        (QuestState::AcquireBySteal, AcquireByStealScore.score(world)),
        (QuestState::Deliver, DeliverScore.score(world)),
        (QuestState::Completed, CompletedScore.score(world)),
    ]
}

fn available_actions(world: &World) -> Vec<&'static str> {
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

fn print_status(reasoner: &mut Reasoner<World, QuestState>, world: &mut World) {
    let _ = reasoner.process(world);

    println!("\n=== WORLD ===");
    println!("gold: {}", world.gold);
    println!("found_dragon: {}", world.found_dragon);
    println!("dragon_away: {}", world.dragon_away);
    println!("has_sword: {}", world.has_sword);
    println!("quest_completed: {}", world.quest_completed);

    println!("\n=== REASONER ===");
    println!("suggested quest state: {:?}", reasoner.active_state());
    println!("state scores:");
    for (state, score) in state_scores(world) {
        println!("- {:?}: {:.2}", state, score);
    }

    println!("\n=== AVAILABLE USER ACTIONS ===");
    for action in available_actions(world) {
        println!("- {action}");
    }
    println!();
}
