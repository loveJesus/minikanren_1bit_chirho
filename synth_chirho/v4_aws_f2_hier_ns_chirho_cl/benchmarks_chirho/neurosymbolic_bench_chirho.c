// ============================================================================
// For God so loved the world - John 3:16 ☧
// Neurosymbolic Benchmark Suite for miniKanren FPGA
// Real-world and realistic test datasets
// ============================================================================

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include <math.h>

#include <fpga_pci.h>
#include <fpga_mgmt.h>

// Register addresses
#define REG_CONTROL_CHIRHO      0x04
#define REG_STATUS_CHIRHO       0x08
#define REG_TRAIN_MODE_CHIRHO   0x50
#define REG_TRAIN_CMD_LO_CHIRHO 0x54
#define REG_TRAIN_CMD_MID_CHIRHO 0x58
#define REG_TRAIN_CMD_HI_CHIRHO 0x5C
#define REG_TRAIN_CMD_TOP_CHIRHO 0x60
#define REG_TRAIN_RESP_LO_CHIRHO 0x64
#define REG_TRAIN_RESP_HI_CHIRHO 0x68
#define REG_INFER_MODE_CHIRHO   0x70
#define REG_WEIGHT_BASE_CHIRHO  0x100  // Weight memory base

// Control bits
#define CTRL_ENABLE_CHIRHO      (1 << 0)
#define STATUS_DONE_CHIRHO      (1 << 0)
#define TRAIN_ENABLE_CHIRHO     (1 << 0)

static pci_bar_handle_t bar0_chirho = PCI_BAR_HANDLE_INIT;
static pci_bar_handle_t bar4_chirho = PCI_BAR_HANDLE_INIT;

static inline int reg_read_chirho(uint32_t addr, uint32_t *val) {
    return fpga_pci_peek(bar0_chirho, addr, val);
}

static inline int reg_write_chirho(uint32_t addr, uint32_t val) {
    return fpga_pci_poke(bar0_chirho, addr, val);
}

static void wait_done_chirho(void) {
    uint32_t status;
    do { reg_read_chirho(REG_STATUS_CHIRHO, &status); }
    while (!(status & STATUS_DONE_CHIRHO));
}

static double get_time_ns_chirho(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1e9 + ts.tv_nsec;
}

// Q16.16 fixed-point helpers
static inline uint32_t float_to_q16_chirho(float f) {
    return (uint32_t)(f * 65536.0f);
}

static inline float q16_to_float_chirho(uint32_t q) {
    return (float)q / 65536.0f;
}

// ============================================================================
// DATASET 1: Family Relations (Knowledge Graph)
// Classic AI benchmark: learn family relationships
// ============================================================================

// Relation IDs
#define REL_PARENT_CHIRHO    0
#define REL_SIBLING_CHIRHO   1
#define REL_GRANDPARENT_CHIRHO 2
#define REL_UNCLE_CHIRHO     3
#define REL_COUSIN_CHIRHO    4

// Entity IDs (small family tree)
#define ENT_ALICE_CHIRHO     0
#define ENT_BOB_CHIRHO       1
#define ENT_CAROL_CHIRHO     2
#define ENT_DAVID_CHIRHO     3
#define ENT_EVE_CHIRHO       4
#define ENT_FRANK_CHIRHO     5
#define ENT_GRACE_CHIRHO     6
#define ENT_HENRY_CHIRHO     7
#define NUM_ENTITIES_CHIRHO  8

// Training data: (relation, entity1, entity2, label)
typedef struct {
    uint8_t rel;
    uint8_t e1;
    uint8_t e2;
    uint8_t label;  // 1 = true, 0 = false
} triple_chirho_t;

