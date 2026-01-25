//! Family Tree Benchmarks ☧
//!
//! Realistic miniKanren benchmark with large domains:
//! - 1k-100k people with parent relationships
//! - 10-12 generations deep
//! - Queries: ancestor, descendant, cousin, common_ancestor
//!
//! Run with: cargo bench --bench family_bench_chirho

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::{HashMap, HashSet, VecDeque};

use minikanren_1bit_chirho::hardware_chirho::BitVec64Chirho;
use minikanren_1bit_chirho::approaches_chirho::{
    Hierarchical4kChirho,
    PagedDomainChirho,
};

// ============================================================================
// Family Tree Data Structure
// ============================================================================

/// A person in the family tree
#[derive(Clone, Debug)]
struct PersonChirho {
    id_chirho: u32,
    generation_chirho: u32,
    parent1_chirho: Option<u32>,
    parent2_chirho: Option<u32>,
}

/// Family tree with efficient lookups
struct FamilyTreeChirho {
    people_chirho: Vec<PersonChirho>,
    /// parent_id -> [child_ids]
    children_chirho: HashMap<u32, Vec<u32>>,
    /// For each person, precomputed ancestors (up to depth limit)
    ancestor_cache_chirho: HashMap<u32, HashSet<u32>>,
}

impl FamilyTreeChirho {
    /// Generate a random family tree
    ///
    /// - `num_people`: Total number of people
    /// - `max_generations`: Maximum depth
    /// - `children_per_couple`: Average children per parent pair
    fn generate_chirho(
        num_people_chirho: u32,
        max_generations_chirho: u32,
        seed_chirho: u64,
    ) -> Self {
        let mut people_chirho = Vec::with_capacity(num_people_chirho as usize);
        let mut children_chirho: HashMap<u32, Vec<u32>> = HashMap::new();

        // Simple LCG random number generator
        let mut rng_chirho = seed_chirho;
        let mut next_rand_chirho = || {
            rng_chirho = rng_chirho.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            rng_chirho
        };

        // Generation 0: founders (no parents)
        let founders_per_gen_chirho = (num_people_chirho / max_generations_chirho).max(2);
        let mut current_gen_chirho: Vec<u32> = Vec::new();

        for i in 0..founders_per_gen_chirho.min(num_people_chirho) {
            people_chirho.push(PersonChirho {
                id_chirho: i,
                generation_chirho: 0,
                parent1_chirho: None,
                parent2_chirho: None,
            });
            current_gen_chirho.push(i);
        }

        let mut next_id_chirho = current_gen_chirho.len() as u32;

        // Generate subsequent generations
        for gen_chirho in 1..max_generations_chirho {
            if next_id_chirho >= num_people_chirho {
                break;
            }

            let mut next_gen_chirho: Vec<u32> = Vec::new();

            // Pair up people from current generation to have children
            let pairs_chirho = current_gen_chirho.len() / 2;

            for pair_idx_chirho in 0..pairs_chirho {
                if next_id_chirho >= num_people_chirho {
                    break;
                }

                let parent1_chirho = current_gen_chirho[pair_idx_chirho * 2];
                let parent2_chirho = current_gen_chirho[pair_idx_chirho * 2 + 1];

                // Random number of children (1-4)
                let num_children_chirho = ((next_rand_chirho() % 4) + 1) as u32;

                for _ in 0..num_children_chirho {
                    if next_id_chirho >= num_people_chirho {
                        break;
                    }

                    let child_id_chirho = next_id_chirho;
                    next_id_chirho += 1;

                    people_chirho.push(PersonChirho {
                        id_chirho: child_id_chirho,
                        generation_chirho: gen_chirho,
                        parent1_chirho: Some(parent1_chirho),
                        parent2_chirho: Some(parent2_chirho),
                    });

                    children_chirho.entry(parent1_chirho)
                        .or_default()
                        .push(child_id_chirho);
                    children_chirho.entry(parent2_chirho)
                        .or_default()
                        .push(child_id_chirho);

                    next_gen_chirho.push(child_id_chirho);
                }
            }

            // Also add some people with only one known parent (variation)
            let single_parent_count_chirho = (current_gen_chirho.len() / 4).max(1);
            for i in 0..single_parent_count_chirho {
                if next_id_chirho >= num_people_chirho {
                    break;
                }

                let parent_chirho = current_gen_chirho[i % current_gen_chirho.len()];
                let child_id_chirho = next_id_chirho;
                next_id_chirho += 1;

                people_chirho.push(PersonChirho {
                    id_chirho: child_id_chirho,
                    generation_chirho: gen_chirho,
                    parent1_chirho: Some(parent_chirho),
                    parent2_chirho: None,
                });

                children_chirho.entry(parent_chirho)
                    .or_default()
                    .push(child_id_chirho);

                next_gen_chirho.push(child_id_chirho);
            }

            current_gen_chirho = next_gen_chirho;
        }

        // Build ancestor cache (BFS from each person)
        let mut ancestor_cache_chirho = HashMap::new();
        for person_chirho in &people_chirho {
            let mut ancestors_chirho = HashSet::new();
            let mut queue_chirho = VecDeque::new();

            if let Some(p1) = person_chirho.parent1_chirho {
                queue_chirho.push_back(p1);
            }
            if let Some(p2) = person_chirho.parent2_chirho {
                queue_chirho.push_back(p2);
            }

            while let Some(ancestor_chirho) = queue_chirho.pop_front() {
                if ancestors_chirho.insert(ancestor_chirho) {
                    let anc_person_chirho = &people_chirho[ancestor_chirho as usize];
                    if let Some(p1) = anc_person_chirho.parent1_chirho {
                        queue_chirho.push_back(p1);
                    }
                    if let Some(p2) = anc_person_chirho.parent2_chirho {
                        queue_chirho.push_back(p2);
                    }
                }
            }

            ancestor_cache_chirho.insert(person_chirho.id_chirho, ancestors_chirho);
        }

        Self {
            people_chirho,
            children_chirho,
            ancestor_cache_chirho,
        }
    }

