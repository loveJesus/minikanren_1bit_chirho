//! Zebra Puzzle (Einstein's Riddle) using 1-Bit Domains ☧
//!
//! The classic logic puzzle: 5 houses, 5 attributes each.
//! Who owns the zebra? Who drinks water?
//!
//! Run with: cargo run --example zebra_chirho
//!
//! "The fear of the Lord is the beginning of wisdom:
//!  and the knowledge of the holy is understanding." — Proverbs 9:10


// ============================================================================
// Constants
// ============================================================================

const N_CHIRHO: usize = 5; // 5 houses, 5 values per attribute

// Attribute indices
const COLOR_CHIRHO: usize = 0;
const NATION_CHIRHO: usize = 1;
const DRINK_CHIRHO: usize = 2;
const SMOKE_CHIRHO: usize = 3;
const PET_CHIRHO: usize = 4;
const N_ATTRS_CHIRHO: usize = 5;

// Color values
const YELLOW_CHIRHO: u8 = 0;
const BLUE_CHIRHO: u8 = 1;
const RED_CHIRHO: u8 = 2;
const IVORY_CHIRHO: u8 = 3;
const GREEN_CHIRHO: u8 = 4;

// Nationality values
const NORWEGIAN_CHIRHO: u8 = 0;
const UKRAINIAN_CHIRHO: u8 = 1;
const ENGLISHMAN_CHIRHO: u8 = 2;
const SPANIARD_CHIRHO: u8 = 3;
const JAPANESE_CHIRHO: u8 = 4;

// Drink values
const WATER_CHIRHO: u8 = 0;
const TEA_CHIRHO: u8 = 1;
const MILK_CHIRHO: u8 = 2;
const OJ_CHIRHO: u8 = 3;
const COFFEE_CHIRHO: u8 = 4;

// Smoke values
const KOOLS_CHIRHO: u8 = 0;
const CHESTERFIELD_CHIRHO: u8 = 1;
const OLD_GOLD_CHIRHO: u8 = 2;
const LUCKY_STRIKE_CHIRHO: u8 = 3;
const PARLIAMENT_CHIRHO: u8 = 4;

// Pet values
const FOX_CHIRHO: u8 = 0;
const HORSE_CHIRHO: u8 = 1;
const SNAILS_CHIRHO: u8 = 2;
const DOG_CHIRHO: u8 = 3;
const ZEBRA_CHIRHO: u8 = 4;

// ============================================================================
// Value Names (for display)
// ============================================================================

const COLOR_NAMES_CHIRHO: [&str; 5] = ["Yellow", "Blue", "Red", "Ivory", "Green"];
const NATION_NAMES_CHIRHO: [&str; 5] = ["Norwegian", "Ukrainian", "Englishman", "Spaniard", "Japanese"];
const DRINK_NAMES_CHIRHO: [&str; 5] = ["Water", "Tea", "Milk", "OJ", "Coffee"];
const SMOKE_NAMES_CHIRHO: [&str; 5] = ["Kools", "Chesterfield", "OldGold", "LuckyStrike", "Parliament"];
const PET_NAMES_CHIRHO: [&str; 5] = ["Fox", "Horse", "Snails", "Dog", "Zebra"];

// ============================================================================
// Solver State
// ============================================================================

/// State: 5 houses × 5 attributes, each with 5-bit domain
struct ZebraStateChirho {
    /// domains[house][attr] = bitmask of possible values
    domains_chirho: [[u8; N_ATTRS_CHIRHO]; N_CHIRHO],
    /// For each (attr, value), which houses can have it?
    value_domains_chirho: [[u8; N_CHIRHO]; N_ATTRS_CHIRHO],
}

impl ZebraStateChirho {
    fn new_chirho() -> Self {
        Self {
            domains_chirho: [[0b11111; N_ATTRS_CHIRHO]; N_CHIRHO],
            value_domains_chirho: [[0b11111; N_CHIRHO]; N_ATTRS_CHIRHO],
        }
    }

