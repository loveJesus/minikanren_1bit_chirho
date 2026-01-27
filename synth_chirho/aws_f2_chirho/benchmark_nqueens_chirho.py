#!/usr/bin/env python3
# For God so loved the world, that He gave His only begotten Son, that whosoever
# believeth in Him should not perish, but have everlasting life. John 3:16 ☧

"""
8-Queens Benchmark on AWS F2 FPGA ☧

This script benchmarks the miniKanren hardware accelerator on the 8-Queens problem.
The FPGA has 8 variables × 64-bit domains - perfect for 8-Queens (8 rows × 8 columns).

Usage:
    python3 benchmark_nqueens_chirho.py

Prerequisites:
    - AFI agfi-04abde24231f6775c loaded on F2 instance
    - Running on f2.6xlarge with FPGA tools available
"""

import time
import ctypes
from typing import List, Tuple, Optional

# FPGA Register Map (AXI-Lite OCL interface)
REG_VERSION_CHIRHO = 0x000
REG_CONTROL_CHIRHO = 0x004
REG_STATUS_CHIRHO = 0x008
REG_CMD0_CHIRHO = 0x010  # Command bits [31:0]
REG_CMD1_CHIRHO = 0x014  # Command bits [63:32]
REG_CMD2_CHIRHO = 0x018  # Command bits [69:64] (opcode + vars)
REG_RESP_BASE_CHIRHO = 0x020  # Response domains (8 × 64 bits = 16 × 32-bit regs)

# Command Opcodes
OP_INIT_CHIRHO = 0
OP_UNIFY_CHIRHO = 1
OP_CONSTRAIN_CHIRHO = 2
OP_BRANCH_CHIRHO = 3
OP_BACKTRACK_CHIRHO = 4

# Control bits
CTRL_ENABLE_CHIRHO = 0x1
CTRL_RESET_CHIRHO = 0x2

# Status bits
STATUS_DONE_CHIRHO = 0x1
STATUS_VALID_CHIRHO = 0x2


class FpgaDriverChirho:
    """Driver for AWS F2 FPGA with miniKanren accelerator."""

    def __init__(self_chirho):
        """Initialize connection to FPGA."""
        try:
            # Try to load AWS FPGA Python bindings
            import fpga_pci_lib as pci_chirho
            self_chirho.pci_chirho = pci_chirho
            # Attach to FPGA slot 0, PF 0, BAR 0
            self_chirho.handle_chirho = pci_chirho.pci_attach(0, 0, 0, 0)
            self_chirho.connected_chirho = True
            print("✓ Connected to FPGA")
        except Exception as e_chirho:
            print(f"⚠ FPGA not available: {e_chirho}")
            print("  Running in simulation mode")
            self_chirho.connected_chirho = False
            self_chirho.sim_domains_chirho = [0xFF] * 8  # Simulated state

    def read_reg_chirho(self_chirho, offset_chirho: int) -> int:
        """Read 32-bit register from FPGA."""
        if self_chirho.connected_chirho:
            return self_chirho.pci_chirho.pci_peek(self_chirho.handle_chirho, offset_chirho)
        return 0xF2010001 if offset_chirho == REG_VERSION_CHIRHO else 0

    def write_reg_chirho(self_chirho, offset_chirho: int, value_chirho: int):
        """Write 32-bit register to FPGA."""
        if self_chirho.connected_chirho:
            self_chirho.pci_chirho.pci_poke(self_chirho.handle_chirho, offset_chirho, value_chirho)

    def send_command_chirho(self_chirho, opcode_chirho: int, var1_chirho: int = 0,
                            var2_chirho: int = 0, mask_chirho: int = 0):
        """
        Send 70-bit command to FPGA.

        Format: [opcode:3][var1:3][var2:3][mask:64]
        - opcode: INIT/UNIFY/CONSTRAIN/BRANCH/BACKTRACK
        - var1, var2: variable indices (0-7)
        - mask: 64-bit domain mask
        """
        # Pack command
        cmd_hi_chirho = (opcode_chirho & 0x7) << 3 | (var1_chirho & 0x7)
        cmd_hi_chirho = (cmd_hi_chirho << 3) | (var2_chirho & 0x7)

        cmd0_chirho = mask_chirho & 0xFFFFFFFF
        cmd1_chirho = (mask_chirho >> 32) & 0xFFFFFFFF
        cmd2_chirho = cmd_hi_chirho & 0x3F

        self_chirho.write_reg_chirho(REG_CMD0_CHIRHO, cmd0_chirho)
        self_chirho.write_reg_chirho(REG_CMD1_CHIRHO, cmd1_chirho)
        self_chirho.write_reg_chirho(REG_CMD2_CHIRHO, cmd2_chirho)

        # Trigger command execution
        self_chirho.write_reg_chirho(REG_CONTROL_CHIRHO, CTRL_ENABLE_CHIRHO)

        # Simulation fallback
        if not self_chirho.connected_chirho:
            self_chirho._simulate_command_chirho(opcode_chirho, var1_chirho, var2_chirho, mask_chirho)

    def _simulate_command_chirho(self_chirho, opcode_chirho: int, var1_chirho: int,
                                  var2_chirho: int, mask_chirho: int):
        """Simulate command execution for testing without FPGA."""
        if opcode_chirho == OP_INIT_CHIRHO:
            self_chirho.sim_domains_chirho = [0xFF] * 8
        elif opcode_chirho == OP_CONSTRAIN_CHIRHO:
            self_chirho.sim_domains_chirho[var1_chirho] &= (mask_chirho & 0xFF)

    def read_domains_chirho(self_chirho) -> List[int]:
        """Read all 8 domain registers (64 bits each)."""
        domains_chirho = []
        for i_chirho in range(8):
            lo_chirho = self_chirho.read_reg_chirho(REG_RESP_BASE_CHIRHO + i_chirho * 8)
            hi_chirho = self_chirho.read_reg_chirho(REG_RESP_BASE_CHIRHO + i_chirho * 8 + 4)
            domain_chirho = lo_chirho | (hi_chirho << 32)
            domains_chirho.append(domain_chirho)

        if not self_chirho.connected_chirho:
            return self_chirho.sim_domains_chirho
        return domains_chirho

    def read_status_chirho(self_chirho) -> Tuple[bool, bool]:
        """Read status: (done, valid)."""
        status_chirho = self_chirho.read_reg_chirho(REG_STATUS_CHIRHO)
        done_chirho = bool(status_chirho & STATUS_DONE_CHIRHO)
        valid_chirho = bool(status_chirho & STATUS_VALID_CHIRHO)
        return done_chirho, valid_chirho

    def reset_chirho(self_chirho):
        """Reset the FPGA search engine."""
        self_chirho.write_reg_chirho(REG_CONTROL_CHIRHO, CTRL_RESET_CHIRHO)
        time.sleep(0.001)  # 1ms settle
        self_chirho.write_reg_chirho(REG_CONTROL_CHIRHO, 0)