static triple_chirho_t family_data_chirho[] = {
    // Parent relations (ground truth)
    {REL_PARENT_CHIRHO, ENT_ALICE_CHIRHO, ENT_CAROL_CHIRHO, 1},
    {REL_PARENT_CHIRHO, ENT_ALICE_CHIRHO, ENT_DAVID_CHIRHO, 1},
    {REL_PARENT_CHIRHO, ENT_BOB_CHIRHO, ENT_CAROL_CHIRHO, 1},
    {REL_PARENT_CHIRHO, ENT_BOB_CHIRHO, ENT_DAVID_CHIRHO, 1},
    {REL_PARENT_CHIRHO, ENT_CAROL_CHIRHO, ENT_EVE_CHIRHO, 1},
    {REL_PARENT_CHIRHO, ENT_CAROL_CHIRHO, ENT_FRANK_CHIRHO, 1},
    {REL_PARENT_CHIRHO, ENT_DAVID_CHIRHO, ENT_GRACE_CHIRHO, 1},
    {REL_PARENT_CHIRHO, ENT_DAVID_CHIRHO, ENT_HENRY_CHIRHO, 1},
    // Negative examples
    {REL_PARENT_CHIRHO, ENT_EVE_CHIRHO, ENT_ALICE_CHIRHO, 0},
    {REL_PARENT_CHIRHO, ENT_FRANK_CHIRHO, ENT_BOB_CHIRHO, 0},
    // Sibling relations (derived)
    {REL_SIBLING_CHIRHO, ENT_CAROL_CHIRHO, ENT_DAVID_CHIRHO, 1},
    {REL_SIBLING_CHIRHO, ENT_EVE_CHIRHO, ENT_FRANK_CHIRHO, 1},
    {REL_SIBLING_CHIRHO, ENT_GRACE_CHIRHO, ENT_HENRY_CHIRHO, 1},
    {REL_SIBLING_CHIRHO, ENT_ALICE_CHIRHO, ENT_CAROL_CHIRHO, 0},
    // Grandparent relations (compositional)
    {REL_GRANDPARENT_CHIRHO, ENT_ALICE_CHIRHO, ENT_EVE_CHIRHO, 1},
    {REL_GRANDPARENT_CHIRHO, ENT_ALICE_CHIRHO, ENT_FRANK_CHIRHO, 1},
    {REL_GRANDPARENT_CHIRHO, ENT_BOB_CHIRHO, ENT_GRACE_CHIRHO, 1},
    {REL_GRANDPARENT_CHIRHO, ENT_CAROL_CHIRHO, ENT_ALICE_CHIRHO, 0},
};
#define FAMILY_DATA_SIZE_CHIRHO (sizeof(family_data_chirho) / sizeof(triple_chirho_t))