    /// Assign value to house.attr, propagate constraints
    fn assign_chirho(&mut self, house_chirho: usize, attr_chirho: usize, val_chirho: u8) -> bool {
        let bit_chirho = 1u8 << val_chirho;

        // This house gets exactly this value
        self.domains_chirho[house_chirho][attr_chirho] = bit_chirho;

        // No other house can have this value for this attribute
        for h_chirho in 0..N_CHIRHO {
            if h_chirho != house_chirho {
                self.domains_chirho[h_chirho][attr_chirho] &= !bit_chirho;
                if self.domains_chirho[h_chirho][attr_chirho] == 0 {
                    return false;
                }
            }
        }

        // This house can't have other values for this attribute
        for v_chirho in 0..N_CHIRHO as u8 {
            if v_chirho != val_chirho {
                self.value_domains_chirho[attr_chirho][v_chirho as usize] &= !(1u8 << house_chirho);
            }
        }
        self.value_domains_chirho[attr_chirho][val_chirho as usize] = 1u8 << house_chirho;

        true
    }

    /// Constrain: if house has attr1=val1, it must have attr2=val2
    fn same_house_chirho(&mut self, attr1_chirho: usize, val1_chirho: u8, attr2_chirho: usize, val2_chirho: u8) {
        // Find houses where both are possible
        let houses1_chirho = self.value_domains_chirho[attr1_chirho][val1_chirho as usize];
        let houses2_chirho = self.value_domains_chirho[attr2_chirho][val2_chirho as usize];
        let common_chirho = houses1_chirho & houses2_chirho;

        // Restrict both to only common houses
        self.value_domains_chirho[attr1_chirho][val1_chirho as usize] = common_chirho;
        self.value_domains_chirho[attr2_chirho][val2_chirho as usize] = common_chirho;

        // Update house domains
        for h_chirho in 0..N_CHIRHO {
            let bit_chirho = 1u8 << h_chirho;
            if (common_chirho & bit_chirho) == 0 {
                self.domains_chirho[h_chirho][attr1_chirho] &= !(1u8 << val1_chirho);
                self.domains_chirho[h_chirho][attr2_chirho] &= !(1u8 << val2_chirho);
            }
        }
    }

    /// Constrain: next_to (houses differ by 1)
    fn next_to_chirho(&mut self, attr1_chirho: usize, val1_chirho: u8, attr2_chirho: usize, val2_chirho: u8) {
        let houses1_chirho = self.value_domains_chirho[attr1_chirho][val1_chirho as usize];
        let houses2_chirho = self.value_domains_chirho[attr2_chirho][val2_chirho as usize];

        // For each house in houses1, check if adjacent house is in houses2
        let mut valid1_chirho = 0u8;
        let mut valid2_chirho = 0u8;

        for h1_chirho in 0..N_CHIRHO {
            if (houses1_chirho & (1u8 << h1_chirho)) != 0 {
                // Check neighbors
                for h2_chirho in 0..N_CHIRHO {
                    if (houses2_chirho & (1u8 << h2_chirho)) != 0 {
                        let diff_chirho = (h1_chirho as i32 - h2_chirho as i32).abs();
                        if diff_chirho == 1 {
                            valid1_chirho |= 1u8 << h1_chirho;
                            valid2_chirho |= 1u8 << h2_chirho;
                        }
                    }
                }
            }
        }

        self.value_domains_chirho[attr1_chirho][val1_chirho as usize] &= valid1_chirho;
        self.value_domains_chirho[attr2_chirho][val2_chirho as usize] &= valid2_chirho;
    }

    /// Constrain: left_of (house1 = house2 - 1)
    fn left_of_chirho(&mut self, attr1_chirho: usize, val1_chirho: u8, attr2_chirho: usize, val2_chirho: u8) {
        let houses1_chirho = self.value_domains_chirho[attr1_chirho][val1_chirho as usize];
        let houses2_chirho = self.value_domains_chirho[attr2_chirho][val2_chirho as usize];

        let mut valid1_chirho = 0u8;
        let mut valid2_chirho = 0u8;

        for h1_chirho in 0..(N_CHIRHO - 1) {
            if (houses1_chirho & (1u8 << h1_chirho)) != 0 {
                let h2_chirho = h1_chirho + 1;
                if (houses2_chirho & (1u8 << h2_chirho)) != 0 {
                    valid1_chirho |= 1u8 << h1_chirho;
                    valid2_chirho |= 1u8 << h2_chirho;
                }
            }
        }

        self.value_domains_chirho[attr1_chirho][val1_chirho as usize] &= valid1_chirho;
        self.value_domains_chirho[attr2_chirho][val2_chirho as usize] &= valid2_chirho;
    }

