#!/usr/bin/env python3
"""
Sudoku Solver using 1-Bit Domain Propagation ☧

Demonstrates practical use of miniKanren-style constraint propagation
with bit-parallel domain operations.

Each cell has a 9-bit domain where bit i means value (i+1) is possible.
Constraints (row, column, box) are applied via bitwise AND.

"Whether therefore ye eat, or drink, or whatsoever ye do, 
 do all to the glory of God." — 1 Corinthians 10:31
"""

import time
from typing import List, Optional, Tuple

# Domain constants
FULL_DOMAIN_CHIRHO = 0b111111111  # All values 1-9 possible
EMPTY_DOMAIN_CHIRHO = 0b000000000  # No values possible (failure)

def value_to_domain_chirho(v_chirho: int) -> int:
    """Convert value 1-9 to singleton domain."""
    if v_chirho < 1 or v_chirho > 9:
        raise ValueError(f"Value must be 1-9, got {v_chirho}")
    return 1 << (v_chirho - 1)

def domain_to_value_chirho(d_chirho: int) -> Optional[int]:
    """Convert singleton domain to value, or None if not singleton."""
    if d_chirho == 0 or (d_chirho & (d_chirho - 1)) != 0:
        return None  # Empty or multiple values
    return d_chirho.bit_length()

def popcount_chirho(d_chirho: int) -> int:
    """Count possible values in domain."""
    return bin(d_chirho).count('1')

def is_singleton_chirho(d_chirho: int) -> bool:
    """Check if exactly one value is possible."""
    return d_chirho != 0 and (d_chirho & (d_chirho - 1)) == 0

def lowest_value_chirho(d_chirho: int) -> Tuple[int, int]:
    """Fork: get (lowest single value, remaining values)."""
    lowest_chirho = d_chirho & (-d_chirho)  # Isolate lowest bit
    rest_chirho = d_chirho & (d_chirho - 1)  # Clear lowest bit
    return lowest_chirho, rest_chirho