static void bench_family_relations_chirho(int epochs) {
    printf("\n=== Knowledge Graph: Family Relations ===\n");
    printf("Entities: %d, Triples: %lu, Epochs: %d\n",
           NUM_ENTITIES_CHIRHO, FAMILY_DATA_SIZE_CHIRHO, epochs);

    // Initialize weights for each relation
    reg_write_chirho(REG_TRAIN_MODE_CHIRHO, TRAIN_ENABLE_CHIRHO);

    double start = get_time_ns_chirho();
    float total_loss = 0;

    for (int epoch = 0; epoch < epochs; epoch++) {
        float epoch_loss = 0;

        for (size_t i = 0; i < FAMILY_DATA_SIZE_CHIRHO; i++) {
            triple_chirho_t *t = &family_data_chirho[i];

            // Encode triple as training command
            // CMD_LO: [relation:8][entity1:8][entity2:8][label:8]
            uint32_t cmd_lo = (t->rel << 24) | (t->e1 << 16) | (t->e2 << 8) | t->label;
            uint32_t cmd_mid = float_to_q16_chirho(0.01f);  // Learning rate

            reg_write_chirho(REG_TRAIN_CMD_LO_CHIRHO, cmd_lo);
            reg_write_chirho(REG_TRAIN_CMD_MID_CHIRHO, cmd_mid);
            reg_write_chirho(REG_TRAIN_CMD_HI_CHIRHO, 0x80000000);  // Train flag
            wait_done_chirho();

            // Read loss
            uint32_t loss_q16;
            reg_read_chirho(REG_TRAIN_RESP_LO_CHIRHO, &loss_q16);
            epoch_loss += q16_to_float_chirho(loss_q16);
        }

        total_loss = epoch_loss / FAMILY_DATA_SIZE_CHIRHO;
        if (epoch % 100 == 0 || epoch == epochs - 1) {
            printf("  Epoch %4d: avg_loss = %.4f\n", epoch, total_loss);
        }
    }

    double end = get_time_ns_chirho();
    double total_ms = (end - start) / 1e6;
    double per_triple_us = total_ms * 1000 / (epochs * FAMILY_DATA_SIZE_CHIRHO);

    printf("Training time: %.1f ms (%.2f us/triple)\n", total_ms, per_triple_us);
    printf("Final loss: %.4f\n", total_loss);

    // Test inference
    printf("\nInference test:\n");
    reg_write_chirho(REG_INFER_MODE_CHIRHO, 0x01);

    // Query: Is Alice grandparent of Grace?
    uint32_t query = (REL_GRANDPARENT_CHIRHO << 24) | (ENT_ALICE_CHIRHO << 16) | (ENT_GRACE_CHIRHO << 8);
    reg_write_chirho(REG_TRAIN_CMD_LO_CHIRHO, query);
    reg_write_chirho(REG_TRAIN_CMD_HI_CHIRHO, 0x00000000);  // Infer flag
    wait_done_chirho();

    uint32_t prob_q16;
    reg_read_chirho(REG_TRAIN_RESP_LO_CHIRHO, &prob_q16);
    printf("  grandparent(Alice, Grace) = %.3f (expect ~1.0)\n", q16_to_float_chirho(prob_q16));

    reg_write_chirho(REG_TRAIN_MODE_CHIRHO, 0);
}

// ============================================================================
// DATASET 2: Program Synthesis Patterns
// Learn to predict program structure from I/O examples
// ============================================================================

// Operation IDs
#define OP_ADD_CHIRHO   0
#define OP_SUB_CHIRHO   1
#define OP_MUL_CHIRHO   2
#define OP_DIV_CHIRHO   3
#define OP_NEG_CHIRHO   4
#define OP_ABS_CHIRHO   5
#define OP_MAX_CHIRHO   6
#define OP_MIN_CHIRHO   7
#define NUM_OPS_CHIRHO  8

// I/O example: (input1, input2, output, correct_op)
typedef struct {
    int16_t in1;
    int16_t in2;
    int16_t out;
    uint8_t op;
} io_example_chirho_t;

static io_example_chirho_t synth_data_chirho[] = {
    // Addition examples
    {3, 5, 8, OP_ADD_CHIRHO},
    {-2, 7, 5, OP_ADD_CHIRHO},
    {10, -3, 7, OP_ADD_CHIRHO},
    {0, 0, 0, OP_ADD_CHIRHO},
    // Subtraction examples
    {10, 3, 7, OP_SUB_CHIRHO},
    {5, 8, -3, OP_SUB_CHIRHO},
    {0, 5, -5, OP_SUB_CHIRHO},
    // Multiplication examples
    {3, 4, 12, OP_MUL_CHIRHO},
    {-2, 5, -10, OP_MUL_CHIRHO},
    {7, 0, 0, OP_MUL_CHIRHO},
    // Max examples
    {3, 7, 7, OP_MAX_CHIRHO},
    {-2, -5, -2, OP_MAX_CHIRHO},
    {4, 4, 4, OP_MAX_CHIRHO},
    // Min examples
    {3, 7, 3, OP_MIN_CHIRHO},
    {-2, -5, -5, OP_MIN_CHIRHO},
    {4, 4, 4, OP_MIN_CHIRHO},
};
#define SYNTH_DATA_SIZE_CHIRHO (sizeof(synth_data_chirho) / sizeof(io_example_chirho_t))