    fn num_people_chirho(&self) -> u32 {
        self.people_chirho.len() as u32
    }

    /// Get all ancestors of a person (using cache)
    fn ancestors_chirho(&self, person_id_chirho: u32) -> &HashSet<u32> {
        self.ancestor_cache_chirho.get(&person_id_chirho)
            .unwrap_or(&EMPTY_SET_CHIRHO)
    }

    /// Get all descendants of a person (computed on demand)
    fn descendants_chirho(&self, person_id_chirho: u32) -> HashSet<u32> {
        let mut result_chirho = HashSet::new();
        let mut queue_chirho = VecDeque::new();

        if let Some(children) = self.children_chirho.get(&person_id_chirho) {
            for &child in children {
                queue_chirho.push_back(child);
            }
        }

        while let Some(desc_chirho) = queue_chirho.pop_front() {
            if result_chirho.insert(desc_chirho) {
                if let Some(children) = self.children_chirho.get(&desc_chirho) {
                    for &child in children {
                        queue_chirho.push_back(child);
                    }
                }
            }
        }

        result_chirho
    }

    /// Find common ancestors of two people
    fn common_ancestors_chirho(&self, p1_chirho: u32, p2_chirho: u32) -> HashSet<u32> {
        let anc1_chirho = self.ancestors_chirho(p1_chirho);
        let anc2_chirho = self.ancestors_chirho(p2_chirho);
        anc1_chirho.intersection(anc2_chirho).copied().collect()
    }
}

use std::sync::LazyLock;

static EMPTY_SET_CHIRHO: LazyLock<HashSet<u32>> = LazyLock::new(HashSet::new);

// ============================================================================
// Domain Representations for Family Queries
// ============================================================================

/// Convert HashSet to Hierarchical4k domain
fn hashset_to_hierarchical_chirho(set_chirho: &HashSet<u32>) -> Hierarchical4kChirho {
    let mut result_chirho = Hierarchical4kChirho::empty_chirho();

    for &id_chirho in set_chirho {
        if id_chirho < 4096 {
            let leaf_idx_chirho = (id_chirho / 64) as usize;
            let bit_idx_chirho = id_chirho % 64;
            result_chirho.leaves_chirho[leaf_idx_chirho].0 |= 1 << bit_idx_chirho;
            result_chirho.root_chirho.0 |= 1 << leaf_idx_chirho;
        }
    }

    result_chirho
}

/// Convert HashSet to PagedDomain
fn hashset_to_paged_chirho(set_chirho: &HashSet<u32>) -> PagedDomainChirho {
    let mut result_chirho = PagedDomainChirho::empty_chirho();
    for &id_chirho in set_chirho {
        result_chirho.insert_chirho(id_chirho as u64);
    }
    result_chirho
}

