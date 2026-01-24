//! Graph Coloring using 1-Bit Domain Propagation ☧
//!
//! Classic constraint satisfaction: color a graph such that
//! no two adjacent vertices share the same color.
//!
//! Run with: cargo run --example graph_color_chirho
//!
//! "And God saw every thing that he had made, and, behold,
//!  it was very good." — Genesis 1:31

use std::collections::HashMap;

// ============================================================================
// Graph Representation
// ============================================================================

/// A simple undirected graph
struct GraphChirho {
    /// Number of vertices
    n_chirho: usize,
    /// Adjacency list: vertex -> list of neighbors
    edges_chirho: Vec<Vec<usize>>,
}

impl GraphChirho {
    fn new_chirho(n_chirho: usize) -> Self {
        Self {
            n_chirho,
            edges_chirho: vec![Vec::new(); n_chirho],
        }
    }

    fn add_edge_chirho(&mut self, u_chirho: usize, v_chirho: usize) {
        if u_chirho != v_chirho {
            self.edges_chirho[u_chirho].push(v_chirho);
            self.edges_chirho[v_chirho].push(u_chirho);
        }
    }

    /// Create the Petersen graph (famous non-planar graph)
    fn petersen_chirho() -> Self {
        let mut g_chirho = Self::new_chirho(10);
        // Outer pentagon
        for i in 0..5 {
            g_chirho.add_edge_chirho(i, (i + 1) % 5);
        }
        // Inner pentagram
        for i in 0..5 {
            g_chirho.add_edge_chirho(5 + i, 5 + (i + 2) % 5);
        }
        // Spokes
        for i in 0..5 {
            g_chirho.add_edge_chirho(i, 5 + i);
        }
        g_chirho
    }

    /// Create a complete graph K_n
    fn complete_chirho(n_chirho: usize) -> Self {
        let mut g_chirho = Self::new_chirho(n_chirho);
        for i in 0..n_chirho {
            for j in (i + 1)..n_chirho {
                g_chirho.add_edge_chirho(i, j);
            }
        }
        g_chirho
    }

    /// Create a cycle graph C_n
    fn cycle_chirho(n_chirho: usize) -> Self {
        let mut g_chirho = Self::new_chirho(n_chirho);
        for i in 0..n_chirho {
            g_chirho.add_edge_chirho(i, (i + 1) % n_chirho);
        }
        g_chirho
    }
}

// ============================================================================
// Graph Coloring Solver
// ============================================================================

/// Solver using 1-bit domain propagation
struct GraphColorSolverChirho<'a> {
    graph_chirho: &'a GraphChirho,
    #[allow(dead_code)]
    k_chirho: usize,              // Number of colors
    domains_chirho: Vec<u32>,     // Bitmask of available colors per vertex
    solution_chirho: Vec<u8>,     // Current assignment
}

impl<'a> GraphColorSolverChirho<'a> {
    fn new_chirho(graph_chirho: &'a GraphChirho, k_chirho: usize) -> Self {
        assert!(k_chirho <= 32, "Max 32 colors supported");
        let full_domain_chirho = (1u32 << k_chirho) - 1;
        Self {
            graph_chirho,
            k_chirho,
            domains_chirho: vec![full_domain_chirho; graph_chirho.n_chirho],
            solution_chirho: vec![0; graph_chirho.n_chirho],
        }
    }

    /// Propagate constraints: if a vertex is colored, remove that color from neighbors
    fn propagate_chirho(&mut self) -> bool {
        let mut changed_chirho = true;
        while changed_chirho {
            changed_chirho = false;
            for v_chirho in 0..self.graph_chirho.n_chirho {
                let d_chirho = self.domains_chirho[v_chirho];

                // Check for failure
                if d_chirho == 0 {
                    return false;
                }

                // If singleton, propagate to neighbors
                if d_chirho.count_ones() == 1 {
                    for &neighbor_chirho in &self.graph_chirho.edges_chirho[v_chirho] {
                        let old_chirho = self.domains_chirho[neighbor_chirho];
                        let new_chirho = old_chirho & !d_chirho;
                        if new_chirho != old_chirho {
                            self.domains_chirho[neighbor_chirho] = new_chirho;
                            changed_chirho = true;
                        }
                    }
                }
            }
        }
        true
    }

    /// Find vertex with smallest domain > 1 (MRV heuristic)
    fn choose_vertex_chirho(&self) -> Option<usize> {
        let mut best_chirho: Option<(usize, u32)> = None;
        for v_chirho in 0..self.graph_chirho.n_chirho {
            let count_chirho = self.domains_chirho[v_chirho].count_ones();
            if count_chirho > 1 {
                if best_chirho.is_none() || count_chirho < best_chirho.unwrap().1 {
                    best_chirho = Some((v_chirho, count_chirho));
                }
            }
        }
        best_chirho.map(|(v, _)| v)
    }