static void bench_program_synthesis_chirho(int epochs) {
    printf("\n=== Program Synthesis: Operation Learning ===\n");
    printf("Operations: %d, Examples: %lu, Epochs: %d\n",
           NUM_OPS_CHIRHO, SYNTH_DATA_SIZE_CHIRHO, epochs);

    reg_write_chirho(REG_TRAIN_MODE_CHIRHO, TRAIN_ENABLE_CHIRHO);

    double start = get_time_ns_chirho();
    int correct = 0;

    for (int epoch = 0; epoch < epochs; epoch++) {
        correct = 0;

        for (size_t i = 0; i < SYNTH_DATA_SIZE_CHIRHO; i++) {
            io_example_chirho_t *ex = &synth_data_chirho[i];

            // Encode I/O example
            // CMD_LO: [in1:16][in2:16]
            // CMD_MID: [out:16][target_op:8][0:8]
            uint32_t cmd_lo = ((uint16_t)ex->in1 << 16) | (uint16_t)ex->in2;
            uint32_t cmd_mid = ((uint16_t)ex->out << 16) | (ex->op << 8);

            reg_write_chirho(REG_TRAIN_CMD_LO_CHIRHO, cmd_lo);
            reg_write_chirho(REG_TRAIN_CMD_MID_CHIRHO, cmd_mid);
            reg_write_chirho(REG_TRAIN_CMD_HI_CHIRHO, 0x80000001);  // Synth train
            wait_done_chirho();

            // Read predicted op
            uint32_t resp;
            reg_read_chirho(REG_TRAIN_RESP_LO_CHIRHO, &resp);
            uint8_t predicted_op = resp & 0xFF;
            if (predicted_op == ex->op) correct++;
        }

        if (epoch % 100 == 0 || epoch == epochs - 1) {
            printf("  Epoch %4d: accuracy = %d/%lu (%.1f%%)\n",
                   epoch, correct, SYNTH_DATA_SIZE_CHIRHO,
                   100.0 * correct / SYNTH_DATA_SIZE_CHIRHO);
        }
    }

    double end = get_time_ns_chirho();
    printf("Training time: %.1f ms\n", (end - start) / 1e6);
    printf("Final accuracy: %.1f%%\n", 100.0 * correct / SYNTH_DATA_SIZE_CHIRHO);

    reg_write_chirho(REG_TRAIN_MODE_CHIRHO, 0);
}

// ============================================================================
// DATASET 3: Type Inference
// Learn type constraints from expressions
// ============================================================================

// Type IDs
#define TYPE_INT_CHIRHO     0
#define TYPE_FLOAT_CHIRHO   1
#define TYPE_BOOL_CHIRHO    2
#define TYPE_STRING_CHIRHO  3
#define TYPE_LIST_CHIRHO    4
#define TYPE_FUN_CHIRHO     5
#define NUM_TYPES_CHIRHO    6

// Expression pattern: (expr_type, arg1_type, arg2_type, result_type)
typedef struct {
    uint8_t expr;   // 0=literal, 1=binop, 2=if, 3=app, 4=lambda
    uint8_t arg1;
    uint8_t arg2;
    uint8_t result;
} type_example_chirho_t;

static type_example_chirho_t type_data_chirho[] = {
    // int + int -> int
    {1, TYPE_INT_CHIRHO, TYPE_INT_CHIRHO, TYPE_INT_CHIRHO},
    // float + float -> float
    {1, TYPE_FLOAT_CHIRHO, TYPE_FLOAT_CHIRHO, TYPE_FLOAT_CHIRHO},
    // int < int -> bool
    {1, TYPE_INT_CHIRHO, TYPE_INT_CHIRHO, TYPE_BOOL_CHIRHO},
    // if bool then int else int -> int
    {2, TYPE_BOOL_CHIRHO, TYPE_INT_CHIRHO, TYPE_INT_CHIRHO},
    // if bool then float else float -> float
    {2, TYPE_BOOL_CHIRHO, TYPE_FLOAT_CHIRHO, TYPE_FLOAT_CHIRHO},
    // (int -> int) applied to int -> int
    {3, TYPE_FUN_CHIRHO, TYPE_INT_CHIRHO, TYPE_INT_CHIRHO},
    // string concat
    {1, TYPE_STRING_CHIRHO, TYPE_STRING_CHIRHO, TYPE_STRING_CHIRHO},
    // list append
    {1, TYPE_LIST_CHIRHO, TYPE_LIST_CHIRHO, TYPE_LIST_CHIRHO},
    // length of list -> int
    {3, TYPE_LIST_CHIRHO, TYPE_LIST_CHIRHO, TYPE_INT_CHIRHO},
};
#define TYPE_DATA_SIZE_CHIRHO (sizeof(type_data_chirho) / sizeof(type_example_chirho_t))

