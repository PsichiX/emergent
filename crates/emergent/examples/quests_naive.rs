use std::io::Write;

/// This example demonstrates a typical hand-written quest progression loop
/// without GOAP/planner abstractions. It is intentionally imperative and local:
/// each update recomputes quest stage and suggested next actions from world state.
fn main() {
    let mut world = World::default();

    println!("Interactive naive quest progression example");
    println!("Goal: deliver sword to quest giver.");
    println!("This version uses no emergent APIs, only custom game logic.\n");

    print_status(&world);

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
            _ => {
                if let Err(err) = apply_user_action(command, &mut world) {
                    println!("error: {err}");
                }
            }
        }

        print_status(&world);
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum QuestStep {
    FindDragon,
    WaitDragonLeaves,
    AcquireSword,
    DeliverSword,
    Completed,
}

fn current_quest_step(world: &World) -> QuestStep {
    if world.quest_completed {
        QuestStep::Completed
    } else if !world.has_sword {
        if world.gold >= 1000 {
            QuestStep::AcquireSword
        } else if !world.found_dragon {
            QuestStep::FindDragon
        } else if !world.dragon_away {
            QuestStep::WaitDragonLeaves
        } else {
            QuestStep::AcquireSword
        }
    } else {
        QuestStep::DeliverSword
    }
}

fn suggested_actions(world: &World) -> Vec<&'static str> {
    let mut actions = Vec::new();

    match current_quest_step(world) {
        QuestStep::Completed => {}
        QuestStep::FindDragon => {
            actions.push("find_dragon");
            actions.push("earn_gold <amount>");
        }
        QuestStep::WaitDragonLeaves => {
            actions.push("wait_dragon_leaves");
            actions.push("earn_gold <amount>");
        }
        QuestStep::AcquireSword => {
            if world.gold >= 1000 {
                actions.push("buy_sword");
            }
            if world.found_dragon && world.dragon_away {
                actions.push("steal_sword");
            }
            if actions.is_empty() {
                actions.push("find_dragon");
                actions.push("earn_gold <amount>");
            }
        }
        QuestStep::DeliverSword => actions.push("deliver"),
    }

    actions.push("reset");
    actions.push("quit");
    actions
}

fn apply_user_action(command: &str, world: &mut World) -> Result<(), String> {
    let mut parts = command.split_whitespace();
    let Some(cmd) = parts.next() else {
        return Ok(());
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
            Ok(())
        }
        "find_dragon" => {
            world.found_dragon = true;
            println!("You found the dragon cave.");
            Ok(())
        }
        "wait_dragon_leaves" => {
            if world.found_dragon && !world.dragon_away {
                world.dragon_away = true;
                println!("Dragon left the cave for now.");
                Ok(())
            } else {
                Err("cannot wait for dragon leaves right now".to_string())
            }
        }
        "steal_sword" => {
            if world.found_dragon && world.dragon_away {
                world.has_sword = true;
                println!("You stole the sword from the cave.");
                Ok(())
            } else {
                Err("cannot steal sword now (need: found_dragon + dragon_away)".to_string())
            }
        }
        "buy_sword" => {
            if world.gold >= 1000 {
                world.gold = world.gold.saturating_sub(1000);
                world.has_sword = true;
                println!("You bought the sword for 1000 gold.");
                Ok(())
            } else {
                Err("not enough gold to buy sword".to_string())
            }
        }
        "deliver" => {
            if world.has_sword && !world.quest_completed {
                world.quest_completed = true;
                println!("You delivered the sword to quest giver. Quest complete.");
                Ok(())
            } else {
                Err("cannot deliver (need: has_sword and quest not completed)".to_string())
            }
        }
        "reset" => {
            *world = World::default();
            println!("World reset.");
            Ok(())
        }
        "quit" => Ok(()),
        _ => Err("unknown command".to_string()),
    }
}

fn print_status(world: &World) {
    println!("\n=== WORLD ===");
    println!("gold: {}", world.gold);
    println!("found_dragon: {}", world.found_dragon);
    println!("dragon_away: {}", world.dragon_away);
    println!("has_sword: {}", world.has_sword);
    println!("quest_completed: {}", world.quest_completed);

    let step = current_quest_step(world);
    println!("\n=== NAIVE QUEST LOGIC ===");
    println!("current quest step: {:?}", step);

    println!("\n=== SUGGESTED USER ACTIONS ===");
    for action in suggested_actions(world) {
        println!("- {action}");
    }
    println!();
}
