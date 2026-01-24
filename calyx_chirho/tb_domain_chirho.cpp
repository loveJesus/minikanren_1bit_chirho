// Verilator Testbench for miniKanren Domain Operations
#include "Vmain.h"
#include "verilated.h"
#include <iostream>

int main(int argc, char** argv) {
    Verilated::commandArgs(argc, argv);
    Vmain* top = new Vmain;

    std::cout << "=== miniKanren Domain Operations Testbench ===" << std::endl;
    std::cout << "Testing: 0xFF AND 0xF0 = 0xF0 (values 4-7)" << std::endl;

    // Initialize
    top->clk = 0;
    top->reset = 1;
    top->go = 0;

    // Reset for 5 cycles
    for (int i = 0; i < 10; i++) {
        top->clk = !top->clk;
        top->eval();
    }

    // Release reset and start
    top->reset = 0;
    top->go = 1;

    std::cout << "Running simulation..." << std::endl;

    // Run for up to 100 cycles
    int cycle = 0;
    while (cycle < 100 && !top->done) {
        top->clk = !top->clk;
        top->eval();
        cycle++;
    }

    std::cout << "Simulation completed in " << (cycle/2) << " clock cycles" << std::endl;
    std::cout << "Done signal: " << (int)top->done << std::endl;

    delete top;
    return 0;
}