static void bench_type_inference_chirho(int epochs) {
    printf("\n=== Type Inference: Constraint Learning ===\n");
    printf("Types: %d, Examples: %lu, Epochs: %d\n",
           NUM_TYPES_CHIRHO, TYPE_DATA_SIZE_CHIRHO, epochs);

    reg_write_chirho(REG_TRAIN_MODE_CHIRHO, TRAIN_ENABLE_CHIRHO);

    double start = get_time_ns_chirho();

    for (int epoch = 0; epoch < epochs; epoch++) {
        float epoch_loss = 0;

        for (size_t i = 0; i < TYPE_DATA_SIZE_CHIRHO; i++) {
            type_example_chirho_t *ex = &type_data_chirho[i];

            uint32_t cmd_lo = (ex->expr << 24) | (ex->arg1 << 16) | (ex->arg2 << 8) | ex->result;

            reg_write_chirho(REG_TRAIN_CMD_LO_CHIRHO, cmd_lo);
            reg_write_chirho(REG_TRAIN_CMD_HI_CHIRHO, 0x80000002);  // Type train
            wait_done_chirho();

            uint32_t loss_q16;
            reg_read_chirho(REG_TRAIN_RESP_LO_CHIRHO, &loss_q16);
            epoch_loss += q16_to_float_chirho(loss_q16);
        }

        if (epoch % 100 == 0 || epoch == epochs - 1) {
            printf("  Epoch %4d: loss = %.4f\n", epoch, epoch_loss / TYPE_DATA_SIZE_CHIRHO);
        }
    }

    double end = get_time_ns_chirho();
    printf("Training time: %.1f ms\n", (end - start) / 1e6);

    reg_write_chirho(REG_TRAIN_MODE_CHIRHO, 0);
}

// ============================================================================
// DATASET 4: Probabilistic Logic (Noisy OR)
// Learn probabilistic rules from uncertain data
// ============================================================================

typedef struct {
    uint8_t causes[4];    // Up to 4 cause variables
    uint8_t num_causes;
    uint8_t effect;
    float probability;    // Observed probability
} prob_rule_chirho_t;

static prob_rule_chirho_t prob_data_chirho[] = {
    // Disease diagnosis (simplified)
    // fever AND cough -> flu (0.8)
    {{0, 1, 0, 0}, 2, 10, 0.8f},
    // fever AND rash -> measles (0.7)
    {{0, 2, 0, 0}, 2, 11, 0.7f},
    // cough AND fatigue -> cold (0.6)
    {{1, 3, 0, 0}, 2, 12, 0.6f},
    // fever alone -> infection (0.5)
    {{0, 0, 0, 0}, 1, 13, 0.5f},
    // fever AND cough AND fatigue -> serious (0.9)
    {{0, 1, 3, 0}, 3, 14, 0.9f},
    // no symptoms -> healthy (0.95)
    {{0, 0, 0, 0}, 0, 15, 0.95f},
};
#define PROB_DATA_SIZE_CHIRHO (sizeof(prob_data_chirho) / sizeof(prob_rule_chirho_t))

