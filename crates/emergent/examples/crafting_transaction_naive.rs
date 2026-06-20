use std::{collections::HashMap, io::Write};

/// Example of a transactional crafting mechanism implemented without `emergent`
/// crate primitives.
fn main() {
    let mut inventory = default_inventory();
    let mut process = CraftingProcess::new(crafting_recipe());

    println!("Interactive naive crafting transaction example");
    println!("Goal: craft a sword by executing recipe steps in order.");
    println!("This main function is pure user-side code driving your API.\n");

    print_status(&inventory, &process);

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

        if command == "quit" || command == "exit" {
            break;
        }

        if let Err(error) = apply_user_command(command, &mut inventory, &mut process) {
            println!("error: {error}");
        }

        print_status(&inventory, &process);
    }

    println!("bye");
}

fn default_inventory() -> Inventory {
    let mut inventory = Inventory::default();
    inventory.insert(Resource::IronOre, 3);
    inventory.insert(Resource::Wood, 1);
    inventory.insert(Resource::Coal, 2);
    inventory
}

fn crafting_recipe() -> CraftingRecipe {
    CraftingRecipe::default()
        .step(CraftingStep::new(CraftingOperation::Prepare).take(Resource::Wood, 1))
        .step(
            CraftingStep::new(CraftingOperation::Smelt)
                .take(Resource::IronOre, 2)
                .take(Resource::Coal, 1),
        )
        .step(CraftingStep::new(CraftingOperation::Hammer))
        .step(CraftingStep::new(CraftingOperation::Assemble).produce(Resource::Sword, 1))
}

fn apply_user_command(
    command: &str,
    inventory: &mut Inventory,
    process: &mut CraftingProcess,
) -> Result<(), String> {
    match command {
        "next" => process.perform_next_step(inventory),
        "cancel" => {
            process.rollback(inventory);
            println!("Transaction rolled back.");
            Ok(())
        }
        "reset" => {
            *inventory = default_inventory();
            *process = CraftingProcess::new(crafting_recipe());
            println!("Inventory and process reset.");
            Ok(())
        }
        _ => Err("unknown command".to_string()),
    }
}

fn available_actions(process: &CraftingProcess) -> Vec<&'static str> {
    let mut actions = Vec::new();
    if process.next_operation().is_some() {
        actions.push("next");
        if process.step_index > 0 {
            actions.push("cancel");
        }
    }
    actions.push("reset");
    actions.push("quit");
    actions
}

fn print_status(inventory: &Inventory, process: &CraftingProcess) {
    println!("\n=== INVENTORY ===");
    println!(
        "iron_ore: {}",
        inventory.get(&Resource::IronOre).copied().unwrap_or(0)
    );
    println!(
        "wood: {}",
        inventory.get(&Resource::Wood).copied().unwrap_or(0)
    );
    println!(
        "coal: {}",
        inventory.get(&Resource::Coal).copied().unwrap_or(0)
    );
    println!(
        "sword: {}",
        inventory.get(&Resource::Sword).copied().unwrap_or(0)
    );

    println!("\n=== PROCESS ===");
    println!("step_index: {}", process.step_index);
    println!("next_operation: {:?}", process.next_operation());

    println!("\n=== AVAILABLE USER ACTIONS ===");
    for action in available_actions(process) {
        println!("- {action}");
    }
    println!();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Resource {
    IronOre,
    Wood,
    Coal,
    Sword,
}

type Inventory = HashMap<Resource, usize>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CraftingOperation {
    Prepare,
    Smelt,
    Hammer,
    Assemble,
}

#[derive(Debug)]
struct CraftingStep {
    operation: CraftingOperation,
    take_resources: Inventory,
    produce_resources: Inventory,
}

impl CraftingStep {
    fn new(operation: CraftingOperation) -> Self {
        Self {
            operation,
            take_resources: Default::default(),
            produce_resources: Default::default(),
        }
    }

    fn take(mut self, resource: Resource, amount: usize) -> Self {
        *self.take_resources.entry(resource).or_default() += amount;
        self
    }

    fn produce(mut self, resource: Resource, amount: usize) -> Self {
        *self.produce_resources.entry(resource).or_default() += amount;
        self
    }
}

#[derive(Debug, Default)]
struct CraftingRecipe {
    steps: Vec<CraftingStep>,
}

impl CraftingRecipe {
    fn step(mut self, step: CraftingStep) -> Self {
        self.steps.push(step);
        self
    }
}

#[derive(Debug, Default)]
struct CraftingProcess {
    steps: Vec<CraftingStep>,
    step_index: usize,
}

impl CraftingProcess {
    fn new(recipe: CraftingRecipe) -> Self {
        Self {
            steps: recipe.steps,
            step_index: 0,
        }
    }

    fn next_operation(&self) -> Option<CraftingOperation> {
        self.steps.get(self.step_index).map(|step| step.operation)
    }

    fn perform_next_step(&mut self, inventory: &mut Inventory) -> Result<(), String> {
        if let Some(step) = self.steps.get(self.step_index) {
            for (resource, &amount) in &step.take_resources {
                let available = inventory.get(resource).copied().unwrap_or(0);
                if available < amount {
                    return Err(format!(
                        "Not enough {:?}: needed {}, available {}",
                        resource, amount, available
                    ));
                }
            }
            for (resource, &amount) in &step.take_resources {
                *inventory.entry(*resource).or_default() -= amount;
            }
            for (resource, &amount) in &step.produce_resources {
                *inventory.entry(*resource).or_default() += amount;
            }
            self.step_index += 1;
            Ok(())
        } else {
            Err("No more steps left".to_string())
        }
    }

    fn rollback(&mut self, inventory: &mut Inventory) {
        while self.step_index > 0 {
            self.step_index -= 1;
            if let Some(step) = self.steps.get(self.step_index) {
                for (resource, &amount) in &step.produce_resources {
                    *inventory.entry(*resource).or_default() -= amount;
                }
                for (resource, &amount) in &step.take_resources {
                    *inventory.entry(*resource).or_default() += amount;
                }
            }
        }
    }
}