    /// Solve with backtracking
    fn solve_chirho(&mut self) -> bool {
        if !self.propagate_chirho() {
            return false;
        }

        // Check if solved (all singletons)
        match self.choose_vertex_chirho() {
            None => {
                // All assigned - extract solution
                for v_chirho in 0..self.graph_chirho.n_chirho {
                    self.solution_chirho[v_chirho] = self.domains_chirho[v_chirho].trailing_zeros() as u8;
                }
                true
            }
            Some(v_chirho) => {
                // Branch on colors
                let domain_chirho = self.domains_chirho[v_chirho];
                let saved_chirho = self.domains_chirho.clone();

                let mut remaining_chirho = domain_chirho;
                while remaining_chirho != 0 {
                    let bit_chirho = remaining_chirho & remaining_chirho.wrapping_neg();
                    remaining_chirho &= remaining_chirho - 1;

                    // Try this color
                    self.domains_chirho[v_chirho] = bit_chirho;
                    if self.solve_chirho() {
                        return true;
                    }
                    // Backtrack
                    self.domains_chirho.clone_from(&saved_chirho);
                }
                false
            }
        }
    }

    /// Check if current solution is valid
    fn verify_chirho(&self) -> bool {
        for v_chirho in 0..self.graph_chirho.n_chirho {
            for &neighbor_chirho in &self.graph_chirho.edges_chirho[v_chirho] {
                if self.solution_chirho[v_chirho] == self.solution_chirho[neighbor_chirho] {
                    return false;
                }
            }
        }
        true
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("Graph Coloring with 1-Bit Domains ☧\n");

    // Example 1: Petersen graph (chromatic number = 3)
    println!("=== Petersen Graph (10 vertices) ===");
    let petersen_chirho = GraphChirho::petersen_chirho();

    for k_chirho in 1..=4 {
        let mut solver_chirho = GraphColorSolverChirho::new_chirho(&petersen_chirho, k_chirho);
        let t0_chirho = std::time::Instant::now();
        let found_chirho = solver_chirho.solve_chirho();
        let dt_chirho = t0_chirho.elapsed();

        if found_chirho {
            print!("{}-coloring found: ", k_chirho);
            println!("{:?}", solver_chirho.solution_chirho);
            println!("Valid: {}", solver_chirho.verify_chirho());
            println!("Time: {:?}", dt_chirho);
            break;
        } else {
            println!("{}-coloring: impossible", k_chirho);
        }
    }
    println!();

    // Example 2: Complete graph K_5 (needs 5 colors)
    println!("=== Complete Graph K_5 ===");
    let k5_chirho = GraphChirho::complete_chirho(5);
    for k_chirho in 1..=5 {
        let mut solver_chirho = GraphColorSolverChirho::new_chirho(&k5_chirho, k_chirho);
        if solver_chirho.solve_chirho() {
            println!("{}-coloring found: {:?}", k_chirho, solver_chirho.solution_chirho);
            break;
        } else {
            println!("{}-coloring: impossible", k_chirho);
        }
    }
    println!();

    // Example 3: Cycle graph C_7 (odd cycle needs 3 colors)
    println!("=== Cycle Graph C_7 ===");
    let c7_chirho = GraphChirho::cycle_chirho(7);
    for k_chirho in 2..=3 {
        let mut solver_chirho = GraphColorSolverChirho::new_chirho(&c7_chirho, k_chirho);
        if solver_chirho.solve_chirho() {
            println!("{}-coloring found: {:?}", k_chirho, solver_chirho.solution_chirho);
            break;
        } else {
            println!("{}-coloring: impossible", k_chirho);
        }
    }
    println!();

    // Example 4: Map coloring (Australia)
    println!("=== Australia Map Coloring ===");
    let states_chirho: HashMap<&str, usize> = [
        ("WA", 0), ("NT", 1), ("SA", 2), ("Q", 3),
        ("NSW", 4), ("V", 5), ("T", 6),
    ].into_iter().collect();

    let mut australia_chirho = GraphChirho::new_chirho(7);
    // Borders (adjacent states)
    let borders_chirho = [
        ("WA", "NT"), ("WA", "SA"),
        ("NT", "SA"), ("NT", "Q"),
        ("SA", "Q"), ("SA", "NSW"), ("SA", "V"),
        ("Q", "NSW"),
        ("NSW", "V"),
    ];
    for (a_chirho, b_chirho) in borders_chirho {
        australia_chirho.add_edge_chirho(states_chirho[a_chirho], states_chirho[b_chirho]);
    }

    let color_names_chirho = ["Red", "Green", "Blue", "Yellow"];
    let mut solver_chirho = GraphColorSolverChirho::new_chirho(&australia_chirho, 4);
    if solver_chirho.solve_chirho() {
        println!("4-coloring found:");
        for (name_chirho, &idx_chirho) in &states_chirho {
            println!("  {}: {}", name_chirho, color_names_chirho[solver_chirho.solution_chirho[idx_chirho] as usize]);
        }
    }

    println!("\nSoli Deo Gloria ☧");
}
