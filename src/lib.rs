use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::Mutex,
    time::{Duration, Instant},
};

use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use rayon::prelude::*;

const MAX_ANT_MOVES: u64 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "north" => Some(Direction::North),
            "south" => Some(Direction::South),
            "east" => Some(Direction::East),
            "west" => Some(Direction::West),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Direction::North => "north",
            Direction::South => "south",
            Direction::East => "east",
            Direction::West => "west",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Colony {
    pub exits: Vec<(Direction, usize)>,
    pub inbound: HashSet<(usize, Direction)>,
}

#[derive(Debug, Clone)]
pub struct Ant {
    pub location: usize,
    pub moved: u64,
    pub is_alive: bool,
}

pub struct Hiveum {
    pub colonies: Vec<Colony>,
    pub colony_names: Vec<String>,
    pub name_to_index: HashMap<String, usize>,
    pub active_flags: Vec<bool>,
    pub ants: Vec<Ant>,
    pub seed: u64,
}

impl Hiveum {
    /// Creates a new simulation from a map file with the specified number of ants
    pub fn new(path: &str, ant_count: usize, seed: u64) -> Self {
        let mut hiveum = Self {
            colonies: Vec::new(),
            colony_names: Vec::new(),
            name_to_index: HashMap::new(),
            active_flags: Vec::new(),
            ants: Vec::new(),
            seed,
        };

        hiveum.load_map_file(path);
        hiveum.spawn_ants(ant_count, seed);
        hiveum
    }

    /// Loads and parses the map file to build the colony network
    fn load_map_file(&mut self, path: &str) {
        let map_text = fs::read_to_string(path).expect("Map file error");

        for line in map_text.lines() {
            let entries: Vec<&str> = line.trim().split_whitespace().collect();
            if entries.is_empty() {
                continue;
            }
            self.process_map_line(entries);
        }
    }

    /// Processes a single line from the map file
    fn process_map_line(&mut self, entries: Vec<&str>) {
        let src_idx = self.add_or_get_colony(entries[0].to_string());

        for link in &entries[1..] {
            let (dir_txt, tgt_txt) = link
                .split_once('=')
                .unwrap_or_else(|| panic!("Invalid link format: {}", link));

            let dir = Direction::from_str(dir_txt)
                .unwrap_or_else(|| panic!("Invalid direction: {}", dir_txt));

            let tgt_idx = self.add_or_get_colony(tgt_txt.to_string());
            self.link_colonies(src_idx, dir, tgt_idx);
        }
    }

    /// Spawns ants at random active colonies
    fn spawn_ants(&mut self, ant_count: usize, seed: u64) {
        let available_colonies: Vec<usize> = self
            .colonies
            .iter()
            .enumerate()
            .filter_map(
                |(i, _)| if self.active_flags[i] { Some(i) } else { None },
            )
            .collect();

        if available_colonies.is_empty() {
            panic!("No active colonies available to spawn ants");
        }

        let mut rng = SmallRng::seed_from_u64(seed);
        self.ants = (0..ant_count)
            .map(|_| {
                let loc = *available_colonies
                    .choose(&mut rng)
                    .expect("No available colonies to choose from");
                Ant {
                    location: loc,
                    moved: 0,
                    is_alive: true,
                }
            })
            .collect();
    }

    /// Getts existing colony index or adds a new one
    fn add_or_get_colony(&mut self, name: String) -> usize {
        if let Some(&idx) = self.name_to_index.get(&name) {
            idx
        } else {
            let idx = self.colonies.len();
            self.name_to_index.insert(name.clone(), idx);
            self.colony_names.push(name);
            self.colonies.push(Colony {
                exits: Vec::new(),
                inbound: HashSet::new(),
            });
            self.active_flags.push(true);
            idx
        }
    }

    /// Creates a directional link between two colonies
    fn link_colonies(&mut self, from: usize, dir: Direction, to: usize) {
        self.colonies[from].exits.push((dir.clone(), to));
        self.colonies[to].inbound.insert((from, dir));
    }

    /// Destroys a colony and updates all related connections
    fn destroy_colony(&mut self, idx: usize) {
        if !self.active_flags[idx] {
            return;
        }
        self.active_flags[idx] = false;

        let colonies = self.colonies.split_at_mut(idx);
        let inbound = colonies.1[0].inbound.drain().collect::<Vec<_>>();
        for (src, dir) in inbound {
            let src_colony = colonies
                .0
                .get_mut(src)
                .unwrap_or_else(|| colonies.1.get_mut(0).unwrap());
            src_colony.exits.retain(|(d, t)| *d != dir || *t != idx);
        }

        let outbound: Vec<(Direction, usize)> =
            self.colonies[idx].exits.drain(..).collect();
        for (dir, dst) in outbound {
            self.colonies[dst].inbound.remove(&(idx, dir));
        }
    }