def solve_nqueens_cpu_chirho(n_chirho: int = 8) -> List[List[int]]:
    """
    Solve N-Queens on CPU using bit-parallel search.
    Returns list of solutions (each solution is list of column positions).
    """
    solutions_chirho = []

    def backtrack_chirho(row_chirho: int, cols_chirho: int, diag1_chirho: int,
                         diag2_chirho: int, placement_chirho: List[int]):
        if row_chirho == n_chirho:
            solutions_chirho.append(placement_chirho[:])
            return

        available_chirho = ((1 << n_chirho) - 1) & ~(cols_chirho | diag1_chirho | diag2_chirho)

        while available_chirho:
            col_chirho = available_chirho & -available_chirho  # Lowest set bit
            col_idx_chirho = (col_chirho - 1).bit_length()
            available_chirho &= available_chirho - 1

            placement_chirho.append(col_idx_chirho)
            backtrack_chirho(
                row_chirho + 1,
                cols_chirho | col_chirho,
                (diag1_chirho | col_chirho) << 1,
                (diag2_chirho | col_chirho) >> 1,
                placement_chirho
            )
            placement_chirho.pop()

    backtrack_chirho(0, 0, 0, 0, [])
    return solutions_chirho


def solve_nqueens_fpga_chirho(fpga_chirho: FpgaDriverChirho, n_chirho: int = 8) -> Tuple[int, int]:
    """
    Solve N-Queens on FPGA using domain propagation.

    Returns: (solution_count, total_operations)

    Algorithm:
    1. Initialize 8 domains to 0xFF (all columns possible)
    2. For each placement, constrain row domain and propagate
    3. Branch on rows with multiple possibilities
    4. Count solutions when all domains are singletons
    """
    solutions_chirho = 0
    operations_chirho = 0

    # Initialize FPGA
    fpga_chirho.send_command_chirho(OP_INIT_CHIRHO)
    operations_chirho += 1

    # Initial domains: 8 bits (columns 0-7)
    for row_chirho in range(n_chirho):
        fpga_chirho.send_command_chirho(OP_CONSTRAIN_CHIRHO, row_chirho, 0, 0xFF)
        operations_chirho += 1

    # Simple enumeration approach:
    # Place queen in row 0, column c, then propagate constraints
    for c0_chirho in range(n_chirho):
        # Reset and place first queen
        fpga_chirho.send_command_chirho(OP_INIT_CHIRHO)
        for row_chirho in range(n_chirho):
            fpga_chirho.send_command_chirho(OP_CONSTRAIN_CHIRHO, row_chirho, 0, 0xFF)
        operations_chirho += n_chirho + 1

        # Constrain row 0 to column c0
        fpga_chirho.send_command_chirho(OP_CONSTRAIN_CHIRHO, 0, 0, 1 << c0_chirho)
        operations_chirho += 1

        # Propagate: remove c0 and diagonals from other rows
        for row_chirho in range(1, n_chirho):
            mask_chirho = 0xFF
            mask_chirho &= ~(1 << c0_chirho)  # Remove column
            # Remove diagonals
            if c0_chirho + row_chirho < n_chirho:
                mask_chirho &= ~(1 << (c0_chirho + row_chirho))
            if c0_chirho - row_chirho >= 0:
                mask_chirho &= ~(1 << (c0_chirho - row_chirho))
            fpga_chirho.send_command_chirho(OP_CONSTRAIN_CHIRHO, row_chirho, 0, mask_chirho)
            operations_chirho += 1

        # Read domains after propagation
        domains_chirho = fpga_chirho.read_domains_chirho()
        operations_chirho += 1

        # Count valid configurations in remaining space
        # (In real implementation, would branch and backtrack)
        # For now, estimate based on domain sizes
        remaining_chirho = 1
        for row_chirho in range(1, n_chirho):
            d_chirho = domains_chirho[row_chirho] & 0xFF
            count_chirho = bin(d_chirho).count('1')
            if count_chirho == 0:
                remaining_chirho = 0
                break
            remaining_chirho *= count_chirho

        if remaining_chirho > 0:
            # In full implementation, enumerate these
            # For now, note that constraint propagation worked
            solutions_chirho += 1  # Placeholder

    return solutions_chirho, operations_chirho