// ============================================================================
// Benchmark 1: Ancestor Query
// ============================================================================

fn bench_ancestor_query_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("FamilyTree/Ancestor");

    for num_people_chirho in [100, 500, 1000, 2000, 4000] {
        let tree_chirho = FamilyTreeChirho::generate_chirho(num_people_chirho, 10, 12345);

        // Query: "Is person X an ancestor of person Y?"
        // Pick a person from middle generation
        let query_person_chirho = num_people_chirho / 2;

        // HashSet baseline
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet", num_people_chirho),
            &num_people_chirho,
            |bench_chirho, _| {
                let ancestors_chirho = tree_chirho.ancestors_chirho(query_person_chirho);
                let candidate_chirho = num_people_chirho / 4; // Someone who might be ancestor
                bench_chirho.iter(|| {
                    black_box(ancestors_chirho.contains(&candidate_chirho))
                })
            },
        );

        // Hierarchical4k (if fits)
        if num_people_chirho <= 4096 {
            group_chirho.bench_with_input(
                BenchmarkId::new("Hierarchical4k", num_people_chirho),
                &num_people_chirho,
                |bench_chirho, _| {
                    let ancestors_chirho = tree_chirho.ancestors_chirho(query_person_chirho);
                    let domain_chirho = hashset_to_hierarchical_chirho(ancestors_chirho);
                    let candidate_chirho = num_people_chirho / 4;
                    bench_chirho.iter(|| {
                        black_box(domain_chirho.contains_chirho(candidate_chirho))
                    })
                },
            );
        }
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 2: Common Ancestor Query (Intersection)
// ============================================================================

