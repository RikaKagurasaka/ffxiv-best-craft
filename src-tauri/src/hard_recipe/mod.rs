use ffxiv_crafting::Condition::*;
use ffxiv_crafting::{Actions, Status};

pub trait Solver {
    fn run(s: &Status) -> (String, Vec<Actions>);
}

pub struct RedstoneSuan611;

impl Solver for RedstoneSuan611 {
    fn run(s: &Status) -> (String, Vec<Actions>) {
        let mut sb = String::new();
        let mut action = match s {
            s if s.step == 0 || s.buffs.muscle_memory > 0 => {
                sb.push_str(format!("[Phase 1] 坚信起手，").as_str());
                vec![match s.step {
                    0 => Actions::MuscleMemory,
                    1 => Actions::Veneration,
                    _ => Actions::RapidSynthesis,
                }]
            }
            s if s.progress > 0 && s.progress < 7198 => {
                sb.push_str(format!("[Phase 2] 崇敬高速、进展{}，", s.progress).as_str());
                if s.durability < 20 {
                    sb.push_str(format!("耐久 < 20，").as_str());
                    vec![match s.condition {
                        Good => {
                            sb.push_str(format!("红球秘诀，").as_str());
                            Actions::TricksOfTheTrade
                        }
                        Sturdy if s.durability % 5 > 3 => {
                            sb.push_str(format!("蓝球俭约加工把耐久尾数修到2，").as_str());
                            Actions::PrudentSynthesis
                        }
                        _ => {
                            if s.buffs.manipulation == 0 {
                                Actions::Manipulation
                            } else if s.durability <= s.calc_durability(10) {
                                sb.push_str(format!("耐久不够高速、掌握层数有剩下，").as_str());
                                Actions::Observe
                            } else {
                                Actions::RapidSynthesis
                            }
                        }
                    }]
                } else {
                    sb.push_str(format!("耐久 >= 20，").as_str());

                    match s.condition {
                        Primed if s.buffs.manipulation <= 1 => vec![Actions::Manipulation],
                        Good => vec![Actions::TricksOfTheTrade],
                        Malleable if s.buffs.veneration > 0 => {
                            vec![Actions::FinalAppraisal, Actions::RapidSynthesis]
                        }
                        Malleable | _ if s.buffs.veneration > 0 => vec![Actions::RapidSynthesis],
                        _ => vec![Actions::Veneration],
                    }
                }
            }
            s if s.buffs.inner_quiet < 8 => {
                sb.push_str(format!("[Phase 3] 堆叠内静，").as_str());
                let ext_du = s.durability as i32 + 5 * s.buffs.manipulation as i32
                    - 5 * (10 - s.buffs.inner_quiet as i32)
                    - 11;
                sb.push_str(format!("额外耐久：{ext_du}，").as_str());

                if s.quality == 0 && s.buffs.manipulation == 0 {
                    vec![Actions::Manipulation]
                } else if ext_du > 0 {
                    if s.buffs.innovation > 0 {
                        vec![Actions::PrudentTouch]
                    } else {
                        vec![Actions::Innovation]
                    }
                } else if s.buffs.innovation == 0 {
                    vec![match s.condition {
                        Good => Actions::TricksOfTheTrade,
                        Sturdy if s.buffs.inner_quiet == 0 => Actions::PrudentTouch,
                        _ if s.buffs.manipulation <= 1 => Actions::Manipulation,
                        _ => Actions::Innovation,
                    }]
                } else {
                    vec![match s.condition {
                        Good if s.calc_durability(10) < s.durability => Actions::PreciseTouch,
                        Good => Actions::TricksOfTheTrade,

                        Sturdy if s.buffs.inner_quiet == 0 => Actions::PrudentTouch,
                        Sturdy if s.calc_durability(20) < s.durability => Actions::PreparatoryTouch,
                        Sturdy => Actions::BasicTouch,

                        Primed if s.buffs.manipulation <= 1 => Actions::Manipulation,
                        Primed if s.buffs.innovation <= 2 => Actions::Innovation,

                        _ if s.is_action_allowed(Actions::HeartAndSoul).is_ok() => {
                            Actions::HeartAndSoul
                        }
                        _ if s.buffs.heart_and_soul > 0 => Actions::PreciseTouch,
                        _ => Actions::PrudentTouch,
                    }]
                }
            }
            s if s.buffs.inner_quiet < 10 => match (s.buffs.manipulation % 2, s.durability % 10) {
                (1, 1..=5) => vec![
                    Actions::PrudentTouch,
                    Actions::Observe,
                    Actions::FocusedTouch,
                ],
                (1, _) | (0, 1..=5) => vec![
                    Actions::Observe,
                    Actions::FocusedTouch,
                    Actions::Observe,
                    Actions::FocusedTouch,
                ],
                (0, _) => vec![
                    Actions::PrudentTouch,
                    Actions::Observe,
                    Actions::FocusedTouch,
                ],
                _ => unreachable!(),
            },
            _ if s.buffs.inner_quiet == 10 => {
                sb.push_str(format!("[Phase 4] 推满加工条，").as_str());
                vec![match s.condition {
                    Good if s.buffs.innovation == 0 => Actions::TricksOfTheTrade,
                    _ if s.buffs.innovation == 0 => Actions::Innovation,
                    Good if s.calc_durability(10) < s.durability => Actions::PreciseTouch,
                    Good => Actions::TricksOfTheTrade,
                    Sturdy if s.calc_durability(20) < s.durability => Actions::PreparatoryTouch,
                    Primed if s.buffs.innovation <= 2 => Actions::Innovation,
                    _ => Actions::TrainedFinesse,
                }]
            }
            _ => vec![Actions::BasicSynthesis],
        };
        if action.len() > 0 {
            let mut tmp_state = s.clone();
            tmp_state.cast_action(action[0]);
            action = if tmp_state.craft_points >= 74
                || tmp_state.craft_points >= 56 && tmp_state.buffs.innovation >= 2
            {
                action
            } else if s.craft_points >= 74 {
                vec![
                    Actions::GreatStrides,
                    Actions::Innovation,
                    Actions::ByregotsBlessing,
                ]
            } else if s.craft_points >= 56 && s.buffs.innovation >= 2 {
                vec![Actions::GreatStrides, Actions::ByregotsBlessing]
            } else {
                vec![]
            }
        }
        (sb, action)
    }
}