static void bench_probabilistic_logic_chirho(int epochs) {
    printf("\n=== Probabilistic Logic: Noisy-OR Learning ===\n");
    printf("Rules: %lu, Epochs: %d\n", PROB_DATA_SIZE_CHIRHO, epochs);

    reg_write_chirho(REG_TRAIN_MODE_CHIRHO, TRAIN_ENABLE_CHIRHO);
    reg_write_chirho(REG_INFER_MODE_CHIRHO, 0x01);  // Probabilistic mode

    double start = get_time_ns_chirho();

    for (int epoch = 0; epoch < epochs; epoch++) {
        float total_error = 0;

        for (size_t i = 0; i < PROB_DATA_SIZE_CHIRHO; i++) {
            prob_rule_chirho_t *r = &prob_data_chirho[i];

            // Encode rule
            uint32_t cmd_lo = (r->causes[0] << 24) | (r->causes[1] << 16) |
                              (r->causes[2] << 8) | r->causes[3];
            uint32_t cmd_mid = (r->num_causes << 24) | (r->effect << 16) |
                               float_to_q16_chirho(r->probability);

            reg_write_chirho(REG_TRAIN_CMD_LO_CHIRHO, cmd_lo);
            reg_write_chirho(REG_TRAIN_CMD_MID_CHIRHO, cmd_mid);
            reg_write_chirho(REG_TRAIN_CMD_HI_CHIRHO, 0x80000003);  // Prob train
            wait_done_chirho();

            // Read predicted vs actual
            uint32_t pred_q16;
            reg_read_chirho(REG_TRAIN_RESP_LO_CHIRHO, &pred_q16);
            float pred = q16_to_float_chirho(pred_q16);
            total_error += fabsf(pred - r->probability);
        }

        if (epoch % 100 == 0 || epoch == epochs - 1) {
            printf("  Epoch %4d: MAE = %.4f\n", epoch, total_error / PROB_DATA_SIZE_CHIRHO);
        }
    }

    double end = get_time_ns_chirho();
    printf("Training time: %.1f ms\n", (end - start) / 1e6);

    // Test inference
    printf("\nProbabilistic inference:\n");
    // Query: P(flu | fever, cough)
    uint32_t query_lo = (0 << 24) | (1 << 16);  // fever=0, cough=1
    uint32_t query_mid = (2 << 24) | (10 << 16);  // 2 causes, effect=flu(10)
    reg_write_chirho(REG_TRAIN_CMD_LO_CHIRHO, query_lo);
    reg_write_chirho(REG_TRAIN_CMD_MID_CHIRHO, query_mid);
    reg_write_chirho(REG_TRAIN_CMD_HI_CHIRHO, 0x00000003);  // Prob infer
    wait_done_chirho();

    uint32_t prob_q16;
    reg_read_chirho(REG_TRAIN_RESP_LO_CHIRHO, &prob_q16);
    printf("  P(flu | fever, cough) = %.3f (expect ~0.8)\n", q16_to_float_chirho(prob_q16));

    reg_write_chirho(REG_TRAIN_MODE_CHIRHO, 0);
    reg_write_chirho(REG_INFER_MODE_CHIRHO, 0);
}

// ============================================================================
// DATASET 5: Relational Path Queries (larger scale)
// Simulate knowledge graph traversal
// ============================================================================

#define KG_NUM_ENTITIES_CHIRHO 1000
#define KG_NUM_RELATIONS_CHIRHO 50
#define KG_NUM_TRIPLES_CHIRHO 5000

