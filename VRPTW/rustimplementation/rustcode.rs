use anyhow::{anyhow, Result};
use rand::{
    rngs::{SmallRng, StdRng},
    Rng, SeedableRng, seq::SliceRandom,
};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Map, Value};
use std::collections::HashSet;

#[cfg(feature = "cuda")]
use crate::CudaKernel;
#[cfg(feature = "cuda")]
use cudarc::driver::*;
#[cfg(feature = "cuda")]
use std::{collections::HashMap, sync::Arc};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Difficulty {
    pub num_nodes: usize,
    pub better_than_baseline: u32,
}

impl crate::DifficultyTrait<2> for Difficulty {
    fn from_arr(arr: &[i32; 2]) -> Self {
        Self {
            num_nodes: arr[0] as usize,
            better_than_baseline: arr[1] as u32,
        }
    }

    fn to_arr(&self) -> [i32; 2] {
        [self.num_nodes as i32, self.better_than_baseline as i32]
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Solution {
    pub routes: Vec<Vec<usize>>,
}

impl crate::SolutionTrait for Solution {}

impl TryFrom<Map<String, Value>> for Solution {
    type Error = serde_json::Error;

    fn try_from(v: Map<String, Value>) -> Result<Self, Self::Error> {
        from_value(Value::Object(v))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Challenge {
    pub seed: [u8; 32],
    pub difficulty: Difficulty,
    pub demands: Vec<i32>,
    pub distance_matrix: Vec<Vec<i32>>,
    pub ready_times: Vec<i32>,
    pub due_dates: Vec<i32>,
    pub service_time: i32,
    pub max_total_distance: i32,
    pub max_capacity: i32,
    pub max_num_vehicles: usize,
}

/// -------------------------------
/// Helper Functions for Instance Generation
/// -------------------------------
#[inline]
fn euclidean_distance(p1: (i32, i32), p2: (i32, i32)) -> i32 {
    let dx = (p1.0 - p2.0) as f64;
    let dy = (p1.1 - p2.1) as f64;
    dx.hypot(dy).round() as i32
}

#[inline]
fn calculate_probability(point: (i32, i32), seeds: &[(i32, i32)]) -> f64 {
    seeds.iter()
         .map(|&seed| {
             let dist = euclidean_distance(point, seed) as f64;
             (-dist / 40.0).exp()
         })
         .sum()
}

#[inline]
fn find_nearest_cluster(point: (i32, i32), seeds: &[(i32, i32)]) -> i32 {
    seeds.iter()
         .enumerate()
         .min_by_key(|(_, &seed)| euclidean_distance(point, seed))
         .map(|(idx, _)| idx as i32)
         .unwrap_or(-1)
}

// TIG dev bounty available for a GPU optimisation for instance generation!
#[cfg(feature = "cuda")]
pub const KERNEL: Option<CudaKernel> = None;

impl crate::ChallengeTrait<Solution, Difficulty, 2> for Challenge {
    #[cfg(feature = "cuda")]
    fn cuda_generate_instance(
        seed: [u8; 32],
        difficulty: &Difficulty,
        dev: &Arc<CudaDevice>,
        mut funcs: HashMap<&'static str, CudaFunction>,
    ) -> Result<Self> {
        // TIG dev bounty available for a GPU optimisation for instance generation!
        Self::generate_instance(seed, difficulty)
    }

    fn generate_instance(seed: [u8; 32], difficulty: &Difficulty) -> Result<Challenge> {
        let mut rng = SmallRng::from_seed(StdRng::from_seed(seed).gen());

        let num_nodes = difficulty.num_nodes;
        let max_capacity = 200;
        let n = num_nodes - 1;
        let grid_size = 1000;
        let depot = (500, 500);

        // Generate seed points for clustering.
        let max_seeds = std::cmp::min(n, 8);
        let num_seeds = if max_seeds >= 3 { rng.gen_range(3..=max_seeds) } else { 1 };
        let mut seeds: Vec<(i32, i32)> = Vec::with_capacity(num_seeds);
        let mut used_points = HashSet::with_capacity(n * 2);
        used_points.insert(depot);
        while seeds.len() < num_seeds {
            let candidate = (rng.gen_range(0..grid_size), rng.gen_range(0..grid_size));
            if used_points.insert(candidate) {
                seeds.push(candidate);
            }
        }

        // Build node_positions: start with depot then seed points.
        let mut node_positions: Vec<(i32, i32)> = Vec::with_capacity(n + 1);
        node_positions.push(depot);
        node_positions.extend(seeds.iter().copied());

        // Initialize cluster assignments for depot and seeds.
        let mut cluster_assignments: Vec<i32> = vec![-1; node_positions.len()];
        for i in 0..num_seeds {
            cluster_assignments[i + 1] = i as i32;
        }

        // Generate additional clustered nodes until roughly half of the customers.
        while node_positions.len() < (n / 2) + 1 {
            let candidate = (rng.gen_range(0..grid_size), rng.gen_range(0..grid_size));
            if used_points.insert(candidate) {
                if rng.gen::<f64>() < calculate_probability(candidate, &seeds) {
                    node_positions.push(candidate);
                    let cluster_idx = find_nearest_cluster(candidate, &seeds);
                    cluster_assignments.push(cluster_idx);
                }
            }
        }

        // Generate remaining random nodes.
        while node_positions.len() < n + 1 {
            let candidate = (rng.gen_range(0..grid_size), rng.gen_range(0..grid_size));
            if used_points.insert(candidate) {
                node_positions.push(candidate);
                cluster_assignments.push(-1);
            }
        }

        let mut demands: Vec<i32> = (0..num_nodes).map(|_| rng.gen_range(1..36)).collect();
        demands[0] = 0; // Depot demand is 0

        // Compute time window parameters.
        let total_demand: i32 = demands.iter().skip(1).sum();
        let avg_demand = total_demand as f64 / n as f64;
        let avg_route_size = 200.0 / avg_demand;
        let avg_distance = (1000.0 / 4.0) * 0.5214;
        let max_distance = node_positions.iter().skip(1)
            .map(|&p| euclidean_distance(depot, p))
            .max()
            .unwrap_or(0);
        let service_time = 10;
        let depot_due_date = (max_distance as f64 + (avg_distance + service_time as f64) * avg_route_size)
            .round() as i32;

        let mut ready_times = vec![0; n + 1];
        let mut due_dates = vec![0; n + 1];
        due_dates[0] = depot_due_date;

        // Assign due dates and initial ready times.
        for i in 1..=n {
            let dist_from_depot = euclidean_distance(depot, node_positions[i]);
            let min_due = dist_from_depot;
            let mut max_due = depot_due_date - service_time - dist_from_depot;
            if max_due <= min_due {
                max_due = min_due + 1;
            }
            due_dates[i] = rng.gen_range(min_due..max_due);
            ready_times[i] = 0;
        }

        // Adjust due dates for clustered customers.
        for i in 1..=n {
            if cluster_assignments[i] != -1 {
                let dist_from_depot = euclidean_distance(depot, node_positions[i]);
                let min_due = dist_from_depot;
                let max_due = depot_due_date - service_time - dist_from_depot;
                let seed_index = (cluster_assignments[i] + 1) as usize;
                due_dates[i] = (due_dates[i] + due_dates[seed_index]) / 2;
                if due_dates[i] < min_due {
                    due_dates[i] = min_due;
                } else if due_dates[i] > max_due {
                    due_dates[i] = max_due;
                }
            }
        }

        // For a random subset of customers, set nonzero ready times.
        let density = 0.5;
        let mut idxs: Vec<usize> = (1..=n).collect();
        idxs.shuffle(&mut rng);
        let threshold = ((n as f64) * density).round() as usize;
        for &cust_idx in idxs.iter().take(threshold) {
            let window = rng.gen_range(10..61);
            ready_times[cust_idx] = due_dates[cust_idx].saturating_sub(window);
        }
        let distance_matrix: Vec<Vec<i32>> = node_positions
            .iter()
            .map(|&from| {
                node_positions
                    .iter()
                    .map(|&to| {
                        let dx = from.0 - to.0;
                        let dy = from.1 - to.1;
                        dx.hypot(dy).round() as i32
                    })
                    .collect()
            })
            .collect();

        let baseline_routes =
            calc_baseline_routes(num_nodes, max_capacity, &demands, &distance_matrix)?;
        let baseline_routes_total_distance = calc_routes_total_distance(
            num_nodes,
            max_capacity,
            &demands,
            &distance_matrix,
            &baseline_routes,
        )?;
        let max_total_distance = (baseline_routes_total_distance
            * (1000 - difficulty.better_than_baseline as i32)
            / 1000) as i32;

        Ok(Challenge {
            seed,
            difficulty: difficulty.clone(),
            demands,
            distance_matrix,
            max_total_distance,
            max_capacity,
        })
    }

    fn verify_solution(&self, solution: &Solution) -> Result<()> {
        let total_distance = calc_routes_total_distance(
            self.difficulty.num_nodes,
            self.max_capacity,
            &self.demands,
            &self.distance_matrix,
            &solution.routes,
        )?;
        if total_distance <= self.max_total_distance {
            Ok(())
        } else {
            Err(anyhow!(
                "Total distance ({}) exceeds max total distance ({})",
                total_distance,
                self.max_total_distance
            ))
        }
    }
}

pub fn calc_baseline_routes(
    num_nodes: usize,
    max_capacity: i32,
    demands: &Vec<i32>,
    distance_matrix: &Vec<Vec<i32>>,
) -> Result<Vec<Vec<usize>>> {
    let mut routes = Vec::new();
    let mut visited = vec![false; num_nodes];
    visited[0] = true;

    while visited.iter().any(|&v| !v) {
        let mut route = vec![0];
        let mut current_node = 0;
        let mut capacity = max_capacity;

        while capacity > 0 && visited.iter().any(|&v| !v) {
            let eligible_nodes: Vec<usize> = (0..num_nodes)
                .filter(|&node| !visited[node] && demands[node] <= capacity)
                .collect();

            if !eligible_nodes.is_empty() {
                let &closest_node = eligible_nodes
                    .iter()
                    .min_by_key(|&&node| distance_matrix[current_node][node])
                    .unwrap();
                capacity -= demands[closest_node];
                route.push(closest_node);
                visited[closest_node] = true;
                current_node = closest_node;
            } else {
                break;
            }
        }

        route.push(0);
        routes.push(route);
    }

    Ok(routes)
}

pub fn calc_routes_total_distance(
    num_nodes: usize,
    max_capacity: i32,
    demands: &Vec<i32>,
    distance_matrix: &Vec<Vec<i32>>,
    routes: &Vec<Vec<usize>>,
) -> Result<i32> {
    let mut total_distance = 0;
    let mut visited = vec![false; num_nodes];
    visited[0] = true;

    for route in routes {
        if route.len() <= 2 || route[0] != 0 || route[route.len() - 1] != 0 {
            return Err(anyhow!("Each route must start and end at node 0 (the depot), and visit at least one non-depot node"));
        }

        let mut capacity = max_capacity;
        let mut current_node = 0;

        for &node in &route[1..route.len() - 1] {
            if visited[node] {
                return Err(anyhow!(
                    "The same non-depot node cannot be visited more than once"
                ));
            }
            if demands[node] > capacity {
                return Err(anyhow!(
                    "The total demand on each route must not exceed max capacity"
                ));
            }
            visited[node] = true;
            capacity -= demands[node];
            total_distance += distance_matrix[current_node][node];
            current_node = node;
        }

        total_distance += distance_matrix[current_node][0];
    }

    if visited.iter().any(|&v| !v) {
        return Err(anyhow!("All nodes must be visited"));
    }

    Ok(total_distance)
}
