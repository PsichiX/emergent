//! Planner (a.k.a. Goal Oriented Action Planner) decision maker.

use crate::{condition::*, consideration::*, decision_makers::*, task::*, DefaultKey, Scalar};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

/// Planner action.
pub enum PlannerError<CK = DefaultKey, AK = DefaultKey> {
    /// There is no condition with given ID found in planner.
    ConditionDoesNotExists(CK),
    /// Condition with given ID is never used by the planner.
    ConditionIsNeverUsed(CK),
    /// There is no action with given ID found in planner.
    ActionDoesNotExists(AK),
}

impl<CK, AK> Clone for PlannerError<CK, AK>
where
    CK: Clone,
    AK: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::ConditionDoesNotExists(key) => Self::ConditionDoesNotExists(key.clone()),
            Self::ConditionIsNeverUsed(key) => Self::ConditionIsNeverUsed(key.clone()),
            Self::ActionDoesNotExists(key) => Self::ActionDoesNotExists(key.clone()),
        }
    }
}

impl<CK, AK> PartialEq for PlannerError<CK, AK>
where
    CK: PartialEq,
    AK: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ConditionDoesNotExists(a), Self::ConditionDoesNotExists(b)) => a == b,
            (Self::ConditionIsNeverUsed(a), Self::ConditionIsNeverUsed(b)) => a == b,
            (Self::ActionDoesNotExists(a), Self::ActionDoesNotExists(b)) => a == b,
            _ => false,
        }
    }
}

impl<CK, AK> Eq for PlannerError<CK, AK>
where
    CK: Eq,
    AK: Eq,
{
}

impl<CK, AK> std::fmt::Debug for PlannerError<CK, AK>
where
    CK: std::fmt::Debug,
    AK: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConditionDoesNotExists(key) => {
                write!(f, "ConditionDoesNotExists({:?})", key)
            }
            Self::ConditionIsNeverUsed(key) => {
                write!(f, "ConditionIsNeverUsed({:?})", key)
            }
            Self::ActionDoesNotExists(key) => {
                write!(f, "ActionDoesNotExists({:?})", key)
            }
        }
    }
}

/// Planner action with preconditions, postconditions, action cost and action task.
///
/// TODO: Explain how planner works and provide simple example code.
pub struct PlannerAction<M = (), K = DefaultKey>
where
    K: Clone + Hash + Eq,
{
    preconditions: HashSet<K>,
    postconditions: HashSet<K>,
    cost: Box<dyn Consideration<M>>,
    task: Box<dyn Task<M>>,
}

impl<M, K> PlannerAction<M, K>
where
    K: Clone + Hash + Eq,
{
    /// Constructs new planner action with set of preconditions, post conditions, cost of the action
    /// and action task.
    pub fn new<C, T>(
        preconditions: HashSet<K>,
        postconditions: HashSet<K>,
        cost: C,
        task: T,
    ) -> Self
    where
        C: Consideration<M> + 'static,
        T: Task<M> + 'static,
    {
        Self {
            preconditions,
            postconditions,
            cost: Box::new(cost),
            task: Box::new(task),
        }
    }

    /// Constructs new planner action with set of preconditions, post conditions, cost of the action
    /// and action task.
    pub fn new_raw(
        preconditions: HashSet<K>,
        postconditions: HashSet<K>,
        cost: Box<dyn Consideration<M>>,
        task: Box<dyn Task<M>>,
    ) -> Self {
        Self {
            preconditions,
            postconditions,
            cost,
            task,
        }
    }

    fn score_preconditions(
        &self,
        conditions: &HashMap<K, Box<dyn Condition<M>>>,
        memory: &M,
    ) -> usize {
        self.preconditions
            .iter()
            .filter(|id| conditions.get(id).unwrap().validate(memory))
            .count()
    }

    fn validate_preconditions(
        &self,
        conditions: &HashMap<K, Box<dyn Condition<M>>>,
        memory: &M,
    ) -> bool {
        self.preconditions
            .iter()
            .all(|id| conditions.get(id).unwrap().validate(memory))
    }

    fn validate_postconditions(
        &self,
        conditions: &HashMap<K, Box<dyn Condition<M>>>,
        memory: &M,
    ) -> bool {
        self.postconditions
            .iter()
            .all(|id| conditions.get(id).unwrap().validate(memory))
    }
}

pub struct Planner<M = (), CK = DefaultKey, AK = DefaultKey>
where
    CK: Clone + Hash + Eq,
    AK: Clone + Hash + Eq,
{
    conditions: HashMap<CK, Box<dyn Condition<M>>>,
    actions: HashMap<AK, PlannerAction<M, CK>>,
    connections: Vec<(AK, AK, Scalar)>,
    goal_selector: Box<dyn DecisionMaker<M, AK>>,
    plan: Option<(usize, Vec<AK>)>,
}