#[cfg(test)]
mod test {
    use super::{RedstoneSuan611, Solver};
    use ffxiv_crafting::{Actions, Attributes, ConditionIterator, Recipe, Status};
    use rand::prelude::*;

    #[test]
    fn simulate() {
        let mut rng = thread_rng();
        let r = Recipe {
            rlv: 611,
            job_level: 90,
            difficulty: 7480,
            quality: 13620,
            durability: 60,
            conditions_flag: 435,
        };
        let a = Attributes {
            level: 90,
            craftsmanship: 4214,
            control: 3528,
            craft_points: 691,
        };
        let conditions =
            ConditionIterator::new(r.conditions_flag as i32, a.level as i32).collect::<Vec<_>>();
        for i in 0..30 {
            println!("running simulation {i}");
            let mut status = Status::new(a, r);
            'solve: while !status.is_finished() {
                print!("{}/{}，", status.progress, status.quality);
                print!("球色：{:?}，", status.condition);
                let (log, next_actions) = RedstoneSuan611::run(&status);
                if next_actions.len() == 0 {
                    println!("求解结果为空：{status:?}");
                    break;
                };
                print!("{log}");
                for next_action in next_actions.into_iter() {
                    print!("{next_action:?}");
                    if let Err(e) = status.is_action_allowed(next_action) {
                        println!("技能错误：{e:?}");
                        break 'solve;
                    }
                    if status.success_rate(next_action) as f32 / 100.0 > random() {
                        print!("，");
                        status.cast_action(next_action);
                    } else {
                        print!("失败，");
                        status.cast_action(match next_action {
                            Actions::RapidSynthesis => Actions::RapidSynthesisFail,
                            Actions::HastyTouch => Actions::HastyTouchFail,
                            Actions::FocusedSynthesis => Actions::FocusedSynthesisFail,
                            Actions::FocusedTouch => Actions::FocusedTouchFail,
                            _ => unreachable!(),
                        });
                    }
                    status.condition = conditions.choose_weighted(&mut rng, |c| c.1).unwrap().0;
                }
                println!();
            }
            println!(
                "simulation {i} result: 进展{}/品质{}",
                status.progress, status.quality
            )
        }
    }
}