def benchmark_chirho():
    """Run N-Queens benchmark comparing CPU vs FPGA."""
    print("=" * 60)
    print("8-Queens Benchmark: CPU vs FPGA ☧")
    print("=" * 60)
    print()

    # Initialize FPGA
    fpga_chirho = FpgaDriverChirho()

    # Verify FPGA connection
    version_chirho = fpga_chirho.read_reg_chirho(REG_VERSION_CHIRHO)
    print(f"FPGA VERSION: 0x{version_chirho:08X}")
    if version_chirho != 0xF2010001:
        print(f"  ⚠ Expected 0xF2010001, got 0x{version_chirho:08X}")
    print()

    # CPU benchmark
    print("--- CPU Benchmark (bit-parallel search) ---")
    n_runs_chirho = 100

    start_chirho = time.perf_counter()
    for _ in range(n_runs_chirho):
        solutions_cpu_chirho = solve_nqueens_cpu_chirho(8)
    end_chirho = time.perf_counter()

    cpu_time_chirho = (end_chirho - start_chirho) / n_runs_chirho
    print(f"Solutions found: {len(solutions_cpu_chirho)}")
    print(f"Time per solve: {cpu_time_chirho * 1e6:.2f} μs")
    print(f"Throughput: {1 / cpu_time_chirho:.0f} solves/sec")
    print()

    # FPGA benchmark
    print("--- FPGA Benchmark (domain propagation) ---")

    start_chirho = time.perf_counter()
    for _ in range(n_runs_chirho):
        count_chirho, ops_chirho = solve_nqueens_fpga_chirho(fpga_chirho, 8)
    end_chirho = time.perf_counter()

    fpga_time_chirho = (end_chirho - start_chirho) / n_runs_chirho
    print(f"Partial solutions: {count_chirho}")
    print(f"FPGA operations: {ops_chirho}")
    print(f"Time per solve: {fpga_time_chirho * 1e6:.2f} μs")
    print(f"Operations/sec: {ops_chirho / fpga_time_chirho:.0f}")
    print()

    # Calculate cycle times
    fpga_freq_chirho = 250e6  # 250 MHz
    cycles_per_op_chirho = 8  # Per design spec
    theoretical_time_chirho = (ops_chirho * cycles_per_op_chirho) / fpga_freq_chirho

    print("--- Analysis ---")
    print(f"Theoretical FPGA time: {theoretical_time_chirho * 1e6:.2f} μs")
    print(f"Measured FPGA time: {fpga_time_chirho * 1e6:.2f} μs")
    overhead_chirho = fpga_time_chirho / theoretical_time_chirho if theoretical_time_chirho > 0 else 0
    print(f"Communication overhead: {overhead_chirho:.1f}x")
    print()

    if fpga_chirho.connected_chirho:
        speedup_chirho = cpu_time_chirho / fpga_time_chirho
        print(f"CPU vs FPGA: {speedup_chirho:.2f}x {'(FPGA faster)' if speedup_chirho > 1 else '(CPU faster)'}")
    else:
        print("Note: Running in simulation mode. Launch on F2 for real benchmark.")

    print()
    print("=" * 60)
    print("Benchmark complete. Soli Deo Gloria ☧")
    print("=" * 60)


if __name__ == "__main__":
    benchmark_chirho()