    /// Arc consistency propagation
    fn propagate_chirho(&mut self) -> bool {
        let mut changed_chirho = true;
        while changed_chirho {
            changed_chirho = false;

            // For each attribute/value, if only one house possible, assign
            for attr_chirho in 0..N_ATTRS_CHIRHO {
                for val_chirho in 0..N_CHIRHO {
                    let houses_chirho = self.value_domains_chirho[attr_chirho][val_chirho];
                    if houses_chirho == 0 {
                        return false; // No house can have this value
                    }
                    if houses_chirho.count_ones() == 1 {
                        let h_chirho = houses_chirho.trailing_zeros() as usize;
                        if self.domains_chirho[h_chirho][attr_chirho].count_ones() > 1 {
                            if !self.assign_chirho(h_chirho, attr_chirho, val_chirho as u8) {
                                return false;
                            }
                            changed_chirho = true;
                        }
                    }
                }
            }

            // For each house/attr, if only one value possible, assign
            for h_chirho in 0..N_CHIRHO {
                for attr_chirho in 0..N_ATTRS_CHIRHO {
                    let vals_chirho = self.domains_chirho[h_chirho][attr_chirho];
                    if vals_chirho == 0 {
                        return false;
                    }
                    if vals_chirho.count_ones() == 1 {
                        let v_chirho = vals_chirho.trailing_zeros() as u8;
                        // Ensure other houses don't have this value
                        for other_chirho in 0..N_CHIRHO {
                            if other_chirho != h_chirho {
                                self.domains_chirho[other_chirho][attr_chirho] &= !(1u8 << v_chirho);
                                if self.domains_chirho[other_chirho][attr_chirho] == 0 {
                                    return false;
                                }
                            }
                        }
                        self.value_domains_chirho[attr_chirho][v_chirho as usize] = 1u8 << h_chirho;
                    }
                }
            }
        }
        true
    }

    /// Find first undetermined cell
    fn choose_cell_chirho(&self) -> Option<(usize, usize)> {
        for h_chirho in 0..N_CHIRHO {
            for attr_chirho in 0..N_ATTRS_CHIRHO {
                if self.domains_chirho[h_chirho][attr_chirho].count_ones() > 1 {
                    return Some((h_chirho, attr_chirho));
                }
            }
        }
        None
    }

    /// Solve with backtracking
    fn solve_chirho(&mut self) -> bool {
        if !self.propagate_chirho() {
            return false;
        }

        match self.choose_cell_chirho() {
            None => true, // Solved
            Some((h_chirho, attr_chirho)) => {
                let domain_chirho = self.domains_chirho[h_chirho][attr_chirho];
                let saved_chirho = self.clone();

                for val_chirho in 0..N_CHIRHO as u8 {
                    if (domain_chirho & (1u8 << val_chirho)) != 0 {
                        if self.assign_chirho(h_chirho, attr_chirho, val_chirho) && self.solve_chirho() {
                            return true;
                        }
                        *self = saved_chirho.clone();
                    }
                }
                false
            }
        }
    }

    fn print_solution_chirho(&self) {
        println!("\nSolution:");
        println!("{:>10} {:>12} {:>10} {:>14} {:>8}", "House", "Color", "Nation", "Drink", "Smoke", );
        for h_chirho in 0..N_CHIRHO {
            let color_chirho = self.domains_chirho[h_chirho][COLOR_CHIRHO].trailing_zeros() as usize;
            let nation_chirho = self.domains_chirho[h_chirho][NATION_CHIRHO].trailing_zeros() as usize;
            let drink_chirho = self.domains_chirho[h_chirho][DRINK_CHIRHO].trailing_zeros() as usize;
            let smoke_chirho = self.domains_chirho[h_chirho][SMOKE_CHIRHO].trailing_zeros() as usize;
            let pet_chirho = self.domains_chirho[h_chirho][PET_CHIRHO].trailing_zeros() as usize;

            println!("{:>10} {:>12} {:>10} {:>14} {:>8} {:>8}",
                h_chirho + 1,
                COLOR_NAMES_CHIRHO[color_chirho],
                NATION_NAMES_CHIRHO[nation_chirho],
                DRINK_NAMES_CHIRHO[drink_chirho],
                SMOKE_NAMES_CHIRHO[smoke_chirho],
                PET_NAMES_CHIRHO[pet_chirho],
            );
        }
    }
}