class SudokuSolverChirho:
    """
    Sudoku solver using bit-parallel domain propagation.
    
    Each cell is a 9-bit domain. Propagation uses bitwise AND.
    Search uses branching on smallest domain (fail-first heuristic).
    """
    
    def __init__(self_chirho):
        # 81 cells, each with a 9-bit domain
        self_chirho.domains_chirho: List[int] = [FULL_DOMAIN_CHIRHO] * 81
        self_chirho.propagation_count_chirho = 0
        self_chirho.branch_count_chirho = 0
    
    def cell_index_chirho(self_chirho, row_chirho: int, col_chirho: int) -> int:
        """Convert (row, col) to flat index."""
        return row_chirho * 9 + col_chirho
    
    def get_peers_chirho(self_chirho, idx_chirho: int) -> List[int]:
        """Get indices of all cells that constrain this cell."""
        row_chirho = idx_chirho // 9
        col_chirho = idx_chirho % 9
        box_row_chirho = (row_chirho // 3) * 3
        box_col_chirho = (col_chirho // 3) * 3
        
        peers_chirho = set()
        
        # Same row
        for c_chirho in range(9):
            peers_chirho.add(row_chirho * 9 + c_chirho)
        
        # Same column
        for r_chirho in range(9):
            peers_chirho.add(r_chirho * 9 + col_chirho)
        
        # Same 3x3 box
        for r_chirho in range(box_row_chirho, box_row_chirho + 3):
            for c_chirho in range(box_col_chirho, box_col_chirho + 3):
                peers_chirho.add(r_chirho * 9 + c_chirho)
        
        peers_chirho.discard(idx_chirho)  # Remove self
        return list(peers_chirho)
    
    def set_value_chirho(self_chirho, idx_chirho: int, val_chirho: int) -> bool:
        """
        Set a cell to a specific value and propagate constraints.
        Returns False if this leads to failure.
        """
        domain_chirho = value_to_domain_chirho(val_chirho)
        
        # Intersect with current domain (unification!)
        self_chirho.domains_chirho[idx_chirho] &= domain_chirho
        
        if self_chirho.domains_chirho[idx_chirho] == EMPTY_DOMAIN_CHIRHO:
            return False
        
        return self_chirho.propagate_chirho()
    
    def propagate_chirho(self_chirho) -> bool:
        """
        Arc consistency propagation using bit operations.
        
        For each singleton domain, remove that value from all peers.
        Repeat until no changes (fixed point).
        """
        changed_chirho = True
        while changed_chirho:
            changed_chirho = False
            self_chirho.propagation_count_chirho += 1
            
            for idx_chirho in range(81):
                d_chirho = self_chirho.domains_chirho[idx_chirho]
                
                if d_chirho == EMPTY_DOMAIN_CHIRHO:
                    return False  # Failure
                
                if is_singleton_chirho(d_chirho):
                    # Remove this value from all peers
                    not_d_chirho = FULL_DOMAIN_CHIRHO ^ d_chirho
                    
                    for peer_idx_chirho in self_chirho.get_peers_chirho(idx_chirho):
                        old_chirho = self_chirho.domains_chirho[peer_idx_chirho]
                        # This is the key operation: domain intersection via AND
                        new_chirho = old_chirho & not_d_chirho
                        
                        if new_chirho != old_chirho:
                            changed_chirho = True
                            self_chirho.domains_chirho[peer_idx_chirho] = new_chirho
                            
                            if new_chirho == EMPTY_DOMAIN_CHIRHO:
                                return False  # Failure
        
        return True
    
    def is_solved_chirho(self_chirho) -> bool:
        """Check if all cells are singletons."""
        return all(is_singleton_chirho(d_chirho) for d_chirho in self_chirho.domains_chirho)
    
    def find_branch_cell_chirho(self_chirho) -> Optional[int]:
        """
        Find cell with smallest domain > 1 (fail-first heuristic).
        Returns None if all cells are singletons.
        """
        best_idx_chirho = None
        best_count_chirho = 10  # More than max possible
        
        for idx_chirho in range(81):
            count_chirho = popcount_chirho(self_chirho.domains_chirho[idx_chirho])
            if 1 < count_chirho < best_count_chirho:
                best_idx_chirho = idx_chirho
                best_count_chirho = count_chirho
        
        return best_idx_chirho
    
    def clone_chirho(self_chirho) -> 'SudokuSolverChirho':
        """Create a copy for branching."""
        new_chirho = SudokuSolverChirho()
        new_chirho.domains_chirho = self_chirho.domains_chirho.copy()
        new_chirho.propagation_count_chirho = self_chirho.propagation_count_chirho
        new_chirho.branch_count_chirho = self_chirho.branch_count_chirho
        return new_chirho
    
    def solve_chirho(self_chirho) -> bool:
        """
        Solve using constraint propagation + search.
        Returns True if solved, False if unsolvable.
        """
        if not self_chirho.propagate_chirho():
            return False
        
        if self_chirho.is_solved_chirho():
            return True
        
        # Branch on smallest domain
        branch_idx_chirho = self_chirho.find_branch_cell_chirho()
        if branch_idx_chirho is None:
            return self_chirho.is_solved_chirho()
        
        domain_chirho = self_chirho.domains_chirho[branch_idx_chirho]
        
        # Try each possible value
        while domain_chirho != 0:
            self_chirho.branch_count_chirho += 1
            
            # Fork: try lowest value first
            try_val_chirho, domain_chirho = lowest_value_chirho(domain_chirho)
            
            # Clone state and try this value
            attempt_chirho = self_chirho.clone_chirho()
            attempt_chirho.domains_chirho[branch_idx_chirho] = try_val_chirho
            
            if attempt_chirho.solve_chirho():
                # Success! Copy solution back
                self_chirho.domains_chirho = attempt_chirho.domains_chirho
                self_chirho.propagation_count_chirho = attempt_chirho.propagation_count_chirho
                self_chirho.branch_count_chirho = attempt_chirho.branch_count_chirho
                return True
        
        return False  # All branches failed
    
    def load_puzzle_chirho(self_chirho, puzzle_chirho: str) -> bool:
        """
        Load puzzle from string (81 chars, 0 or . for empty).
        Returns False if puzzle is invalid.
        """
        puzzle_chirho = puzzle_chirho.replace('\n', '').replace(' ', '')
        if len(puzzle_chirho) != 81:
            raise ValueError(f"Puzzle must be 81 chars, got {len(puzzle_chirho)}")
        
        for idx_chirho, ch_chirho in enumerate(puzzle_chirho):
            if ch_chirho in '123456789':
                if not self_chirho.set_value_chirho(idx_chirho, int(ch_chirho)):
                    return False
        
        return True
    
    def to_string_chirho(self_chirho) -> str:
        """Convert solution to string."""
        result_chirho = []
        for idx_chirho in range(81):
            val_chirho = domain_to_value_chirho(self_chirho.domains_chirho[idx_chirho])
            result_chirho.append(str(val_chirho) if val_chirho else '.')
        return ''.join(result_chirho)
    
    def print_board_chirho(self_chirho):
        """Pretty-print the board."""
        for row_chirho in range(9):
            if row_chirho % 3 == 0 and row_chirho > 0:
                print("------+-------+------")
            
            line_chirho = []
            for col_chirho in range(9):
                if col_chirho % 3 == 0 and col_chirho > 0:
                    line_chirho.append("|")
                
                idx_chirho = row_chirho * 9 + col_chirho
                val_chirho = domain_to_value_chirho(self_chirho.domains_chirho[idx_chirho])
                line_chirho.append(f" {val_chirho if val_chirho else '.'}")
            
            print(''.join(line_chirho))


def main_chirho():
    print("Sudoku Solver using 1-Bit Domain Propagation ☧\n")
    
    # Test puzzles (easy to hard)
    puzzles_chirho = [
        # Easy
        ("Easy",
         "530070000"
         "600195000"
         "098000060"
         "800060003"
         "400803001"
         "700020006"
         "060000280"
         "000419005"
         "000080079"),
        
        # Medium
        ("Medium",
         "000000680"
         "030000000"
         "900800100"
         "000002074"
         "060090030"
         "780500000"
         "001007005"
         "000000020"
         "024000000"),
        
        # Hard (17-clue minimal)
        ("Hard (17 clues)",
         "000000010"
         "400000000"
         "020000000"
         "000050407"
         "008000300"
         "001090000"
         "300400200"
         "050100000"
         "000806000"),
    ]
    
    for name_chirho, puzzle_chirho in puzzles_chirho:
        print(f"{'='*40}")
        print(f"Puzzle: {name_chirho}")
        print(f"{'='*40}")
        
        solver_chirho = SudokuSolverChirho()
        
        if not solver_chirho.load_puzzle_chirho(puzzle_chirho):
            print("Invalid puzzle!")
            continue
        
        print("\nInitial:")
        solver_chirho.print_board_chirho()
        
        start_chirho = time.perf_counter()
        solved_chirho = solver_chirho.solve_chirho()
        elapsed_chirho = (time.perf_counter() - start_chirho) * 1000
        
        if solved_chirho:
            print("\nSolved:")
            solver_chirho.print_board_chirho()
            print(f"\n✓ Solved in {elapsed_chirho:.2f}ms")
            print(f"  Propagation rounds: {solver_chirho.propagation_count_chirho}")
            print(f"  Search branches: {solver_chirho.branch_count_chirho}")
        else:
            print("\n✗ No solution exists")
        
        print()
    
    print("☧ Soli Deo Gloria ☧")


if __name__ == "__main__":
    main_chirho()
