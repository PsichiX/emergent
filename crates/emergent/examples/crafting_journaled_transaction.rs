use emergent::{
    condition::Condition,
    decision_makers::sequencer::{Sequencer, SequencerState},
    task::{
        JournaledTransactionTask, Task, TaskStopReason, TransactionCommitPolicy,
        TransactionJournal, TransactionalMemory,
    },
};
use std::{
    io::Write,
    sync::{Arc, Mutex},
};

/// This example demonstrates a transactional crafting flow backed by an undo
/// journal. Each crafting task records undo data before it mutates the world.
fn main() {
    let mut crafting = make_crafting_transaction();
    let mut world = World::default();

    println!("Interactive journaled crafting transaction example");
    println!("Goal: craft an iron sword.");
    println!("Use listed available actions to progress.\n");

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
            _ => match apply_user_action(command, &mut crafting, &mut world) {
                Ok(()) => {}
                Err(err) => println!("error: {err}"),
            },
        }

        print_status(&world);
    }

    println!("bye");
}

#[derive(Debug, Clone)]
struct World {
    iron_ore: usize,
    wood: usize,
    coal: usize,
    iron_swords: usize,
    journal: TransactionJournal<Snapshot>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            iron_ore: 3,
            wood: 1,
            coal: 2,
            iron_swords: 0,
            journal: TransactionJournal::default(),
        }
    }
}

impl TransactionalMemory for World {
    type Undo = Snapshot;

    fn begin_transaction(&mut self) {
        self.journal.begin();
    }

    fn commit_transaction(&mut self, policy: TransactionCommitPolicy) {
        self.journal.commit(policy);
        println!("Transaction committed. Journal cleared.");
    }

    fn rollback_transaction(&mut self) {
        let undos = self.journal.rollback().collect::<Vec<_>>();
        for snapshot in undos {
            self.iron_ore = snapshot.iron_ore;
            self.wood = snapshot.wood;
            self.coal = snapshot.coal;
            self.iron_swords = snapshot.iron_swords;
        }
        println!("Transaction rolled back. Journal entries undone.");
    }

    fn record_undo(&mut self, undo: Self::Undo) {
        self.journal.record(undo);
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

#[derive(Debug, Copy, Clone)]
struct Snapshot {
    iron_ore: usize,
    wood: usize,
    coal: usize,
    iron_swords: usize,
}

type ActionSignal = Arc<Mutex<Option<CraftAction>>>;

#[derive(Debug)]
struct CraftingTransaction {
    task: JournaledTransactionTask<World>,
    requested_action: ActionSignal,
}

#[derive(Debug, Clone)]
struct IsRequestedAction {
    action: CraftAction,
    requested_action: ActionSignal,
}

impl Condition<World> for IsRequestedAction {
    fn validate(&self, _world: &World) -> bool {
        get_requested_action(&self.requested_action) == Some(self.action)
    }
}

#[derive(Debug, Copy, Clone)]
struct PrepareTask;

impl Task<World> for PrepareTask {
    fn on_enter(&mut self, world: &mut World) {
        record_inventory(world);

        world.wood -= 1;
        println!("Materials measured and laid out on the workbench. Wooden grip prepared.");
    }
}

#[derive(Debug, Copy, Clone)]
struct SmeltTask;

impl Task<World> for SmeltTask {
    fn on_enter(&mut self, world: &mut World) {
        record_inventory(world);

        world.iron_ore -= 2;
        world.coal -= 1;
        println!("Ore smelted into a workable ingot.");
    }
}

#[derive(Debug, Copy, Clone)]
struct HammerTask;

impl Task<World> for HammerTask {
    fn on_enter(&mut self, world: &mut World) {
        record_inventory(world);

        println!("Ingot hammered into a blade blank.");
    }
}

#[derive(Debug, Copy, Clone)]
struct AssembleTask;

impl Task<World> for AssembleTask {
    fn on_enter(&mut self, world: &mut World) {
        record_inventory(world);

        println!("Blade fitted into the wooden grip.");
    }
}

#[derive(Debug, Default, Copy, Clone)]
struct FinishTask;

impl Task<World> for FinishTask {
    fn on_enter(&mut self, world: &mut World) {
        record_inventory(world);

        world.iron_swords += 1;
        println!("Sword quenched, sharpened, and added to inventory.");
    }
}

fn make_crafting_transaction() -> CraftingTransaction {
    let requested_action = Arc::new(Mutex::new(None));
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

    CraftingTransaction {
        task: JournaledTransactionTask::new(sequence),
        requested_action,
    }
}

fn record_inventory(world: &mut World) {
    world.record_undo(Snapshot {
        iron_ore: world.iron_ore,
        wood: world.wood,
        coal: world.coal,
        iron_swords: world.iron_swords,
    });
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

fn apply_user_action(
    command: &str,
    crafting: &mut CraftingTransaction,
    world: &mut World,
) -> Result<(), String> {
    match command {
        "start" => {
            if world.journal.is_active() {
                return Err("crafting is already in progress".to_string());
            }
            if world.iron_ore < 2 || world.wood < 1 || world.coal < 1 {
                return Err("need 2 iron_ore, 1 wood, and 1 coal".to_string());
            }
            set_requested_action(&crafting.requested_action, None);
            crafting.task.on_enter(world);
            println!("Crafting transaction opened.");
        }
        "prepare" => advance_crafting(CraftAction::Prepare, crafting, world)?,
        "smelt" => advance_crafting(CraftAction::Smelt, crafting, world)?,
        "hammer" => advance_crafting(CraftAction::Hammer, crafting, world)?,
        "assemble" => advance_crafting(CraftAction::Assemble, crafting, world)?,
        "finish" => advance_crafting(CraftAction::Finish, crafting, world)?,
        "cancel" => {
            if !world.journal.is_active() {
                return Err("no active crafting transaction to cancel".to_string());
            }
            set_requested_action(&crafting.requested_action, None);
            crafting.task.on_stop(world, TaskStopReason::Cancelled);
        }
        "reset" => {
            if world.journal.is_active() {
                set_requested_action(&crafting.requested_action, None);
                crafting.task.on_stop(world, TaskStopReason::Cancelled);
            }
            *world = World::default();
            println!("World reset.");
        }
        _ => return Err("unknown command".to_string()),
    }

    Ok(())
}

fn advance_crafting(
    action: CraftAction,
    crafting: &mut CraftingTransaction,
    world: &mut World,
) -> Result<(), String> {
    if !world.journal.is_active() {
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
    }

    Ok(())
}

fn available_actions(world: &World) -> Vec<&'static str> {
    let mut actions = Vec::new();

    if world.journal.is_active() {
        actions.push("prepare");
        actions.push("smelt");
        actions.push("hammer");
        actions.push("assemble");
        actions.push("finish");
    } else if world.iron_ore >= 2 && world.wood >= 1 && world.coal >= 1 {
        actions.push("start");
    }

    if world.journal.is_active() {
        actions.push("cancel");
    }
    actions.push("reset");
    actions.push("quit");
    actions
}

fn print_status(world: &World) {
    println!("\n=== INVENTORY ===");
    println!("iron_ore: {}", world.iron_ore);
    println!("wood: {}", world.wood);
    println!("coal: {}", world.coal);
    println!("iron_swords: {}", world.iron_swords);

    println!("\n=== TRANSACTION ===");
    println!("journal depth: {}", world.journal.depth());
    println!("journal active: {}", world.journal.is_active());

    println!("\n=== AVAILABLE USER ACTIONS ===");
    for action in available_actions(world) {
        println!("- {action}");
    }
    println!();
}
