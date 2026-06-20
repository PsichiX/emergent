use emergent::{
    condition::Condition,
    decision_makers::sequencer::{Sequencer, SequencerState},
    task::{Task, TaskStopReason, TransactionScopeTask},
};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

/// This example demonstrates a transactional, linearly progressed crafting flow
/// using transaction scope tasks.
fn main() {
    let mut crafting = make_crafting_transaction();
    let mut world = Inventory::default();

    println!("Interactive crafting transaction example");
    println!("Goal: craft an iron sword.");
    println!("Use listed available actions to progress.\n");

    print_status(&crafting, &world);

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
            _ => match apply_user_action(command, &mut crafting, &mut world) {
                Ok(()) => {}
                Err(err) => println!("error: {err}"),
            },
        }

        print_status(&crafting, &world);
    }

    println!("bye");
}

#[derive(Debug, Clone)]
struct Inventory {
    iron_ore: usize,
    wood: usize,
    coal: usize,
    iron_swords: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            iron_ore: 3,
            wood: 1,
            coal: 2,
            iron_swords: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum CraftAction {
    Prepare,
    Smelt,
    Hammer,
    Assemble,
    Finish,
}

type ActionSignal = Arc<Mutex<Option<CraftAction>>>;
type SnapshotStore = Arc<Mutex<Option<Inventory>>>;

#[derive(Debug)]
struct CraftingTransaction {
    task: TransactionScopeTask<Inventory>,
    requested_action: ActionSignal,
    active: bool,
}

#[derive(Debug, Clone)]
struct IsRequestedAction {
    action: CraftAction,
    requested_action: ActionSignal,
}

impl Condition<Inventory> for IsRequestedAction {
    fn validate(&self, _world: &Inventory) -> bool {
        get_requested_action(&self.requested_action) == Some(self.action)
    }
}

#[derive(Debug, Copy, Clone)]
struct PrepareTask;

impl Task<Inventory> for PrepareTask {
    fn on_enter(&mut self, world: &mut Inventory) {
        world.wood -= 1;
        println!("Materials measured and laid out on the workbench. Wooden grip prepared.");
    }
}

#[derive(Debug, Copy, Clone)]
struct SmeltTask;

impl Task<Inventory> for SmeltTask {
    fn on_enter(&mut self, world: &mut Inventory) {
        world.iron_ore -= 2;
        world.coal -= 1;
        println!("Ore smelted into a workable ingot.");
    }
}

#[derive(Debug, Copy, Clone)]
struct HammerTask;

impl Task<Inventory> for HammerTask {
    fn on_enter(&mut self, _world: &mut Inventory) {
        println!("Ingot hammered into a blade blank.");
    }
}

#[derive(Debug, Copy, Clone)]
struct AssembleTask;

impl Task<Inventory> for AssembleTask {
    fn on_enter(&mut self, _world: &mut Inventory) {
        println!("Blade fitted into the wooden grip.");
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct FinishTask;

impl Task<Inventory> for FinishTask {
    fn on_enter(&mut self, world: &mut Inventory) {
        world.iron_swords += 1;
        println!("Sword quenched, sharpened, and added to inventory.");
    }
}

fn make_crafting_transaction() -> CraftingTransaction {
    let requested_action = Arc::new(Mutex::new(None));
    let snapshot = Arc::new(Mutex::new(None));
    let sequence = Sequencer::new(
        vec![
            SequencerState::new(
                IsRequestedAction {
                    action: CraftAction::Prepare,
                    requested_action: Arc::clone(&requested_action),
                },
                PrepareTask,
            ),
            SequencerState::new(
                IsRequestedAction {
                    action: CraftAction::Smelt,
                    requested_action: Arc::clone(&requested_action),
                },
                SmeltTask,
            ),
            SequencerState::new(
                IsRequestedAction {
                    action: CraftAction::Hammer,
                    requested_action: Arc::clone(&requested_action),
                },
                HammerTask,
            ),
            SequencerState::new(
                IsRequestedAction {
                    action: CraftAction::Assemble,
                    requested_action: Arc::clone(&requested_action),
                },
                AssembleTask,
            ),
            SequencerState::new(
                IsRequestedAction {
                    action: CraftAction::Finish,
                    requested_action: Arc::clone(&requested_action),
                },
                FinishTask,
            ),
        ],
        false,
        false,
    );

    let task = TransactionScopeTask::new(sequence)
        .begin({
            let snapshot = Arc::clone(&snapshot);
            move |world: &mut Inventory| {
                let saved = world.clone();
                set_snapshot(&snapshot, Some(saved));
                println!("Crafting transaction opened.");
            }
        })
        .commit({
            let snapshot = Arc::clone(&snapshot);
            move |_world: &mut Inventory| {
                set_snapshot(&snapshot, None);
                println!("Transaction committed.");
            }
        })
        .rollback({
            let snapshot = Arc::clone(&snapshot);
            move |world: &mut Inventory| {
                if let Some(saved) = take_snapshot(&snapshot) {
                    *world = saved;
                }
                println!("Transaction rolled back.");
            }
        });

    CraftingTransaction {
        task,
        requested_action,
        active: false,
    }
}

fn get_requested_action(requested_action: &ActionSignal) -> Option<CraftAction> {
    match requested_action.lock() {
        Ok(action) => *action,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

fn set_requested_action(requested_action: &ActionSignal, action: Option<CraftAction>) {
    match requested_action.lock() {
        Ok(mut value) => {
            *value = action;
        }
        Err(poisoned) => {
            *poisoned.into_inner() = action;
        }
    }
}

fn set_snapshot(snapshot: &SnapshotStore, value: Option<Inventory>) {
    match snapshot.lock() {
        Ok(mut slot) => {
            *slot = value;
        }
        Err(poisoned) => {
            *poisoned.into_inner() = value;
        }
    }
}

fn take_snapshot(snapshot: &SnapshotStore) -> Option<Inventory> {
    match snapshot.lock() {
        Ok(mut slot) => slot.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    }
}

fn apply_user_action(
    command: &str,
    crafting: &mut CraftingTransaction,
    world: &mut Inventory,
) -> Result<(), String> {
    match command {
        "start" => {
            if crafting.active {
                return Err("crafting is already in progress".to_string());
            }
            if world.iron_ore < 2 || world.wood < 1 || world.coal < 1 {
                return Err("need 2 iron_ore, 1 wood, and 1 coal".to_string());
            }
            set_requested_action(&crafting.requested_action, None);
            crafting.task.on_enter(world);
            crafting.active = true;
        }
        "prepare" => advance_crafting(CraftAction::Prepare, crafting, world)?,
        "smelt" => advance_crafting(CraftAction::Smelt, crafting, world)?,
        "hammer" => advance_crafting(CraftAction::Hammer, crafting, world)?,
        "assemble" => advance_crafting(CraftAction::Assemble, crafting, world)?,
        "finish" => advance_crafting(CraftAction::Finish, crafting, world)?,
        "cancel" => {
            if !crafting.active {
                return Err("no active crafting transaction to cancel".to_string());
            }
            set_requested_action(&crafting.requested_action, None);
            crafting.task.on_stop(world, TaskStopReason::Cancelled);
            crafting.active = false;
        }
        "reset" => {
            if crafting.active {
                set_requested_action(&crafting.requested_action, None);
                crafting.task.on_stop(world, TaskStopReason::Cancelled);
                crafting.active = false;
            }
            *world = Inventory::default();
            println!("World reset.");
        }
        _ => return Err("unknown command".to_string()),
    }

    Ok(())
}

fn advance_crafting(
    action: CraftAction,
    crafting: &mut CraftingTransaction,
    world: &mut Inventory,
) -> Result<(), String> {
    if !crafting.active {
        return Err("start crafting first".to_string());
    }

    set_requested_action(&crafting.requested_action, Some(action));
    let did_advance = crafting.task.on_process(world);
    set_requested_action(&crafting.requested_action, None);

    if !did_advance {
        return Err("action is not valid at this sequence position".to_string());
    }

    if action == CraftAction::Finish {
        crafting.task.on_stop(world, TaskStopReason::Completed);
        crafting.active = false;
    }

    Ok(())
}

fn available_actions(crafting: &CraftingTransaction, world: &Inventory) -> Vec<&'static str> {
    let mut actions = Vec::new();

    if crafting.active {
        actions.push("prepare");
        actions.push("smelt");
        actions.push("hammer");
        actions.push("assemble");
        actions.push("finish");
    } else if world.iron_ore >= 2 && world.wood >= 1 && world.coal >= 1 {
        actions.push("start");
    }

    if crafting.active {
        actions.push("cancel");
    }
    actions.push("reset");
    actions.push("quit");
    actions
}

fn print_status(crafting: &CraftingTransaction, world: &Inventory) {
    println!("\n=== INVENTORY ===");
    println!("iron_ore: {}", world.iron_ore);
    println!("wood: {}", world.wood);
    println!("coal: {}", world.coal);
    println!("iron_swords: {}", world.iron_swords);

    println!("\n=== TRANSACTION ===");
    println!("active: {}", crafting.active);

    println!("\n=== AVAILABLE USER ACTIONS ===");
    for action in available_actions(crafting, world) {
        println!("- {action}");
    }
    println!();
}
