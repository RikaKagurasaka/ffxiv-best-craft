use ffxiv_crafting::{Actions, Status};

use crate::memory_search_solver;

pub fn solve(craft: Status, mn: bool, wn: usize, obz: bool) -> Vec<Actions> {
    let tnzes_quality_solver = memory_search_solver::Solver::new(craft.clone(), mn, wn, obz);
    let phase1_routes = generate_routes_phase1(&craft);
    let mut phase2_routes: Vec<(Status, Vec<Actions>)> = Vec::new();
    let prog_120 = craft.calc_synthesis(1.2);
    let prog_180 = craft.calc_synthesis(1.8);
    let prog_200 = craft.calc_synthesis(2.0);
    for (mut s, mut actions) in phase1_routes {
        let (mut final_actions, final_cp, final_du) = match s.recipe.difficulty - s.progress {
            x if x <= prog_120 => (vec![Actions::BasicSynthesis], 0, 1),
            x if x <= prog_180 => (vec![Actions::CarefulSynthesis], 7, 1),
            x if x <= prog_200 => (vec![Actions::Observe, Actions::FocusedSynthesis], 12, 1),
            _ => continue,
        };
        let mut craft_points = s.craft_points - final_cp;
        let mut durability = s.durability - final_du;
        while let Some(next_action) = tnzes_quality_solver
            .next_touch(craft_points, durability, s.buffs)
            .action
        {
            s.cast_action(next_action);
            craft_points = s.craft_points - final_cp;
            durability = s.durability - final_du;
            actions.push(next_action);
        }
        println!("{actions:?} {}", s.quality);
        actions.append(&mut final_actions);
        phase2_routes.push((s, actions));
    }
    let res = phase2_routes.into_iter().max_by(|a, b| {
        let cond1 = a.0.quality.cmp(&b.0.quality);
        let cond2 = a.0.step.cmp(&b.0.step).reverse();
        cond1.then(cond2)
    });
    let Some((_, content)) = res else {
       return vec![]
    };
    content
}

pub fn next_action_picker_1(craft: &Status) -> Vec<Actions> {
    if craft.is_finished() {
        return vec![];
    }
    let mut available_actions = Vec::new();
    let mut forbidden_actions = Vec::new();
    match craft.step {
        0 => return vec![Actions::MuscleMemory],
        1 => return vec![Actions::Manipulation],
        2 => return vec![Actions::Veneration],
        3 => {
            available_actions.push(Actions::WasteNot);
            available_actions.push(Actions::WasteNotII);
        }
        _ => {}
    }
    if craft.buffs.wast_not > 0 || craft.buffs.muscle_memory > 0 {
        available_actions.push(Actions::Groundwork)
    }
    if craft.buffs.muscle_memory > 0 {
        forbidden_actions.append(&mut vec![
            Actions::BasicSynthesis,
            Actions::CarefulSynthesis,
            Actions::PrudentSynthesis,
            Actions::DelicateSynthesis,
        ])
    }
    if craft.buffs.wast_not > 0 {
        forbidden_actions.push(Actions::PrudentSynthesis)
    }
    available_actions.append(&mut vec![
        Actions::BasicSynthesis,
        Actions::CarefulSynthesis,
        Actions::PrudentSynthesis,
        Actions::DelicateSynthesis,
    ]);
    let mut result_actions = Vec::new();
    for action in available_actions {
        if !forbidden_actions.contains(&action)
            && craft.is_action_allowed(action).is_ok()
            && !result_actions.contains(&action)
        {
            result_actions.push(action);
        }
    }
    result_actions
}

pub fn generate_routes_phase1(s: &Status) -> Vec<(Status, Vec<Actions>)> {
    let prog_200 = s.calc_synthesis(2.0);
    let mut queue = vec![(s.clone(), vec![])];
    let mut routes = Vec::new();
    while let Some((status, actions)) = queue.pop() {
        for action in next_action_picker_1(&status) {
            let mut craft = status.clone();
            craft.cast_action(action);
            let remaining_prog = craft.recipe.difficulty - craft.progress;
            let get_actions = || {
                let mut new_actions = Vec::with_capacity(actions.len() + 1);
                new_actions.clone_from(&actions);
                new_actions.push(action);
                new_actions
            };
            if remaining_prog <= prog_200 && remaining_prog > 0 {
                routes.push((craft, get_actions()));
            } else if craft.step < 8 {
                queue.push((craft, get_actions()));
            }
        }
    }
    routes
}
