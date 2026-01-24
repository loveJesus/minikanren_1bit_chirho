#!/usr/bin/env python3
"""
Calyx Syntax Verification ☧

Verifies that our Calyx (.futil) files have correct structure
without needing the full Calyx toolchain installed.
"""

import re
import sys
from pathlib import Path

def verify_calyx_file_chirho(filepath_chirho: Path) -> tuple[bool, list[str]]:
    """Verify a Calyx file has valid structure."""
    errors_chirho = []
    content_chirho = filepath_chirho.read_text()
    lines_chirho = content_chirho.split('\n')

    # Track structure
    brace_depth_chirho = 0
    in_component_chirho = False
    in_cells_chirho = False
    in_wires_chirho = False
    in_control_chirho = False
    component_name_chirho = None

    for i_chirho, line_chirho in enumerate(lines_chirho, 1):
        stripped_chirho = line_chirho.strip()

        # Skip comments and empty lines
        if stripped_chirho.startswith('//') or not stripped_chirho:
            continue

        # Count braces
        brace_depth_chirho += stripped_chirho.count('{') - stripped_chirho.count('}')

        # Check for component definition
        if stripped_chirho.startswith('component '):
            match_chirho = re.match(r'component\s+(\w+)', stripped_chirho)
            if match_chirho:
                component_name_chirho = match_chirho.group(1)
                in_component_chirho = True
                # Verify _chirho suffix (except 'main' which is required by Calyx)
                if not component_name_chirho.endswith('_chirho') and component_name_chirho != 'main':
                    errors_chirho.append(f"Line {i_chirho}: Component '{component_name_chirho}' missing _chirho suffix")

        # Check for section markers
        if stripped_chirho.startswith('cells {'):
            in_cells_chirho = True
        elif stripped_chirho.startswith('wires {'):
            in_wires_chirho = True
        elif stripped_chirho.startswith('control {'):
            in_control_chirho = True

        # Check cell declarations for _chirho suffix
        # Format: @external? name = primitive_type(...)
        if in_cells_chirho and '=' in stripped_chirho and not stripped_chirho.startswith('//'):
            # Match cell declarations: optional @external, then name = ...
            match_chirho = re.match(r'(?:@\w+\s+)?(\w+)\s*=\s*(\w+)\s*\(', stripped_chirho)
            if match_chirho:
                cell_name_chirho = match_chirho.group(1)
                primitive_chirho = match_chirho.group(2)
                # Check if it's a user-defined cell (not a standard primitive instantiation)
                std_primitives_chirho = ['std_reg', 'std_add', 'std_sub', 'std_and',
                                         'std_or', 'std_xor', 'std_eq', 'std_lt', 'std_le',
                                         'std_gt', 'std_ge', 'std_neq', 'std_not',
                                         'std_const', 'std_slice', 'std_pad', 'std_cat',
                                         'seq_mem_d1', 'seq_mem_d2', 'comb_mem_d1']
                # Cell name should have _chirho suffix
                if not cell_name_chirho.endswith('_chirho'):
                    errors_chirho.append(f"Line {i_chirho}: Cell '{cell_name_chirho}' missing _chirho suffix")

        # Check group names for _chirho suffix
        if 'group ' in stripped_chirho:
            match_chirho = re.match(r'group\s+(\w+)', stripped_chirho)
            if match_chirho:
                group_name_chirho = match_chirho.group(1)
                if not group_name_chirho.endswith('_chirho'):
                    errors_chirho.append(f"Line {i_chirho}: Group '{group_name_chirho}' missing _chirho suffix")

    # Check final brace balance
    if brace_depth_chirho != 0:
        errors_chirho.append(f"Unbalanced braces: depth={brace_depth_chirho}")

    return len(errors_chirho) == 0, errors_chirho

def main_chirho():
    """Verify all Calyx files in the project."""
    project_root_chirho = Path(__file__).parent.parent
    calyx_dir_chirho = project_root_chirho / 'calyx_chirho'

    if not calyx_dir_chirho.exists():
        print("No calyx_chirho directory found")
        return 1

    all_passed_chirho = True
    files_checked_chirho = 0

    print("Calyx Syntax Verification ☧")
    print("=" * 50)

    for futil_file_chirho in sorted(calyx_dir_chirho.glob('*.futil')):
        files_checked_chirho += 1
        passed_chirho, errors_chirho = verify_calyx_file_chirho(futil_file_chirho)

        if passed_chirho:
            print(f"✓ {futil_file_chirho.name}")
        else:
            print(f"✗ {futil_file_chirho.name}")
            for err_chirho in errors_chirho:
                print(f"    {err_chirho}")
            all_passed_chirho = False

    print("=" * 50)
    if all_passed_chirho:
        print(f"All {files_checked_chirho} Calyx files passed verification ☧")
        return 0
    else:
        print("Some files have issues")
        return 1

if __name__ == '__main__':
    sys.exit(main_chirho())