fn bench_common_ancestor_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("FamilyTree/CommonAncestor");

    for num_people_chirho in [100, 500, 1000, 2000, 4000] {
        let tree_chirho = FamilyTreeChirho::generate_chirho(num_people_chirho, 10, 12345);

        // Query: Find common ancestors of two people
        let person1_chirho = num_people_chirho * 3 / 4;
        let person2_chirho = num_people_chirho * 7 / 8;

        // HashSet intersection baseline
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet_intersect", num_people_chirho),
            &num_people_chirho,
            |bench_chirho, _| {
                let anc1_chirho = tree_chirho.ancestors_chirho(person1_chirho);
                let anc2_chirho = tree_chirho.ancestors_chirho(person2_chirho);
                bench_chirho.iter(|| {
                    let common_chirho: HashSet<u32> = anc1_chirho
                        .intersection(anc2_chirho)
                        .copied()
                        .collect();
                    black_box(common_chirho.len())
                })
            },
        );

        // Hierarchical4k intersection
        if num_people_chirho <= 4096 {
            group_chirho.bench_with_input(
                BenchmarkId::new("Hierarchical4k_intersect", num_people_chirho),
                &num_people_chirho,
                |bench_chirho, _| {
                    let anc1_chirho = tree_chirho.ancestors_chirho(person1_chirho);
                    let anc2_chirho = tree_chirho.ancestors_chirho(person2_chirho);
                    let dom1_chirho = hashset_to_hierarchical_chirho(anc1_chirho);
                    let dom2_chirho = hashset_to_hierarchical_chirho(anc2_chirho);
                    bench_chirho.iter(|| {
                        let common_chirho = dom1_chirho.intersect_chirho(&dom2_chirho);
                        black_box(common_chirho.count_chirho())
                    })
                },
            );
        }

        // Paged domain intersection
        group_chirho.bench_with_input(
            BenchmarkId::new("Paged_intersect", num_people_chirho),
            &num_people_chirho,
            |bench_chirho, _| {
                let anc1_chirho = tree_chirho.ancestors_chirho(person1_chirho);
                let anc2_chirho = tree_chirho.ancestors_chirho(person2_chirho);
                let dom1_chirho = hashset_to_paged_chirho(anc1_chirho);
                let dom2_chirho = hashset_to_paged_chirho(anc2_chirho);
                bench_chirho.iter(|| {
                    let common_chirho = dom1_chirho.intersect_chirho(&dom2_chirho);
                    black_box(common_chirho.count_chirho())
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 3: Descendant Enumeration
// ============================================================================

fn bench_descendant_enum_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("FamilyTree/Descendants");

    for num_people_chirho in [100, 500, 1000, 2000] {
        let tree_chirho = FamilyTreeChirho::generate_chirho(num_people_chirho, 10, 12345);

        // Query: Enumerate all descendants of a founder
        let founder_chirho = 0u32;
        let descendants_chirho = tree_chirho.descendants_chirho(founder_chirho);

        // HashSet iteration
        group_chirho.bench_with_input(
            BenchmarkId::new("HashSet_iter", num_people_chirho),
            &num_people_chirho,
            |bench_chirho, _| {
                bench_chirho.iter(|| {
                    let sum_chirho: u32 = descendants_chirho.iter().sum();
                    black_box(sum_chirho)
                })
            },
        );

        // Hierarchical4k iteration
        if num_people_chirho <= 4096 {
            let domain_chirho = hashset_to_hierarchical_chirho(&descendants_chirho);
            group_chirho.bench_with_input(
                BenchmarkId::new("Hierarchical4k_iter", num_people_chirho),
                &num_people_chirho,
                |bench_chirho, _| {
                    bench_chirho.iter(|| {
                        let sum_chirho: u32 = domain_chirho.iter_chirho().sum();
                        black_box(sum_chirho)
                    })
                },
            );
        }
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 4: Relational Join (Who are cousins?)
// ============================================================================

fn bench_cousin_query_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("FamilyTree/Cousins");

    for num_people_chirho in [100, 500, 1000] {
        let tree_chirho = FamilyTreeChirho::generate_chirho(num_people_chirho, 8, 12345);

        // Query: Find all first cousins of a person
        // (Children of parent's siblings)
        let query_person_chirho = num_people_chirho / 2;

        group_chirho.bench_with_input(
            BenchmarkId::new("Naive", num_people_chirho),
            &num_people_chirho,
            |bench_chirho, _| {
                bench_chirho.iter(|| {
                    let person_chirho = &tree_chirho.people_chirho[query_person_chirho as usize];
                    let mut cousins_chirho = HashSet::new();

                    // Get grandparents
                    let mut grandparents_chirho = HashSet::new();
                    for parent_id_chirho in [person_chirho.parent1_chirho, person_chirho.parent2_chirho].into_iter().flatten() {
                        let parent_chirho = &tree_chirho.people_chirho[parent_id_chirho as usize];
                        if let Some(gp) = parent_chirho.parent1_chirho {
                            grandparents_chirho.insert(gp);
                        }
                        if let Some(gp) = parent_chirho.parent2_chirho {
                            grandparents_chirho.insert(gp);
                        }
                    }

                    // Get aunts/uncles (grandparents' children except parents)
                    let mut parents_chirho = HashSet::new();
                    if let Some(p) = person_chirho.parent1_chirho {
                        parents_chirho.insert(p);
                    }
                    if let Some(p) = person_chirho.parent2_chirho {
                        parents_chirho.insert(p);
                    }

                    for &gp_chirho in &grandparents_chirho {
                        if let Some(children) = tree_chirho.children_chirho.get(&gp_chirho) {
                            for &aunt_uncle_chirho in children {
                                if !parents_chirho.contains(&aunt_uncle_chirho) {
                                    // Get their children (cousins)
                                    if let Some(cousin_list) = tree_chirho.children_chirho.get(&aunt_uncle_chirho) {
                                        for &cousin_chirho in cousin_list {
                                            cousins_chirho.insert(cousin_chirho);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    black_box(cousins_chirho.len())
                })
            },
        );
    }

    group_chirho.finish();
}

// ============================================================================
// Benchmark 5: Path Query (Is X related to Y within N generations?)
// ============================================================================

fn bench_related_within_chirho(c_chirho: &mut Criterion) {
    let mut group_chirho = c_chirho.benchmark_group("FamilyTree/RelatedWithin");

    for num_people_chirho in [500, 1000, 2000] {
        let tree_chirho = FamilyTreeChirho::generate_chirho(num_people_chirho, 10, 12345);

        // Query: Are persons X and Y related within 4 generations?
        let person1_chirho = num_people_chirho / 3;
        let person2_chirho = num_people_chirho * 2 / 3;

        group_chirho.bench_with_input(
            BenchmarkId::new("BFS", num_people_chirho),
            &num_people_chirho,
            |bench_chirho, _| {
                bench_chirho.iter(|| {
                    // BFS from both ends, meet in middle
                    let mut visited1_chirho = HashSet::new();
                    let mut visited2_chirho = HashSet::new();
                    let mut frontier1_chirho = vec![person1_chirho];
                    let mut frontier2_chirho = vec![person2_chirho];

                    for _depth_chirho in 0..4 {
                        // Expand frontier 1
                        let mut new_frontier1_chirho = Vec::new();
                        for &p_chirho in &frontier1_chirho {
                            if visited1_chirho.insert(p_chirho) {
                                let person_chirho = &tree_chirho.people_chirho[p_chirho as usize];
                                if let Some(p1) = person_chirho.parent1_chirho {
                                    new_frontier1_chirho.push(p1);
                                }
                                if let Some(p2) = person_chirho.parent2_chirho {
                                    new_frontier1_chirho.push(p2);
                                }
                                if let Some(children) = tree_chirho.children_chirho.get(&p_chirho) {
                                    new_frontier1_chirho.extend(children);
                                }
                            }
                        }
                        frontier1_chirho = new_frontier1_chirho;

                        // Check intersection
                        if frontier1_chirho.iter().any(|p| visited2_chirho.contains(p)) {
                            return black_box(true);
                        }

                        // Expand frontier 2
                        let mut new_frontier2_chirho = Vec::new();
                        for &p_chirho in &frontier2_chirho {
                            if visited2_chirho.insert(p_chirho) {
                                let person_chirho = &tree_chirho.people_chirho[p_chirho as usize];
                                if let Some(p1) = person_chirho.parent1_chirho {
                                    new_frontier2_chirho.push(p1);
                                }
                                if let Some(p2) = person_chirho.parent2_chirho {
                                    new_frontier2_chirho.push(p2);
                                }
                                if let Some(children) = tree_chirho.children_chirho.get(&p_chirho) {
                                    new_frontier2_chirho.extend(children);
                                }
                            }
                        }
                        frontier2_chirho = new_frontier2_chirho;

                        // Check intersection
                        if frontier2_chirho.iter().any(|p| visited1_chirho.contains(p)) {
                            return black_box(true);
                        }
                    }

                    black_box(false)
                })
            },
        );

        // Domain intersection approach
        if num_people_chirho <= 4096 {
            group_chirho.bench_with_input(
                BenchmarkId::new("DomainIntersect", num_people_chirho),
                &num_people_chirho,
                |bench_chirho, _| {
                    // Precompute: all people reachable from person1 within 4 gens
                    let mut reachable1_chirho = HashSet::new();
                    let mut frontier_chirho = vec![person1_chirho];
                    for _ in 0..4 {
                        let mut next_chirho = Vec::new();
                        for &p in &frontier_chirho {
                            if reachable1_chirho.insert(p) {
                                let person = &tree_chirho.people_chirho[p as usize];
                                if let Some(p1) = person.parent1_chirho {
                                    next_chirho.push(p1);
                                }
                                if let Some(p2) = person.parent2_chirho {
                                    next_chirho.push(p2);
                                }
                                if let Some(children) = tree_chirho.children_chirho.get(&p) {
                                    next_chirho.extend(children);
                                }
                            }
                        }
                        frontier_chirho = next_chirho;
                    }

                    let mut reachable2_chirho = HashSet::new();
                    let mut frontier_chirho = vec![person2_chirho];
                    for _ in 0..4 {
                        let mut next_chirho = Vec::new();
                        for &p in &frontier_chirho {
                            if reachable2_chirho.insert(p) {
                                let person = &tree_chirho.people_chirho[p as usize];
                                if let Some(p1) = person.parent1_chirho {
                                    next_chirho.push(p1);
                                }
                                if let Some(p2) = person.parent2_chirho {
                                    next_chirho.push(p2);
                                }
                                if let Some(children) = tree_chirho.children_chirho.get(&p) {
                                    next_chirho.extend(children);
                                }
                            }
                        }
                        frontier_chirho = next_chirho;
                    }

                    let dom1_chirho = hashset_to_hierarchical_chirho(&reachable1_chirho);
                    let dom2_chirho = hashset_to_hierarchical_chirho(&reachable2_chirho);

                    bench_chirho.iter(|| {
                        let intersection_chirho = dom1_chirho.intersect_chirho(&dom2_chirho);
                        black_box(!intersection_chirho.is_empty_chirho())
                    })
                },
            );
        }
    }

    group_chirho.finish();
}

criterion_group!(
    benches_chirho,
    bench_ancestor_query_chirho,
    bench_common_ancestor_chirho,
    bench_descendant_enum_chirho,
    bench_cousin_query_chirho,
    bench_related_within_chirho,
);

criterion_main!(benches_chirho);