impl<M, CK, AK> Planner<M, CK, AK>
where
    CK: Clone + Hash + Eq,
    AK: Clone + Hash + Eq,
{
    /// Constructs new planner with conditions, actions, goal selector and exact conditions match seting.
    pub fn new<DM>(
        conditions: HashMap<CK, Box<dyn Condition<M>>>,
        actions: HashMap<AK, PlannerAction<M, CK>>,
        goal_selector: DM,
        exact_conditions_match: bool,
    ) -> Result<Self, PlannerError<CK, AK>>
    where
        DM: DecisionMaker<M, AK> + 'static,
    {
        for id in actions.values().flat_map(|action| {
            action
                .preconditions
                .iter()
                .chain(action.postconditions.iter())
        }) {
            if !conditions.contains_key(id) {
                return Err(PlannerError::ConditionDoesNotExists(id.clone()));
            }
        }
        for cid in conditions.keys() {
            if !actions
                .values()
                .flat_map(|action| {
                    action
                        .preconditions
                        .iter()
                        .chain(action.postconditions.iter())
                })
                .any(|aid| aid == cid)
            {
                return Err(PlannerError::ConditionIsNeverUsed(cid.clone()));
            }
        }
        Ok(unsafe {
            Self::new_unchecked(conditions, actions, goal_selector, exact_conditions_match)
        })
    }

    /// Constructs new planner with conditions, actions, goal selector and exact conditions match seting.
    ///
    /// # Safety
    /// Make sure IDs in all inputs matches each other (there are no IDs pointing to non-existing objects)
    pub unsafe fn new_unchecked<DM>(
        conditions: HashMap<CK, Box<dyn Condition<M>>>,
        actions: HashMap<AK, PlannerAction<M, CK>>,
        goal_selector: DM,
        exact_conditions_match: bool,
    ) -> Self
    where
        DM: DecisionMaker<M, AK> + 'static,
    {
        let connections = actions
            .iter()
            .flat_map(|(ak, av)| {
                actions.iter().filter_map(move |(bk, bv)| {
                    let count = av.postconditions.intersection(&bv.preconditions).count();
                    let limit = av.postconditions.len().min(bv.postconditions.len());
                    if exact_conditions_match {
                        if count == limit {
                            return Some((ak.clone(), bk.clone(), 1.0));
                        }
                    } else if count > 0 {
                        return Some((ak.clone(), bk.clone(), limit as Scalar / count as Scalar));
                    }
                    None
                })
            })
            .collect();
        Self {
            conditions,
            actions,
            connections,
            goal_selector: Box::new(goal_selector),
            plan: None,
        }
    }

    /// Returns slice of currently running plan action IDs.
    pub fn active_plan(&self) -> Option<&[AK]> {
        self.plan.as_ref().map(|(start, plan)| &plan[(*start)..])
    }

    /// Tells currently running action ID.
    pub fn active_action(&self) -> Option<&AK> {
        match self.active_plan() {
            Some(plan) => plan.first(),
            None => None,
        }
    }

    /// Tells current goal action ID.
    pub fn active_goal(&self) -> Option<&AK> {
        match &self.plan {
            Some((_, plan)) => plan.last(),
            None => None,
        }
    }

    /// Tells transition between two actions in running plan - current and next action IDs.
    pub fn active_transition(&self) -> (Option<&AK>, Option<&AK>) {
        match self.active_plan() {
            Some(plan) => {
                let mut iter = plan.iter();
                let prev = iter.next();
                let next = iter.next();
                (prev, next)
            }
            None => (None, None),
        }
    }

    /// Find best possible plan towards goal action.
    ///
    /// By default plan won't change if currently running action is locked, unless we force it.
    pub fn find_plan(
        &mut self,
        goal_action: Option<AK>,
        memory: &mut M,
        forced: bool,
    ) -> Result<bool, PlannerError<CK, AK>> {
        if self.active_action() == goal_action.as_ref() {
            return Ok(false);
        }
        let active_action = self.active_action().cloned();
        if let Some(id) = &active_action {
            if !forced && self.actions.get_mut(id).unwrap().task.is_locked(memory) {
                return Ok(false);
            }
        }
        let goal_action = match goal_action {
            Some(id) => id,
            None => {
                if let Some(id) = &active_action {
                    self.actions.get_mut(id).unwrap().task.on_exit(memory);
                }
                self.plan = None;
                self.goal_selector.change_mind(None, memory);
                return Ok(true);
            }
        };
        if !self.actions.contains_key(&goal_action) {
            return Err(PlannerError::ActionDoesNotExists(goal_action));
        }
        let start_action = match self.find_start_action(memory) {
            Some(id) => id,
            None => return Ok(false),
        };
        if start_action == goal_action {
            return Ok(false);
        }
        if let Some(id) = &active_action {
            self.actions.get_mut(id).unwrap().task.on_exit(memory);
            self.plan = None;
        }
        let mut scores = HashMap::with_capacity(self.actions.len());
        scores.insert(
            start_action.clone(),
            self.actions[&start_action].cost.score(memory),
        );
        let mut gscores = HashMap::with_capacity(self.actions.len());
        gscores.insert(start_action.clone(), scores[&start_action]);
        let mut open = Vec::with_capacity(self.actions.len());
        open.push((gscores[&start_action], start_action.clone()));
        let mut came_from = HashMap::<AK, AK>::with_capacity(self.actions.len());
        while !open.is_empty() {
            let index = open
                .iter()
                .enumerate()
                .min_by(|(_, (a, _)), (_, (b, _))| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap();
            let (total_score, id) = open.swap_remove(index);
            if id == goal_action {
                let mut path = Vec::with_capacity(came_from.len());
                path.push(id.clone());
                let mut current = id;
                while let Some(id) = came_from.remove(&current) {
                    path.push(id.clone());
                    current = id;
                }
                path.reverse();
                self.actions
                    .get_mut(&start_action)
                    .unwrap()
                    .task
                    .on_enter(memory);
                self.plan = Some((0, path));
                self.goal_selector.change_mind(Some(goal_action), memory);
                return Ok(true);
            }
            for (nid, weight) in self
                .connections
                .iter()
                .filter(|(from, _, _)| from == &id)
                .map(|(_, to, weight)| (to, weight))
            {
                let gscore = gscores.get(nid).copied().unwrap_or(Scalar::INFINITY);
                let nscore = *scores
                    .entry(nid.clone())
                    .or_insert_with(|| self.actions[nid].cost.score(memory))
                    * weight;
                let score = total_score + nscore;
                if score < gscore {
                    came_from.insert(nid.clone(), id.clone());
                    gscores.insert(nid.clone(), score);
                    if !open.iter().any(|(_, id)| id == nid) {
                        open.push((score, nid.clone()));
                    }
                }
            }
        }
        Ok(false)
    }

    /// Perform decision making.
    pub fn process(&mut self, memory: &mut M) -> bool {
        let new_id = self.goal_selector.decide(memory);
        if new_id.as_ref() == self.active_goal() {
            match self.active_transition() {
                (Some(prev), Some(next)) => {
                    let prev_passing = self
                        .actions
                        .get(prev)
                        .unwrap()
                        .validate_postconditions(&self.conditions, memory);
                    let next_passing = self
                        .actions
                        .get(next)
                        .unwrap()
                        .validate_preconditions(&self.conditions, memory);
                    if prev_passing && next_passing {
                        let prev = prev.clone();
                        let next = next.clone();
                        self.actions.get_mut(&prev).unwrap().task.on_exit(memory);
                        self.actions.get_mut(&next).unwrap().task.on_enter(memory);
                        self.plan.as_mut().unwrap().0 += 1;
                    }
                }
                (Some(prev), None) => {
                    let prev_passing = self
                        .actions
                        .get(prev)
                        .unwrap()
                        .validate_postconditions(&self.conditions, memory);
                    if prev_passing {
                        let prev = prev.clone();
                        self.actions.get_mut(&prev).unwrap().task.on_exit(memory);
                        self.plan = None;
                    }
                }
                _ => {}
            }
        } else if let Ok(true) = self.find_plan(new_id, memory, false) {
            return true;
        }
        if let Some(id) = self.active_action().cloned() {
            return self.actions.get_mut(&id).unwrap().task.on_process(memory);
        }
        false
    }

    /// Update currently active state.
    pub fn update(&mut self, memory: &mut M) {
        if let Some(id) = self.active_action().cloned() {
            self.actions.get_mut(&id).unwrap().task.on_update(memory);
        }
    }

    fn find_start_action(&self, memory: &M) -> Option<AK> {
        self.actions
            .iter()
            .map(|(id, action)| (id, action.score_preconditions(&self.conditions, memory)))
            .max_by(|a, b| a.1.cmp(&b.1))
            .map(|(id, _)| id.clone())
    }
}

impl<M, CK, AK> DecisionMaker<M, AK> for Planner<M, CK, AK>
where
    CK: Clone + Hash + Eq,
    AK: Clone + Hash + Eq,
{
    fn decide(&mut self, memory: &mut M) -> Option<AK> {
        self.process(memory);
        self.active_action().cloned()
    }

    fn change_mind(&mut self, id: Option<AK>, memory: &mut M) -> bool {
        matches!(self.find_plan(id, memory, true), Ok(true))
    }
}

impl<M, CK, AK> Task<M> for Planner<M, CK, AK>
where
    CK: Clone + Hash + Eq,
    AK: Clone + Hash + Eq,
{
    fn is_locked(&self, memory: &M) -> bool {
        if let Some(id) = self.active_action() {
            if let Some(action) = self.actions.get(id) {
                return action.task.is_locked(memory);
            }
        }
        false
    }

    fn on_enter(&mut self, memory: &mut M) {
        let _ = self.find_plan(None, memory, true);
        self.process(memory);
    }

    fn on_exit(&mut self, memory: &mut M) {
        let _ = self.find_plan(None, memory, true);
    }

    fn on_update(&mut self, memory: &mut M) {
        self.update(memory);
    }

    fn on_process(&mut self, memory: &mut M) -> bool {
        self.process(memory)
    }
}
