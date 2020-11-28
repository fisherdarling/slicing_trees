use std::io::{Write, stdout};
use rayon::prelude::*;

use slicing_trees::{SlicingTree, NPE, Rect};

fn p(delta_cost: f32, temp: f32) -> f32 {
    (-delta_cost / temp).exp()
}

fn simulated_annealing(npe: NPE, time: usize, k: usize, temp_epsilon: f32, temp_reduction: f32, rects: &[Rect]) -> NPE {
    let n = npe.count_operands() * k;
    
    let mut temp = 1.0;
    let mut best = npe;
    let mut best_cost = best.aabb(rects).cost();
    // let mut rejected = 0;
    
    for t in 0..time {
        let mut uphill = 0;
        let mut iters = 0;
        let mut rejected = 0;

        loop {
            iters += 1;

            let mut candidate = best.clone();
            candidate.perturb(1); // Get a neighbor

            let new_cost = candidate.aabb(rects).cost();
            let delta = new_cost - best_cost;
            
            if delta <= 0.0 || rand::random::<f32>() < p(delta, temp) {    
                if delta > 0.0 {
                    uphill += 1;
                }
                
                best = candidate;
                best_cost = new_cost;
            } else {
                rejected += 1;
            }

            if uphill > n || iters > 2 * n {
                break;
            }
            
        }
        
        // print!("Temp: {}\r", temp);
        // stdout().flush();

        temp *= temp_reduction;

        if rejected as f32 / iters as f32 > 0.95 || temp < temp_epsilon {
            // println!("Took {} iterations", t);
            break;
        }
    }

    best
}



fn main() {
    let tree = SlicingTree::random_tree(10_000, 10_000, 49);
    let mut npe = tree.postorder();

    let pre = npe.aabb(&tree.data);
    println!("{:?} -> {}: {}", pre, pre.cost(), npe);

    npe.perturb(1_000);

    let bad_aabb = npe.aabb(&tree.data);
    println!("{:?} -> {}: {}", bad_aabb, bad_aabb.cost(), npe);
    
    let best = (0..100).into_par_iter().map(|_| {
        simulated_annealing(npe.clone(), 1_000_000, 3, 0.05, 0.9999, &tree.data)
    }).min_by_key(|e| {
        e.aabb(&tree.data).cost() as isize
    }).unwrap();

    let best_aabb = best.aabb(&tree.data);
    // println!("{:?} -> {}: {}", best_aabb, best_aabb.cost(), best);
    println!();
    println!("Ratio: {:2.4}", best_aabb.cost() / pre.cost());
}