    /// Moves all ants to adjacent colonies
    fn move_ants(&mut self) -> Vec<Vec<usize>> {
        let ant_groupings: Mutex<Vec<Vec<(usize, usize)>>> =
            Mutex::new(Vec::new());
        let chunk_size = std::cmp::max(
            1,
            self.ants.len() / (rayon::current_num_threads() * 4),
        );

        self.ants.par_chunks_mut(chunk_size).enumerate().for_each(
            |(chunk_id, chunk)| {
                let mut rng =
                    SmallRng::seed_from_u64(self.seed + chunk_id as u64);
                let mut temp = Vec::with_capacity(chunk.len());
                let offset = chunk_id * chunk_size;

                for (local_idx, ant) in chunk.iter_mut().enumerate() {
                    if !self.active_flags[ant.location] {
                        ant.is_alive = false;
                        continue;
                    }

                    ant.moved += 1;
                    if let Some(&(_, target)) =
                        self.colonies[ant.location].exits.choose(&mut rng)
                    {
                        ant.location = target;
                    }

                    if ant.is_alive {
                        temp.push((ant.location, offset + local_idx));
                    }
                }

                ant_groupings.lock().unwrap().push(temp);
            },
        );

        let mut merged = Vec::with_capacity(self.ants.len());
        for group in ant_groupings.into_inner().unwrap() {
            merged.extend(group);
        }

        let mut results = vec![Vec::new(); self.colonies.len()];
        for (loc, ant_idx) in merged {
            results[loc].push(ant_idx);
        }
        results
    }

    /// Handles ant collisions and destroys colonies
    fn handle_collisions(&mut self, encounters: &[Vec<usize>]) {
        let mut destroy_list = Vec::new();

        for (col_id, ants_here) in encounters.iter().enumerate() {
            let ant_count = ants_here.len();
            if ant_count >= 2 {
                let pair_count = ant_count / 2;
                let ants_to_destroy = pair_count * 2;

                if pair_count > 0 {
                    destroy_list.push(col_id);

                    for &ant_id in ants_here.iter().take(ants_to_destroy) {
                        self.ants[ant_id].is_alive = false;
                    }

                    if pair_count == 1 {
                        println!(
                            "{} has been destroyed by ant {} and ant {}!",
                            self.colony_names[col_id],
                            ants_here[0] + 1,
                            ants_here[1] + 1
                        );
                    } else {
                        let ant_pairs: Vec<String> = ants_here
                            .chunks(2)
                            .take(pair_count)
                            .map(|pair| {
                                format!(
                                    "ant {} and ant {}",
                                    pair[0] + 1,
                                    pair[1] + 1
                                )
                            })
                            .collect();

                        println!(
                            "{} has been destroyed by {} ants: {}!",
                            self.colony_names[col_id],
                            pair_count,
                            ant_pairs.join(", ")
                        );
                    }
                }
            }
        }

        for id in destroy_list {
            self.destroy_colony(id);
        }
    }

    /// Runs the simulation
    pub fn simulate(&mut self) -> Duration {
        let start_time = Instant::now();

        loop {
            let collisions = self.move_ants();
            self.handle_collisions(&collisions);

            let no_active = self.ants.iter().all(|a| !a.is_alive);
            let all_maxed = self
                .ants
                .iter()
                .filter(|a| a.is_alive)
                .all(|a| a.moved >= MAX_ANT_MOVES);

            if no_active || all_maxed {
                break;
            }
        }

        start_time.elapsed()
    }

    /// Generates a status report of the current hive state
    pub fn hive_status(&self) -> String {
        let mut output = String::new();
        for (id, colony) in self.colonies.iter().enumerate() {
            if !self.active_flags[id] {
                continue;
            }
            let name = &self.colony_names[id];
            if colony.exits.is_empty() {
                output.push_str(&format!("{}\n", name));
            } else {
                let exits: Vec<String> = colony
                    .exits
                    .iter()
                    .map(|(d, t)| {
                        format!("{}={}", d.as_str(), self.colony_names[*t])
                    })
                    .collect();
                output.push_str(&format!("{} {}\n", name, exits.join(" ")));
            }
        }
        output
    }
}