impl Clone for ZebraStateChirho {
    fn clone(&self) -> Self {
        Self {
            domains_chirho: self.domains_chirho,
            value_domains_chirho: self.value_domains_chirho,
        }
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("Zebra Puzzle (Einstein's Riddle) ☧\n");
    println!("15 clues, 5 houses, 25 variables\n");

    let mut state_chirho = ZebraStateChirho::new_chirho();

    // Apply the 15 clues:

    // 1. The Englishman lives in the red house
    state_chirho.same_house_chirho(NATION_CHIRHO, ENGLISHMAN_CHIRHO, COLOR_CHIRHO, RED_CHIRHO);

    // 2. The Spaniard owns the dog
    state_chirho.same_house_chirho(NATION_CHIRHO, SPANIARD_CHIRHO, PET_CHIRHO, DOG_CHIRHO);

    // 3. Coffee is drunk in the green house
    state_chirho.same_house_chirho(DRINK_CHIRHO, COFFEE_CHIRHO, COLOR_CHIRHO, GREEN_CHIRHO);

    // 4. The Ukrainian drinks tea
    state_chirho.same_house_chirho(NATION_CHIRHO, UKRAINIAN_CHIRHO, DRINK_CHIRHO, TEA_CHIRHO);

    // 5. The green house is immediately to the right of the ivory house
    state_chirho.left_of_chirho(COLOR_CHIRHO, IVORY_CHIRHO, COLOR_CHIRHO, GREEN_CHIRHO);

    // 6. The Old Gold smoker owns snails
    state_chirho.same_house_chirho(SMOKE_CHIRHO, OLD_GOLD_CHIRHO, PET_CHIRHO, SNAILS_CHIRHO);

    // 7. Kools are smoked in the yellow house
    state_chirho.same_house_chirho(SMOKE_CHIRHO, KOOLS_CHIRHO, COLOR_CHIRHO, YELLOW_CHIRHO);

    // 8. Milk is drunk in the middle house (house 3, 0-indexed: 2)
    state_chirho.assign_chirho(2, DRINK_CHIRHO, MILK_CHIRHO);

    // 9. The Norwegian lives in the first house (house 1, 0-indexed: 0)
    state_chirho.assign_chirho(0, NATION_CHIRHO, NORWEGIAN_CHIRHO);

    // 10. The Chesterfield smoker lives next to the fox owner
    state_chirho.next_to_chirho(SMOKE_CHIRHO, CHESTERFIELD_CHIRHO, PET_CHIRHO, FOX_CHIRHO);

    // 11. Kools are smoked next to the horse owner
    state_chirho.next_to_chirho(SMOKE_CHIRHO, KOOLS_CHIRHO, PET_CHIRHO, HORSE_CHIRHO);

    // 12. The Lucky Strike smoker drinks orange juice
    state_chirho.same_house_chirho(SMOKE_CHIRHO, LUCKY_STRIKE_CHIRHO, DRINK_CHIRHO, OJ_CHIRHO);

    // 13. The Japanese smokes Parliaments
    state_chirho.same_house_chirho(NATION_CHIRHO, JAPANESE_CHIRHO, SMOKE_CHIRHO, PARLIAMENT_CHIRHO);

    // 14. The Norwegian lives next to the blue house
    state_chirho.next_to_chirho(NATION_CHIRHO, NORWEGIAN_CHIRHO, COLOR_CHIRHO, BLUE_CHIRHO);

    // Additional: clue 15 often asks "Who drinks water? Who owns the zebra?"
    // These are what we're solving for!

    let t0_chirho = std::time::Instant::now();
    if state_chirho.solve_chirho() {
        let dt_chirho = t0_chirho.elapsed();
        state_chirho.print_solution_chirho();

        // Find who owns the zebra
        for h_chirho in 0..N_CHIRHO {
            if state_chirho.domains_chirho[h_chirho][PET_CHIRHO] == (1u8 << ZEBRA_CHIRHO) {
                let nation_chirho = state_chirho.domains_chirho[h_chirho][NATION_CHIRHO].trailing_zeros() as usize;
                println!("\nThe {} owns the zebra!", NATION_NAMES_CHIRHO[nation_chirho]);
            }
            if state_chirho.domains_chirho[h_chirho][DRINK_CHIRHO] == (1u8 << WATER_CHIRHO) {
                let nation_chirho = state_chirho.domains_chirho[h_chirho][NATION_CHIRHO].trailing_zeros() as usize;
                println!("The {} drinks water!", NATION_NAMES_CHIRHO[nation_chirho]);
            }
        }

        println!("\nSolved in {:?}", dt_chirho);
    } else {
        println!("No solution found!");
    }

    println!("\nSoli Deo Gloria ☧");
}