static void bench_kg_path_queries_chirho(int num_queries) {
    printf("\n=== Knowledge Graph: Path Queries (scaled) ===\n");
    printf("Entities: %d, Relations: %d, Triples: %d, Queries: %d\n",
           KG_NUM_ENTITIES_CHIRHO, KG_NUM_RELATIONS_CHIRHO,
           KG_NUM_TRIPLES_CHIRHO, num_queries);

    // Generate synthetic KG triples
    srand(42);  // Reproducible

    double start = get_time_ns_chirho();

    // Load triples into FPGA memory
    printf("Loading knowledge graph...\n");
    for (int i = 0; i < KG_NUM_TRIPLES_CHIRHO; i++) {
        uint32_t head = rand() % KG_NUM_ENTITIES_CHIRHO;
        uint32_t rel = rand() % KG_NUM_RELATIONS_CHIRHO;
        uint32_t tail = rand() % KG_NUM_ENTITIES_CHIRHO;

        uint32_t cmd = (head << 20) | (rel << 10) | tail;
        reg_write_chirho(REG_TRAIN_CMD_LO_CHIRHO, cmd);
        reg_write_chirho(REG_TRAIN_CMD_HI_CHIRHO, 0x40000000);  // Load triple
        wait_done_chirho();
    }

    double load_time = get_time_ns_chirho() - start;
    printf("Load time: %.1f ms (%.2f us/triple)\n",
           load_time / 1e6, load_time / 1e3 / KG_NUM_TRIPLES_CHIRHO);

    // Run path queries (2-hop reachability)
    printf("Running path queries...\n");
    start = get_time_ns_chirho();
    int found = 0;

    for (int i = 0; i < num_queries; i++) {
        uint32_t src = rand() % KG_NUM_ENTITIES_CHIRHO;
        uint32_t rel1 = rand() % KG_NUM_RELATIONS_CHIRHO;
        uint32_t rel2 = rand() % KG_NUM_RELATIONS_CHIRHO;
        uint32_t dst = rand() % KG_NUM_ENTITIES_CHIRHO;

        // Query: exists X. (src, rel1, X) AND (X, rel2, dst)
        uint32_t cmd_lo = (src << 20) | (rel1 << 10) | dst;
        uint32_t cmd_mid = rel2;

        reg_write_chirho(REG_TRAIN_CMD_LO_CHIRHO, cmd_lo);
        reg_write_chirho(REG_TRAIN_CMD_MID_CHIRHO, cmd_mid);
        reg_write_chirho(REG_TRAIN_CMD_HI_CHIRHO, 0x50000000);  // Path query
        wait_done_chirho();

        uint32_t result;
        reg_read_chirho(REG_TRAIN_RESP_LO_CHIRHO, &result);
        if (result > 0) found++;
    }

    double query_time = get_time_ns_chirho() - start;
    printf("Query time: %.1f ms (%.2f us/query)\n",
           query_time / 1e6, query_time / 1e3 / num_queries);
    printf("Paths found: %d/%d (%.1f%%)\n", found, num_queries, 100.0 * found / num_queries);
}

// ============================================================================
// Main
// ============================================================================

int main(int argc, char **argv) {
    (void)argc; (void)argv;
    int rc;

    printf("============================================================\n");
    printf("Neurosymbolic Benchmark Suite ☧\n");
    printf("Real-world datasets for miniKanren FPGA\n");
    printf("For God so loved the world - John 3:16\n");
    printf("============================================================\n");

    rc = fpga_mgmt_init();
    if (rc) { printf("ERROR: fpga_mgmt_init: %d\n", rc); return 1; }

    rc = fpga_pci_attach(0, FPGA_APP_PF, APP_PF_BAR0, 0, &bar0_chirho);
    if (rc) { printf("ERROR: fpga_pci_attach: %d\n", rc); return 1; }

    rc = fpga_pci_attach(0, FPGA_APP_PF, APP_PF_BAR4, 0, &bar4_chirho);
    if (rc) { printf("WARN: BAR4 attach failed (HBM): %d\n", rc); }

    // Reset
    reg_write_chirho(REG_CONTROL_CHIRHO, 0x02);
    usleep(10000);
    reg_write_chirho(REG_CONTROL_CHIRHO, 0x01);

    // Run benchmarks
    bench_family_relations_chirho(500);
    bench_program_synthesis_chirho(500);
    bench_type_inference_chirho(500);
    bench_probabilistic_logic_chirho(500);
    bench_kg_path_queries_chirho(1000);

    printf("\n============================================================\n");
    printf("Neurosymbolic benchmarks complete. Soli Deo Gloria ☧\n");
    printf("============================================================\n");

    fpga_pci_detach(bar0_chirho);
    fpga_pci_detach(bar4_chirho);
    fpga_mgmt_close();

    return 0;
}
