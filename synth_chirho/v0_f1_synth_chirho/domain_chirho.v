/* verilator lint_off MULTITOP */
/// =================== Unsigned, Fixed Point =========================
module std_fp_add #(
    parameter WIDTH = 32,
    parameter INT_WIDTH = 16,
    parameter FRAC_WIDTH = 16
) (
    input  logic [WIDTH-1:0] left,
    input  logic [WIDTH-1:0] right,
    output logic [WIDTH-1:0] out
);
  assign out = left + right;
endmodule

module std_fp_sub #(
    parameter WIDTH = 32,
    parameter INT_WIDTH = 16,
    parameter FRAC_WIDTH = 16
) (
    input  logic [WIDTH-1:0] left,
    input  logic [WIDTH-1:0] right,
    output logic [WIDTH-1:0] out
);
  assign out = left - right;
endmodule

module std_fp_mult_pipe #(
    parameter WIDTH = 32,
    parameter INT_WIDTH = 16,
    parameter FRAC_WIDTH = 16,
    parameter SIGNED = 0
) (
    input  logic [WIDTH-1:0] left,
    input  logic [WIDTH-1:0] right,
    input  logic             go,
    input  logic             clk,
    input  logic             reset,
    output logic [WIDTH-1:0] out,
    output logic             done
);
  logic [WIDTH-1:0]          rtmp;
  logic [WIDTH-1:0]          ltmp;
  logic [(WIDTH << 1) - 1:0] out_tmp;
  // Buffer used to walk through the 3 cycles of the pipeline.
  logic done_buf[1:0];

  assign done = done_buf[1];

  assign out = out_tmp[(WIDTH << 1) - INT_WIDTH - 1 : WIDTH - INT_WIDTH];

  // If the done buffer is completely empty and go is high then execution
  // just started.
  logic start;
  assign start = go;

  // Start sending the done signal.
  always_ff @(posedge clk) begin
    if (start)
      done_buf[0] <= 1;
    else
      done_buf[0] <= 0;
  end

  // Push the done signal through the pipeline.
  always_ff @(posedge clk) begin
    if (go) begin
      done_buf[1] <= done_buf[0];
    end else begin
      done_buf[1] <= 0;
    end
  end

  // Register the inputs
  always_ff @(posedge clk) begin
    if (reset) begin
      rtmp <= 0;
      ltmp <= 0;
    end else if (go) begin
      if (SIGNED) begin
        rtmp <= $signed(right);
        ltmp <= $signed(left);
      end else begin
        rtmp <= right;
        ltmp <= left;
      end
    end else begin
      rtmp <= 0;
      ltmp <= 0;
    end

  end

  // Compute the output and save it into out_tmp
  always_ff @(posedge clk) begin
    if (reset) begin
      out_tmp <= 0;
    end else if (go) begin
      if (SIGNED) begin
        // In the first cycle, this performs an invalid computation because
        // ltmp and rtmp only get their actual values in cycle 1
        out_tmp <= $signed(
          { {WIDTH{ltmp[WIDTH-1]}}, ltmp} *
          { {WIDTH{rtmp[WIDTH-1]}}, rtmp}
        );
      end else begin
        out_tmp <= ltmp * rtmp;
      end
    end else begin
      out_tmp <= out_tmp;
    end
  end
endmodule

/* verilator lint_off WIDTH */
module std_fp_div_pipe #(
  parameter WIDTH = 32,
  parameter INT_WIDTH = 16,
  parameter FRAC_WIDTH = 16
) (
    input  logic             go,
    input  logic             clk,
    input  logic             reset,
    input  logic [WIDTH-1:0] left,
    input  logic [WIDTH-1:0] right,
    output logic [WIDTH-1:0] out_remainder,
    output logic [WIDTH-1:0] out_quotient,
    output logic             done
);
    localparam ITERATIONS = WIDTH + FRAC_WIDTH;

    logic [WIDTH-1:0] quotient, quotient_next;
    logic [WIDTH:0] acc, acc_next;
    logic [$clog2(ITERATIONS)-1:0] idx;
    logic start, running, finished, dividend_is_zero;

    assign start = go && !running;
    assign dividend_is_zero = start && left == 0;
    assign finished = idx == ITERATIONS - 1 && running;

    always_ff @(posedge clk) begin
      if (reset || finished || dividend_is_zero)
        running <= 0;
      else if (start)
        running <= 1;
      else
        running <= running;
    end

    always_comb begin
      if (acc >= {1'b0, right}) begin
        acc_next = acc - right;
        {acc_next, quotient_next} = {acc_next[WIDTH-1:0], quotient, 1'b1};
      end else begin
        {acc_next, quotient_next} = {acc, quotient} << 1;
      end
    end

    // `done` signaling
    always_ff @(posedge clk) begin
      if (dividend_is_zero || finished)
        done <= 1;
      else
        done <= 0;
    end

    always_ff @(posedge clk) begin
      if (running)
        idx <= idx + 1;
      else
        idx <= 0;
    end

    always_ff @(posedge clk) begin
      if (reset) begin
        out_quotient <= 0;
        out_remainder <= 0;
      end else if (start) begin
        out_quotient <= 0;
        out_remainder <= left;
      end else if (go == 0) begin
        out_quotient <= out_quotient;
        out_remainder <= out_remainder;
      end else if (dividend_is_zero) begin
        out_quotient <= 0;
        out_remainder <= 0;
      end else if (finished) begin
        out_quotient <= quotient_next;
        out_remainder <= out_remainder;
      end else begin
        out_quotient <= out_quotient;
        if (right <= out_remainder)
          out_remainder <= out_remainder - right;
        else
          out_remainder <= out_remainder;
      end
    end

    always_ff @(posedge clk) begin
      if (reset) begin
        acc <= 0;
        quotient <= 0;
      end else if (start) begin
        {acc, quotient} <= {{WIDTH{1'b0}}, left, 1'b0};
      end else begin
        acc <= acc_next;
        quotient <= quotient_next;
      end
    end
endmodule

module std_fp_gt #(
    parameter WIDTH = 32,
    parameter INT_WIDTH = 16,
    parameter FRAC_WIDTH = 16
) (
    input  logic [WIDTH-1:0] left,
    input  logic [WIDTH-1:0] right,
    output logic             out
);
  assign out = left > right;
endmodule

/// =================== Signed, Fixed Point =========================
module std_fp_sadd #(
    parameter WIDTH = 32,
    parameter INT_WIDTH = 16,
    parameter FRAC_WIDTH = 16
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed [WIDTH-1:0] out
);
  assign out = $signed(left + right);
endmodule

module std_fp_ssub #(
    parameter WIDTH = 32,
    parameter INT_WIDTH = 16,
    parameter FRAC_WIDTH = 16
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed [WIDTH-1:0] out
);

  assign out = $signed(left - right);
endmodule

module std_fp_smult_pipe #(
    parameter WIDTH = 32,
    parameter INT_WIDTH = 16,
    parameter FRAC_WIDTH = 16
) (
    input  [WIDTH-1:0]              left,
    input  [WIDTH-1:0]              right,
    input  logic                    reset,
    input  logic                    go,
    input  logic                    clk,
    output logic [WIDTH-1:0]        out,
    output logic                    done
);
  std_fp_mult_pipe #(
    .WIDTH(WIDTH),
    .INT_WIDTH(INT_WIDTH),
    .FRAC_WIDTH(FRAC_WIDTH),
    .SIGNED(1)
  ) comp (
    .clk(clk),
    .done(done),
    .reset(reset),
    .go(go),
    .left(left),
    .right(right),
    .out(out)
  );
endmodule

module std_fp_sdiv_pipe #(
    parameter WIDTH = 32,
    parameter INT_WIDTH = 16,
    parameter FRAC_WIDTH = 16
) (
    input                     clk,
    input                     go,
    input                     reset,
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed [WIDTH-1:0] out_quotient,
    output signed [WIDTH-1:0] out_remainder,
    output logic              done
);

  logic signed [WIDTH-1:0] left_abs, right_abs, comp_out_q, comp_out_r, right_save, out_rem_intermediate;

  // Registers to figure out how to transform outputs.
  logic different_signs, left_sign, right_sign;

  // Latch the value of control registers so that their available after
  // go signal becomes low.
  always_ff @(posedge clk) begin
    if (go) begin
      right_save <= right_abs;
      left_sign <= left[WIDTH-1];
      right_sign <= right[WIDTH-1];
    end else begin
      left_sign <= left_sign;
      right_save <= right_save;
      right_sign <= right_sign;
    end
  end

  assign right_abs = right[WIDTH-1] ? -right : right;
  assign left_abs = left[WIDTH-1] ? -left : left;

  assign different_signs = left_sign ^ right_sign;
  assign out_quotient = different_signs ? -comp_out_q : comp_out_q;

  // Remainder is computed as:
  //  t0 = |left| % |right|
  //  t1 = if left * right < 0 and t0 != 0 then |right| - t0 else t0
  //  rem = if right < 0 then -t1 else t1
  assign out_rem_intermediate = different_signs & |comp_out_r ? $signed(right_save - comp_out_r) : comp_out_r;
  assign out_remainder = right_sign ? -out_rem_intermediate : out_rem_intermediate;

  std_fp_div_pipe #(
    .WIDTH(WIDTH),
    .INT_WIDTH(INT_WIDTH),
    .FRAC_WIDTH(FRAC_WIDTH)
  ) comp (
    .reset(reset),
    .clk(clk),
    .done(done),
    .go(go),
    .left(left_abs),
    .right(right_abs),
    .out_quotient(comp_out_q),
    .out_remainder(comp_out_r)
  );
endmodule

module std_fp_sgt #(
    parameter WIDTH = 32,
    parameter INT_WIDTH = 16,
    parameter FRAC_WIDTH = 16
) (
    input  logic signed [WIDTH-1:0] left,
    input  logic signed [WIDTH-1:0] right,
    output logic signed             out
);
  assign out = $signed(left > right);
endmodule

module std_fp_slt #(
    parameter WIDTH = 32,
    parameter INT_WIDTH = 16,
    parameter FRAC_WIDTH = 16
) (
   input logic signed [WIDTH-1:0] left,
   input logic signed [WIDTH-1:0] right,
   output logic signed            out
);
  assign out = $signed(left < right);
endmodule

/// =================== Unsigned, Bitnum =========================
module std_mult_pipe #(
    parameter WIDTH = 32
) (
    input  logic [WIDTH-1:0] left,
    input  logic [WIDTH-1:0] right,
    input  logic             reset,
    input  logic             go,
    input  logic             clk,
    output logic [WIDTH-1:0] out,
    output logic             done
);
  std_fp_mult_pipe #(
    .WIDTH(WIDTH),
    .INT_WIDTH(WIDTH),
    .FRAC_WIDTH(0),
    .SIGNED(0)
  ) comp (
    .reset(reset),
    .clk(clk),
    .done(done),
    .go(go),
    .left(left),
    .right(right),
    .out(out)
  );
endmodule

module std_div_pipe #(
    parameter WIDTH = 32
) (
    input                    reset,
    input                    clk,
    input                    go,
    input        [WIDTH-1:0] left,
    input        [WIDTH-1:0] right,
    output logic [WIDTH-1:0] out_remainder,
    output logic [WIDTH-1:0] out_quotient,
    output logic             done
);

  logic [WIDTH-1:0] dividend;
  logic [(WIDTH-1)*2:0] divisor;
  logic [WIDTH-1:0] quotient;
  logic [WIDTH-1:0] quotient_msk;
  logic start, running, finished, dividend_is_zero;

  assign start = go && !running;
  assign finished = quotient_msk == 0 && running;
  assign dividend_is_zero = start && left == 0;

  always_ff @(posedge clk) begin
    // Early return if the divisor is zero.
    if (finished || dividend_is_zero)
      done <= 1;
    else
      done <= 0;
  end

  always_ff @(posedge clk) begin
    if (reset || finished || dividend_is_zero)
      running <= 0;
    else if (start)
      running <= 1;
    else
      running <= running;
  end

  // Outputs
  always_ff @(posedge clk) begin
    if (dividend_is_zero || start) begin
      out_quotient <= 0;
      out_remainder <= 0;
    end else if (finished) begin
      out_quotient <= quotient;
      out_remainder <= dividend;
    end else begin
      // Otherwise, explicitly latch the values.
      out_quotient <= out_quotient;
      out_remainder <= out_remainder;
    end
  end

  // Calculate the quotient mask.
  always_ff @(posedge clk) begin
    if (start)
      quotient_msk <= 1 << WIDTH - 1;
    else if (running)
      quotient_msk <= quotient_msk >> 1;
    else
      quotient_msk <= quotient_msk;
  end

  // Calculate the quotient.
  always_ff @(posedge clk) begin
    if (start)
      quotient <= 0;
    else if (divisor <= dividend)
      quotient <= quotient | quotient_msk;
    else
      quotient <= quotient;
  end

  // Calculate the dividend.
  always_ff @(posedge clk) begin
    if (start)
      dividend <= left;
    else if (divisor <= dividend)
      dividend <= dividend - divisor;
    else
      dividend <= dividend;
  end

  always_ff @(posedge clk) begin
    if (start) begin
      divisor <= right << WIDTH - 1;
    end else if (finished) begin
      divisor <= 0;
    end else begin
      divisor <= divisor >> 1;
    end
  end

  // Simulation self test against unsynthesizable implementation.
  `ifdef VERILATOR
    logic [WIDTH-1:0] l, r;
    always_ff @(posedge clk) begin
      if (go) begin
        l <= left;
        r <= right;
      end else begin
        l <= l;
        r <= r;
      end
    end

    always @(posedge clk) begin
      if (done && $unsigned(out_remainder) != $unsigned(l % r))
        $error(
          "\nstd_div_pipe (Remainder): Computed and golden outputs do not match!\n",
          "left: %0d", $unsigned(l),
          "  right: %0d\n", $unsigned(r),
          "expected: %0d", $unsigned(l % r),
          "  computed: %0d", $unsigned(out_remainder)
        );

      if (done && $unsigned(out_quotient) != $unsigned(l / r))
        $error(
          "\nstd_div_pipe (Quotient): Computed and golden outputs do not match!\n",
          "left: %0d", $unsigned(l),
          "  right: %0d\n", $unsigned(r),
          "expected: %0d", $unsigned(l / r),
          "  computed: %0d", $unsigned(out_quotient)
        );
    end
  `endif
endmodule

/// =================== Signed, Bitnum =========================
module std_sadd #(
    parameter WIDTH = 32
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed [WIDTH-1:0] out
);
  assign out = $signed(left + right);
endmodule

module std_ssub #(
    parameter WIDTH = 32
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed [WIDTH-1:0] out
);
  assign out = $signed(left - right);
endmodule

module std_smult_pipe #(
    parameter WIDTH = 32
) (
    input  logic                    reset,
    input  logic                    go,
    input  logic                    clk,
    input  signed       [WIDTH-1:0] left,
    input  signed       [WIDTH-1:0] right,
    output logic signed [WIDTH-1:0] out,
    output logic                    done
);
  std_fp_mult_pipe #(
    .WIDTH(WIDTH),
    .INT_WIDTH(WIDTH),
    .FRAC_WIDTH(0),
    .SIGNED(1)
  ) comp (
    .reset(reset),
    .clk(clk),
    .done(done),
    .go(go),
    .left(left),
    .right(right),
    .out(out)
  );
endmodule

/* verilator lint_off WIDTH */
module std_sdiv_pipe #(
    parameter WIDTH = 32
) (
    input                           reset,
    input                           clk,
    input                           go,
    input  logic signed [WIDTH-1:0] left,
    input  logic signed [WIDTH-1:0] right,
    output logic signed [WIDTH-1:0] out_quotient,
    output logic signed [WIDTH-1:0] out_remainder,
    output logic                    done
);

  logic signed [WIDTH-1:0] left_abs, right_abs, comp_out_q, comp_out_r, right_save, out_rem_intermediate;

  // Registers to figure out how to transform outputs.
  logic different_signs, left_sign, right_sign;

  // Latch the value of control registers so that their available after
  // go signal becomes low.
  always_ff @(posedge clk) begin
    if (go) begin
      right_save <= right_abs;
      left_sign <= left[WIDTH-1];
      right_sign <= right[WIDTH-1];
    end else begin
      left_sign <= left_sign;
      right_save <= right_save;
      right_sign <= right_sign;
    end
  end

  assign right_abs = right[WIDTH-1] ? -right : right;
  assign left_abs = left[WIDTH-1] ? -left : left;

  assign different_signs = left_sign ^ right_sign;
  assign out_quotient = different_signs ? -comp_out_q : comp_out_q;

  // Remainder is computed as:
  //  t0 = |left| % |right|
  //  t1 = if left * right < 0 and t0 != 0 then |right| - t0 else t0
  //  rem = if right < 0 then -t1 else t1
  assign out_rem_intermediate = different_signs & |comp_out_r ? $signed(right_save - comp_out_r) : comp_out_r;
  assign out_remainder = right_sign ? -out_rem_intermediate : out_rem_intermediate;

  std_div_pipe #(
    .WIDTH(WIDTH)
  ) comp (
    .reset(reset),
    .clk(clk),
    .done(done),
    .go(go),
    .left(left_abs),
    .right(right_abs),
    .out_quotient(comp_out_q),
    .out_remainder(comp_out_r)
  );

  // Simulation self test against unsynthesizable implementation.
  `ifdef VERILATOR
    logic signed [WIDTH-1:0] l, r;
    always_ff @(posedge clk) begin
      if (go) begin
        l <= left;
        r <= right;
      end else begin
        l <= l;
        r <= r;
      end
    end

    always @(posedge clk) begin
      if (done && out_quotient != $signed(l / r))
        $error(
          "\nstd_sdiv_pipe (Quotient): Computed and golden outputs do not match!\n",
          "left: %0d", l,
          "  right: %0d\n", r,
          "expected: %0d", $signed(l / r),
          "  computed: %0d", $signed(out_quotient),
        );
      if (done && out_remainder != $signed(((l % r) + r) % r))
        $error(
          "\nstd_sdiv_pipe (Remainder): Computed and golden outputs do not match!\n",
          "left: %0d", l,
          "  right: %0d\n", r,
          "expected: %0d", $signed(((l % r) + r) % r),
          "  computed: %0d", $signed(out_remainder),
        );
    end
  `endif
endmodule

module std_sgt #(
    parameter WIDTH = 32
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed             out
);
  assign out = $signed(left > right);
endmodule

module std_slt #(
    parameter WIDTH = 32
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed             out
);
  assign out = $signed(left < right);
endmodule

module std_seq #(
    parameter WIDTH = 32
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed             out
);
  assign out = $signed(left == right);
endmodule

module std_sneq #(
    parameter WIDTH = 32
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed             out
);
  assign out = $signed(left != right);
endmodule

module std_sge #(
    parameter WIDTH = 32
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed             out
);
  assign out = $signed(left >= right);
endmodule

module std_sle #(
    parameter WIDTH = 32
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed             out
);
  assign out = $signed(left <= right);
endmodule

module std_slsh #(
    parameter WIDTH = 32
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed [WIDTH-1:0] out
);
  assign out = left <<< right;
endmodule

module std_srsh #(
    parameter WIDTH = 32
) (
    input  signed [WIDTH-1:0] left,
    input  signed [WIDTH-1:0] right,
    output signed [WIDTH-1:0] out
);
  assign out = left >>> right;
endmodule

// Signed extension
module std_signext #(
  parameter IN_WIDTH  = 32,
  parameter OUT_WIDTH = 32
) (
  input wire logic [IN_WIDTH-1:0]  in,
  output logic     [OUT_WIDTH-1:0] out
);
  localparam EXTEND = OUT_WIDTH - IN_WIDTH;
  assign out = { {EXTEND {in[IN_WIDTH-1]}}, in};

  `ifdef VERILATOR
    always_comb begin
      if (IN_WIDTH > OUT_WIDTH)
        $error(
          "std_signext: Output width less than input width\n",
          "IN_WIDTH: %0d", IN_WIDTH,
          "OUT_WIDTH: %0d", OUT_WIDTH
        );
    end
  `endif
endmodule
/**
 * Core primitives for Calyx.
 * Implements core primitives used by the compiler.
 *
 * Conventions:
 * - All parameter names must be SNAKE_CASE and all caps.
 * - Port names must be snake_case, no caps.
 */
`default_nettype none

module std_slice #(
    parameter IN_WIDTH  = 32,
    parameter OUT_WIDTH = 32
) (
   input wire                   logic [ IN_WIDTH-1:0] in,
   output logic [OUT_WIDTH-1:0] out
);
  assign out = in[OUT_WIDTH-1:0];

  `ifdef VERILATOR
    always_comb begin
      if (IN_WIDTH < OUT_WIDTH)
        $error(
          "std_slice: Input width less than output width\n",
          "IN_WIDTH: %0d", IN_WIDTH,
          "OUT_WIDTH: %0d", OUT_WIDTH
        );
    end
  `endif
endmodule

module std_pad #(
    parameter IN_WIDTH  = 32,
    parameter OUT_WIDTH = 32
) (
   input wire logic [IN_WIDTH-1:0]  in,
   output logic     [OUT_WIDTH-1:0] out
);
  localparam EXTEND = OUT_WIDTH - IN_WIDTH;
  assign out = { {EXTEND {1'b0}}, in};

  `ifdef VERILATOR
    always_comb begin
      if (IN_WIDTH > OUT_WIDTH)
        $error(
          "std_pad: Output width less than input width\n",
          "IN_WIDTH: %0d", IN_WIDTH,
          "OUT_WIDTH: %0d", OUT_WIDTH
        );
    end
  `endif
endmodule

module std_cat #(
  parameter LEFT_WIDTH  = 32,
  parameter RIGHT_WIDTH = 32,
  parameter OUT_WIDTH = 64
) (
  input wire logic [LEFT_WIDTH-1:0] left,
  input wire logic [RIGHT_WIDTH-1:0] right,
  output logic [OUT_WIDTH-1:0] out
);
  assign out = {left, right};

  `ifdef VERILATOR
    always_comb begin
      if (LEFT_WIDTH + RIGHT_WIDTH != OUT_WIDTH)
        $error(
          "std_cat: Output width must equal sum of input widths\n",
          "LEFT_WIDTH: %0d", LEFT_WIDTH,
          "RIGHT_WIDTH: %0d", RIGHT_WIDTH,
          "OUT_WIDTH: %0d", OUT_WIDTH
        );
    end
  `endif
endmodule

module std_not #(
    parameter WIDTH = 32
) (
   input wire               logic [WIDTH-1:0] in,
   output logic [WIDTH-1:0] out
);
  assign out = ~in;
endmodule

module std_and #(
    parameter WIDTH = 32
) (
   input wire               logic [WIDTH-1:0] left,
   input wire               logic [WIDTH-1:0] right,
   output logic [WIDTH-1:0] out
);
  assign out = left & right;
endmodule

module std_or #(
    parameter WIDTH = 32
) (
   input wire               logic [WIDTH-1:0] left,
   input wire               logic [WIDTH-1:0] right,
   output logic [WIDTH-1:0] out
);
  assign out = left | right;
endmodule

module std_xor #(
    parameter WIDTH = 32
) (
   input wire               logic [WIDTH-1:0] left,
   input wire               logic [WIDTH-1:0] right,
   output logic [WIDTH-1:0] out
);
  assign out = left ^ right;
endmodule

module std_sub #(
    parameter WIDTH = 32
) (
   input wire               logic [WIDTH-1:0] left,
   input wire               logic [WIDTH-1:0] right,
   output logic [WIDTH-1:0] out
);
  assign out = left - right;
endmodule

module std_gt #(
    parameter WIDTH = 32
) (
   input wire   logic [WIDTH-1:0] left,
   input wire   logic [WIDTH-1:0] right,
   output logic out
);
  assign out = left > right;
endmodule

module std_lt #(
    parameter WIDTH = 32
) (
   input wire   logic [WIDTH-1:0] left,
   input wire   logic [WIDTH-1:0] right,
   output logic out
);
  assign out = left < right;
endmodule

module std_eq #(
    parameter WIDTH = 32
) (
   input wire   logic [WIDTH-1:0] left,
   input wire   logic [WIDTH-1:0] right,
   output logic out
);
  assign out = left == right;
endmodule

module std_neq #(
    parameter WIDTH = 32
) (
   input wire   logic [WIDTH-1:0] left,
   input wire   logic [WIDTH-1:0] right,
   output logic out
);
  assign out = left != right;
endmodule

module std_ge #(
    parameter WIDTH = 32
) (
    input wire   logic [WIDTH-1:0] left,
    input wire   logic [WIDTH-1:0] right,
    output logic out
);
  assign out = left >= right;
endmodule

module std_le #(
    parameter WIDTH = 32
) (
   input wire   logic [WIDTH-1:0] left,
   input wire   logic [WIDTH-1:0] right,
   output logic out
);
  assign out = left <= right;
endmodule

module std_lsh #(
    parameter WIDTH = 32
) (
   input wire               logic [WIDTH-1:0] left,
   input wire               logic [WIDTH-1:0] right,
   output logic [WIDTH-1:0] out
);
  assign out = left << right;
endmodule

module std_rsh #(
    parameter WIDTH = 32
) (
   input wire               logic [WIDTH-1:0] left,
   input wire               logic [WIDTH-1:0] right,
   output logic [WIDTH-1:0] out
);
  assign out = left >> right;
endmodule

/// this primitive is intended to be used
/// for lowering purposes (not in source programs)
module std_mux #(
    parameter WIDTH = 32
) (
   input wire               logic cond,
   input wire               logic [WIDTH-1:0] tru,
   input wire               logic [WIDTH-1:0] fal,
   output logic [WIDTH-1:0] out
);
  assign out = cond ? tru : fal;
endmodule

module std_mem_d1 #(
    parameter WIDTH = 32,
    parameter SIZE = 16,
    parameter IDX_SIZE = 4
) (
   input wire                logic [IDX_SIZE-1:0] addr0,
   input wire                logic [ WIDTH-1:0] write_data,
   input wire                logic write_en,
   input wire                logic clk,
   input wire                logic reset,
   output logic [ WIDTH-1:0] read_data,
   output logic              done
);

  logic [WIDTH-1:0] mem[SIZE-1:0];

  /* verilator lint_off WIDTH */
  assign read_data = mem[addr0];

  always_ff @(posedge clk) begin
    if (reset)
      done <= '0;
    else if (write_en)
      done <= '1;
    else
      done <= '0;
  end

  always_ff @(posedge clk) begin
    if (!reset && write_en)
      mem[addr0] <= write_data;
  end

  // Check for out of bounds access
  `ifdef VERILATOR
    always_comb begin
      if (addr0 >= SIZE)
        $error(
          "std_mem_d1: Out of bounds access\n",
          "addr0: %0d\n", addr0,
          "SIZE: %0d", SIZE
        );
    end
  `endif
endmodule

module std_mem_d2 #(
    parameter WIDTH = 32,
    parameter D0_SIZE = 16,
    parameter D1_SIZE = 16,
    parameter D0_IDX_SIZE = 4,
    parameter D1_IDX_SIZE = 4
) (
   input wire                logic [D0_IDX_SIZE-1:0] addr0,
   input wire                logic [D1_IDX_SIZE-1:0] addr1,
   input wire                logic [ WIDTH-1:0] write_data,
   input wire                logic write_en,
   input wire                logic clk,
   input wire                logic reset,
   output logic [ WIDTH-1:0] read_data,
   output logic              done
);

  /* verilator lint_off WIDTH */
  logic [WIDTH-1:0] mem[D0_SIZE-1:0][D1_SIZE-1:0];

  assign read_data = mem[addr0][addr1];

  always_ff @(posedge clk) begin
    if (reset)
      done <= '0;
    else if (write_en)
      done <= '1;
    else
      done <= '0;
  end

  always_ff @(posedge clk) begin
    if (!reset && write_en)
      mem[addr0][addr1] <= write_data;
  end

  // Check for out of bounds access
  `ifdef VERILATOR
    always_comb begin
      if (addr0 >= D0_SIZE)
        $error(
          "std_mem_d2: Out of bounds access\n",
          "addr0: %0d\n", addr0,
          "D0_SIZE: %0d", D0_SIZE
        );
      if (addr1 >= D1_SIZE)
        $error(
          "std_mem_d2: Out of bounds access\n",
          "addr1: %0d\n", addr1,
          "D1_SIZE: %0d", D1_SIZE
        );
    end
  `endif
endmodule

module std_mem_d3 #(
    parameter WIDTH = 32,
    parameter D0_SIZE = 16,
    parameter D1_SIZE = 16,
    parameter D2_SIZE = 16,
    parameter D0_IDX_SIZE = 4,
    parameter D1_IDX_SIZE = 4,
    parameter D2_IDX_SIZE = 4
) (
   input wire                logic [D0_IDX_SIZE-1:0] addr0,
   input wire                logic [D1_IDX_SIZE-1:0] addr1,
   input wire                logic [D2_IDX_SIZE-1:0] addr2,
   input wire                logic [ WIDTH-1:0] write_data,
   input wire                logic write_en,
   input wire                logic clk,
   input wire                logic reset,
   output logic [ WIDTH-1:0] read_data,
   output logic              done
);

  /* verilator lint_off WIDTH */
  logic [WIDTH-1:0] mem[D0_SIZE-1:0][D1_SIZE-1:0][D2_SIZE-1:0];

  assign read_data = mem[addr0][addr1][addr2];

  always_ff @(posedge clk) begin
    if (reset)
      done <= '0;
    else if (write_en)
      done <= '1;
    else
      done <= '0;
  end

  always_ff @(posedge clk) begin
    if (!reset && write_en)
      mem[addr0][addr1][addr2] <= write_data;
  end

  // Check for out of bounds access
  `ifdef VERILATOR
    always_comb begin
      if (addr0 >= D0_SIZE)
        $error(
          "std_mem_d3: Out of bounds access\n",
          "addr0: %0d\n", addr0,
          "D0_SIZE: %0d", D0_SIZE
        );
      if (addr1 >= D1_SIZE)
        $error(
          "std_mem_d3: Out of bounds access\n",
          "addr1: %0d\n", addr1,
          "D1_SIZE: %0d", D1_SIZE
        );
      if (addr2 >= D2_SIZE)
        $error(
          "std_mem_d3: Out of bounds access\n",
          "addr2: %0d\n", addr2,
          "D2_SIZE: %0d", D2_SIZE
        );
    end
  `endif
endmodule

module std_mem_d4 #(
    parameter WIDTH = 32,
    parameter D0_SIZE = 16,
    parameter D1_SIZE = 16,
    parameter D2_SIZE = 16,
    parameter D3_SIZE = 16,
    parameter D0_IDX_SIZE = 4,
    parameter D1_IDX_SIZE = 4,
    parameter D2_IDX_SIZE = 4,
    parameter D3_IDX_SIZE = 4
) (
   input wire                logic [D0_IDX_SIZE-1:0] addr0,
   input wire                logic [D1_IDX_SIZE-1:0] addr1,
   input wire                logic [D2_IDX_SIZE-1:0] addr2,
   input wire                logic [D3_IDX_SIZE-1:0] addr3,
   input wire                logic [ WIDTH-1:0] write_data,
   input wire                logic write_en,
   input wire                logic clk,
   input wire                logic reset,
   output logic [ WIDTH-1:0] read_data,
   output logic              done
);

  /* verilator lint_off WIDTH */
  logic [WIDTH-1:0] mem[D0_SIZE-1:0][D1_SIZE-1:0][D2_SIZE-1:0][D3_SIZE-1:0];

  assign read_data = mem[addr0][addr1][addr2][addr3];

  always_ff @(posedge clk) begin
    if (reset)
      done <= '0;
    else if (write_en)
      done <= '1;
    else
      done <= '0;
  end

  always_ff @(posedge clk) begin
    if (!reset && write_en)
      mem[addr0][addr1][addr2][addr3] <= write_data;
  end

  // Check for out of bounds access
  `ifdef VERILATOR
    always_comb begin
      if (addr0 >= D0_SIZE)
        $error(
          "std_mem_d4: Out of bounds access\n",
          "addr0: %0d\n", addr0,
          "D0_SIZE: %0d", D0_SIZE
        );
      if (addr1 >= D1_SIZE)
        $error(
          "std_mem_d4: Out of bounds access\n",
          "addr1: %0d\n", addr1,
          "D1_SIZE: %0d", D1_SIZE
        );
      if (addr2 >= D2_SIZE)
        $error(
          "std_mem_d4: Out of bounds access\n",
          "addr2: %0d\n", addr2,
          "D2_SIZE: %0d", D2_SIZE
        );
      if (addr3 >= D3_SIZE)
        $error(
          "std_mem_d4: Out of bounds access\n",
          "addr3: %0d\n", addr3,
          "D3_SIZE: %0d", D3_SIZE
        );
    end
  `endif
endmodule

`default_nettype wire

module undef #(
    parameter WIDTH = 32
) (
   output logic [WIDTH-1:0] out
);
assign out = 'x;
endmodule

module std_const #(
    parameter WIDTH = 32,
    parameter VALUE = 32
) (
   output logic [WIDTH-1:0] out
);
assign out = VALUE;
endmodule

module std_wire #(
    parameter WIDTH = 32
) (
   input logic [WIDTH-1:0] in,
   output logic [WIDTH-1:0] out
);
assign out = in;
endmodule

module std_add #(
    parameter WIDTH = 32
) (
   input logic [WIDTH-1:0] left,
   input logic [WIDTH-1:0] right,
   output logic [WIDTH-1:0] out
);
assign out = left + right;
endmodule

module std_reg #(
    parameter WIDTH = 32
) (
   input logic [WIDTH-1:0] in,
   input logic write_en,
   input logic clk,
   input logic reset,
   output logic [WIDTH-1:0] out,
   output logic done
);
always_ff @(posedge clk) begin
    if (reset) begin
       out <= 0;
       done <= 0;
    end else if (write_en) begin
      out <= in;
      done <= 1'd1;
    end else done <= 1'd0;
  end
endmodule

module unify_unit_chirho(
  input logic go,
  input logic [63:0] d1_chirho,
  input logic [63:0] d2_chirho,
  output logic done,
  output logic [63:0] result_chirho,
  output logic failed_chirho,
  input logic clk,
  input logic reset
);
// COMPONENT START: unify_unit_chirho
logic [63:0] result_reg_chirho_in;
logic result_reg_chirho_write_en;
logic result_reg_chirho_clk;
logic result_reg_chirho_reset;
logic [63:0] result_reg_chirho_out;
logic result_reg_chirho_done;
logic [63:0] and_gate_chirho_left;
logic [63:0] and_gate_chirho_right;
logic [63:0] and_gate_chirho_out;
logic [63:0] zero_chirho_out;
logic [63:0] eq_chirho_left;
logic [63:0] eq_chirho_right;
logic eq_chirho_out;
logic failed_reg_chirho_in;
logic failed_reg_chirho_write_en;
logic failed_reg_chirho_clk;
logic failed_reg_chirho_reset;
logic failed_reg_chirho_out;
logic failed_reg_chirho_done;
logic [1:0] fsm_in;
logic fsm_write_en;
logic fsm_clk;
logic fsm_reset;
logic [1:0] fsm_out;
logic fsm_done;
logic invoke0_go_in;
logic invoke0_go_out;
logic invoke0_done_in;
logic invoke0_done_out;
logic invoke1_go_in;
logic invoke1_go_out;
logic invoke1_done_in;
logic invoke1_done_out;
logic tdcc_go_in;
logic tdcc_go_out;
logic tdcc_done_in;
logic tdcc_done_out;
std_reg # (
    .WIDTH(64)
) result_reg_chirho (
    .clk(result_reg_chirho_clk),
    .done(result_reg_chirho_done),
    .in(result_reg_chirho_in),
    .out(result_reg_chirho_out),
    .reset(result_reg_chirho_reset),
    .write_en(result_reg_chirho_write_en)
);
std_and # (
    .WIDTH(64)
) and_gate_chirho (
    .left(and_gate_chirho_left),
    .out(and_gate_chirho_out),
    .right(and_gate_chirho_right)
);
std_const # (
    .VALUE(64'd0),
    .WIDTH(64)
) zero_chirho (
    .out(zero_chirho_out)
);
std_eq # (
    .WIDTH(64)
) eq_chirho (
    .left(eq_chirho_left),
    .out(eq_chirho_out),
    .right(eq_chirho_right)
);
std_reg # (
    .WIDTH(1)
) failed_reg_chirho (
    .clk(failed_reg_chirho_clk),
    .done(failed_reg_chirho_done),
    .in(failed_reg_chirho_in),
    .out(failed_reg_chirho_out),
    .reset(failed_reg_chirho_reset),
    .write_en(failed_reg_chirho_write_en)
);
std_reg # (
    .WIDTH(2)
) fsm (
    .clk(fsm_clk),
    .done(fsm_done),
    .in(fsm_in),
    .out(fsm_out),
    .reset(fsm_reset),
    .write_en(fsm_write_en)
);
std_wire # (
    .WIDTH(1)
) invoke0_go (
    .in(invoke0_go_in),
    .out(invoke0_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke0_done (
    .in(invoke0_done_in),
    .out(invoke0_done_out)
);
std_wire # (
    .WIDTH(1)
) invoke1_go (
    .in(invoke1_go_in),
    .out(invoke1_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke1_done (
    .in(invoke1_done_in),
    .out(invoke1_done_out)
);
std_wire # (
    .WIDTH(1)
) tdcc_go (
    .in(tdcc_go_in),
    .out(tdcc_go_out)
);
std_wire # (
    .WIDTH(1)
) tdcc_done (
    .in(tdcc_done_in),
    .out(tdcc_done_out)
);
wire _guard0 = 1;
wire _guard1 = invoke0_go_out;
wire _guard2 = invoke0_go_out;
wire _guard3 = invoke1_go_out;
wire _guard4 = invoke1_go_out;
wire _guard5 = tdcc_done_out;
wire _guard6 = fsm_out == 2'd2;
wire _guard7 = fsm_out == 2'd0;
wire _guard8 = invoke0_done_out;
wire _guard9 = _guard7 & _guard8;
wire _guard10 = tdcc_go_out;
wire _guard11 = _guard9 & _guard10;
wire _guard12 = _guard6 | _guard11;
wire _guard13 = fsm_out == 2'd1;
wire _guard14 = invoke1_done_out;
wire _guard15 = _guard13 & _guard14;
wire _guard16 = tdcc_go_out;
wire _guard17 = _guard15 & _guard16;
wire _guard18 = _guard12 | _guard17;
wire _guard19 = fsm_out == 2'd0;
wire _guard20 = invoke0_done_out;
wire _guard21 = _guard19 & _guard20;
wire _guard22 = tdcc_go_out;
wire _guard23 = _guard21 & _guard22;
wire _guard24 = fsm_out == 2'd2;
wire _guard25 = fsm_out == 2'd1;
wire _guard26 = invoke1_done_out;
wire _guard27 = _guard25 & _guard26;
wire _guard28 = tdcc_go_out;
wire _guard29 = _guard27 & _guard28;
wire _guard30 = invoke0_go_out;
wire _guard31 = invoke0_go_out;
wire _guard32 = invoke1_go_out;
wire _guard33 = invoke1_go_out;
wire _guard34 = invoke0_done_out;
wire _guard35 = ~_guard34;
wire _guard36 = fsm_out == 2'd0;
wire _guard37 = _guard35 & _guard36;
wire _guard38 = tdcc_go_out;
wire _guard39 = _guard37 & _guard38;
wire _guard40 = invoke1_done_out;
wire _guard41 = ~_guard40;
wire _guard42 = fsm_out == 2'd1;
wire _guard43 = _guard41 & _guard42;
wire _guard44 = tdcc_go_out;
wire _guard45 = _guard43 & _guard44;
wire _guard46 = fsm_out == 2'd2;
assign and_gate_chirho_left = d1_chirho;
assign and_gate_chirho_right = d2_chirho;
assign eq_chirho_left = result_reg_chirho_out;
assign eq_chirho_right = zero_chirho_out;
assign done = _guard5;
assign failed_chirho = failed_reg_chirho_out;
assign result_chirho = result_reg_chirho_out;
assign fsm_write_en = _guard18;
assign fsm_clk = clk;
assign fsm_reset = reset;
assign fsm_in =
  _guard23 ? 2'd1 :
  _guard24 ? 2'd0 :
  _guard29 ? 2'd2 :
  2'd0;
always_comb begin
  if(~$onehot0({_guard29, _guard24, _guard23})) begin
    $fatal(2, "Multiple assignment to port `fsm.in'.");
end
end
assign result_reg_chirho_write_en = _guard30;
assign result_reg_chirho_clk = clk;
assign result_reg_chirho_reset = reset;
assign result_reg_chirho_in = and_gate_chirho_out;
assign failed_reg_chirho_write_en = _guard32;
assign failed_reg_chirho_clk = clk;
assign failed_reg_chirho_reset = reset;
assign failed_reg_chirho_in = eq_chirho_out;
assign invoke0_go_in = _guard39;
assign tdcc_go_in = go;
assign invoke0_done_in = result_reg_chirho_done;
assign invoke1_go_in = _guard45;
assign tdcc_done_in = _guard46;
assign invoke1_done_in = failed_reg_chirho_done;
// COMPONENT END: unify_unit_chirho
endmodule
module disj_unit_chirho(
  input logic go,
  input logic [63:0] d1_chirho,
  input logic [63:0] d2_chirho,
  output logic done,
  output logic [63:0] result_chirho,
  input logic clk,
  input logic reset
);
// COMPONENT START: disj_unit_chirho
logic [63:0] result_reg_chirho_in;
logic result_reg_chirho_write_en;
logic result_reg_chirho_clk;
logic result_reg_chirho_reset;
logic [63:0] result_reg_chirho_out;
logic result_reg_chirho_done;
logic [63:0] or_gate_chirho_left;
logic [63:0] or_gate_chirho_right;
logic [63:0] or_gate_chirho_out;
logic invoke0_go_in;
logic invoke0_go_out;
logic invoke0_done_in;
logic invoke0_done_out;
std_reg # (
    .WIDTH(64)
) result_reg_chirho (
    .clk(result_reg_chirho_clk),
    .done(result_reg_chirho_done),
    .in(result_reg_chirho_in),
    .out(result_reg_chirho_out),
    .reset(result_reg_chirho_reset),
    .write_en(result_reg_chirho_write_en)
);
std_or # (
    .WIDTH(64)
) or_gate_chirho (
    .left(or_gate_chirho_left),
    .out(or_gate_chirho_out),
    .right(or_gate_chirho_right)
);
std_wire # (
    .WIDTH(1)
) invoke0_go (
    .in(invoke0_go_in),
    .out(invoke0_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke0_done (
    .in(invoke0_done_in),
    .out(invoke0_done_out)
);
wire _guard0 = 1;
wire _guard1 = invoke0_done_out;
wire _guard2 = invoke0_go_out;
wire _guard3 = invoke0_go_out;
wire _guard4 = invoke0_go_out;
wire _guard5 = invoke0_go_out;
assign done = _guard1;
assign result_chirho = result_reg_chirho_out;
assign result_reg_chirho_write_en = _guard2;
assign result_reg_chirho_clk = clk;
assign result_reg_chirho_reset = reset;
assign result_reg_chirho_in = or_gate_chirho_out;
assign or_gate_chirho_left = d1_chirho;
assign or_gate_chirho_right = d2_chirho;
assign invoke0_go_in = go;
assign invoke0_done_in = result_reg_chirho_done;
// COMPONENT END: disj_unit_chirho
endmodule
module singleton_check_chirho(
  input logic go,
  input logic [63:0] domain_chirho,
  output logic done,
  output logic is_singleton_chirho,
  input logic clk,
  input logic reset
);
// COMPONENT START: singleton_check_chirho
logic [63:0] sub_one_chirho_left;
logic [63:0] sub_one_chirho_right;
logic [63:0] sub_one_chirho_out;
logic [63:0] one_chirho_out;
logic [63:0] and_chirho_left;
logic [63:0] and_chirho_right;
logic [63:0] and_chirho_out;
logic [63:0] zero_chirho_out;
logic [63:0] eq_zero_chirho_left;
logic [63:0] eq_zero_chirho_right;
logic eq_zero_chirho_out;
logic [63:0] neq_zero_chirho_left;
logic [63:0] neq_zero_chirho_right;
logic neq_zero_chirho_out;
logic result_reg_chirho_in;
logic result_reg_chirho_write_en;
logic result_reg_chirho_clk;
logic result_reg_chirho_reset;
logic result_reg_chirho_out;
logic result_reg_chirho_done;
logic [63:0] x_minus_1_chirho_in;
logic x_minus_1_chirho_write_en;
logic x_minus_1_chirho_clk;
logic x_minus_1_chirho_reset;
logic [63:0] x_minus_1_chirho_out;
logic x_minus_1_chirho_done;
logic [63:0] and_result_chirho_in;
logic and_result_chirho_write_en;
logic and_result_chirho_clk;
logic and_result_chirho_reset;
logic [63:0] and_result_chirho_out;
logic and_result_chirho_done;
logic is_power_of_2_chirho_in;
logic is_power_of_2_chirho_write_en;
logic is_power_of_2_chirho_clk;
logic is_power_of_2_chirho_reset;
logic is_power_of_2_chirho_out;
logic is_power_of_2_chirho_done;
logic is_nonzero_chirho_in;
logic is_nonzero_chirho_write_en;
logic is_nonzero_chirho_clk;
logic is_nonzero_chirho_reset;
logic is_nonzero_chirho_out;
logic is_nonzero_chirho_done;
logic final_and_chirho_left;
logic final_and_chirho_right;
logic final_and_chirho_out;
logic pd_in;
logic pd_write_en;
logic pd_clk;
logic pd_reset;
logic pd_out;
logic pd_done;
logic pd0_in;
logic pd0_write_en;
logic pd0_clk;
logic pd0_reset;
logic pd0_out;
logic pd0_done;
logic [2:0] fsm_in;
logic fsm_write_en;
logic fsm_clk;
logic fsm_reset;
logic [2:0] fsm_out;
logic fsm_done;
logic invoke0_go_in;
logic invoke0_go_out;
logic invoke0_done_in;
logic invoke0_done_out;
logic invoke1_go_in;
logic invoke1_go_out;
logic invoke1_done_in;
logic invoke1_done_out;
logic invoke2_go_in;
logic invoke2_go_out;
logic invoke2_done_in;
logic invoke2_done_out;
logic invoke3_go_in;
logic invoke3_go_out;
logic invoke3_done_in;
logic invoke3_done_out;
logic invoke4_go_in;
logic invoke4_go_out;
logic invoke4_done_in;
logic invoke4_done_out;
logic par0_go_in;
logic par0_go_out;
logic par0_done_in;
logic par0_done_out;
logic tdcc_go_in;
logic tdcc_go_out;
logic tdcc_done_in;
logic tdcc_done_out;
std_sub # (
    .WIDTH(64)
) sub_one_chirho (
    .left(sub_one_chirho_left),
    .out(sub_one_chirho_out),
    .right(sub_one_chirho_right)
);
std_const # (
    .VALUE(64'd1),
    .WIDTH(64)
) one_chirho (
    .out(one_chirho_out)
);
std_and # (
    .WIDTH(64)
) and_chirho (
    .left(and_chirho_left),
    .out(and_chirho_out),
    .right(and_chirho_right)
);
std_const # (
    .VALUE(64'd0),
    .WIDTH(64)
) zero_chirho (
    .out(zero_chirho_out)
);
std_eq # (
    .WIDTH(64)
) eq_zero_chirho (
    .left(eq_zero_chirho_left),
    .out(eq_zero_chirho_out),
    .right(eq_zero_chirho_right)
);
std_neq # (
    .WIDTH(64)
) neq_zero_chirho (
    .left(neq_zero_chirho_left),
    .out(neq_zero_chirho_out),
    .right(neq_zero_chirho_right)
);
std_reg # (
    .WIDTH(1)
) result_reg_chirho (
    .clk(result_reg_chirho_clk),
    .done(result_reg_chirho_done),
    .in(result_reg_chirho_in),
    .out(result_reg_chirho_out),
    .reset(result_reg_chirho_reset),
    .write_en(result_reg_chirho_write_en)
);
std_reg # (
    .WIDTH(64)
) x_minus_1_chirho (
    .clk(x_minus_1_chirho_clk),
    .done(x_minus_1_chirho_done),
    .in(x_minus_1_chirho_in),
    .out(x_minus_1_chirho_out),
    .reset(x_minus_1_chirho_reset),
    .write_en(x_minus_1_chirho_write_en)
);
std_reg # (
    .WIDTH(64)
) and_result_chirho (
    .clk(and_result_chirho_clk),
    .done(and_result_chirho_done),
    .in(and_result_chirho_in),
    .out(and_result_chirho_out),
    .reset(and_result_chirho_reset),
    .write_en(and_result_chirho_write_en)
);
std_reg # (
    .WIDTH(1)
) is_power_of_2_chirho (
    .clk(is_power_of_2_chirho_clk),
    .done(is_power_of_2_chirho_done),
    .in(is_power_of_2_chirho_in),
    .out(is_power_of_2_chirho_out),
    .reset(is_power_of_2_chirho_reset),
    .write_en(is_power_of_2_chirho_write_en)
);
std_reg # (
    .WIDTH(1)
) is_nonzero_chirho (
    .clk(is_nonzero_chirho_clk),
    .done(is_nonzero_chirho_done),
    .in(is_nonzero_chirho_in),
    .out(is_nonzero_chirho_out),
    .reset(is_nonzero_chirho_reset),
    .write_en(is_nonzero_chirho_write_en)
);
std_and # (
    .WIDTH(1)
) final_and_chirho (
    .left(final_and_chirho_left),
    .out(final_and_chirho_out),
    .right(final_and_chirho_right)
);
std_reg # (
    .WIDTH(1)
) pd (
    .clk(pd_clk),
    .done(pd_done),
    .in(pd_in),
    .out(pd_out),
    .reset(pd_reset),
    .write_en(pd_write_en)
);
std_reg # (
    .WIDTH(1)
) pd0 (
    .clk(pd0_clk),
    .done(pd0_done),
    .in(pd0_in),
    .out(pd0_out),
    .reset(pd0_reset),
    .write_en(pd0_write_en)
);
std_reg # (
    .WIDTH(3)
) fsm (
    .clk(fsm_clk),
    .done(fsm_done),
    .in(fsm_in),
    .out(fsm_out),
    .reset(fsm_reset),
    .write_en(fsm_write_en)
);
std_wire # (
    .WIDTH(1)
) invoke0_go (
    .in(invoke0_go_in),
    .out(invoke0_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke0_done (
    .in(invoke0_done_in),
    .out(invoke0_done_out)
);
std_wire # (
    .WIDTH(1)
) invoke1_go (
    .in(invoke1_go_in),
    .out(invoke1_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke1_done (
    .in(invoke1_done_in),
    .out(invoke1_done_out)
);
std_wire # (
    .WIDTH(1)
) invoke2_go (
    .in(invoke2_go_in),
    .out(invoke2_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke2_done (
    .in(invoke2_done_in),
    .out(invoke2_done_out)
);
std_wire # (
    .WIDTH(1)
) invoke3_go (
    .in(invoke3_go_in),
    .out(invoke3_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke3_done (
    .in(invoke3_done_in),
    .out(invoke3_done_out)
);
std_wire # (
    .WIDTH(1)
) invoke4_go (
    .in(invoke4_go_in),
    .out(invoke4_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke4_done (
    .in(invoke4_done_in),
    .out(invoke4_done_out)
);
std_wire # (
    .WIDTH(1)
) par0_go (
    .in(par0_go_in),
    .out(par0_go_out)
);
std_wire # (
    .WIDTH(1)
) par0_done (
    .in(par0_done_in),
    .out(par0_done_out)
);
std_wire # (
    .WIDTH(1)
) tdcc_go (
    .in(tdcc_go_in),
    .out(tdcc_go_out)
);
std_wire # (
    .WIDTH(1)
) tdcc_done (
    .in(tdcc_done_in),
    .out(tdcc_done_out)
);
wire _guard0 = 1;
wire _guard1 = tdcc_done_out;
wire _guard2 = fsm_out == 3'd4;
wire _guard3 = fsm_out == 3'd0;
wire _guard4 = invoke0_done_out;
wire _guard5 = _guard3 & _guard4;
wire _guard6 = tdcc_go_out;
wire _guard7 = _guard5 & _guard6;
wire _guard8 = _guard2 | _guard7;
wire _guard9 = fsm_out == 3'd1;
wire _guard10 = invoke1_done_out;
wire _guard11 = _guard9 & _guard10;
wire _guard12 = tdcc_go_out;
wire _guard13 = _guard11 & _guard12;
wire _guard14 = _guard8 | _guard13;
wire _guard15 = fsm_out == 3'd2;
wire _guard16 = par0_done_out;
wire _guard17 = _guard15 & _guard16;
wire _guard18 = tdcc_go_out;
wire _guard19 = _guard17 & _guard18;
wire _guard20 = _guard14 | _guard19;
wire _guard21 = fsm_out == 3'd3;
wire _guard22 = invoke4_done_out;
wire _guard23 = _guard21 & _guard22;
wire _guard24 = tdcc_go_out;
wire _guard25 = _guard23 & _guard24;
wire _guard26 = _guard20 | _guard25;
wire _guard27 = fsm_out == 3'd1;
wire _guard28 = invoke1_done_out;
wire _guard29 = _guard27 & _guard28;
wire _guard30 = tdcc_go_out;
wire _guard31 = _guard29 & _guard30;
wire _guard32 = fsm_out == 3'd3;
wire _guard33 = invoke4_done_out;
wire _guard34 = _guard32 & _guard33;
wire _guard35 = tdcc_go_out;
wire _guard36 = _guard34 & _guard35;
wire _guard37 = fsm_out == 3'd4;
wire _guard38 = fsm_out == 3'd0;
wire _guard39 = invoke0_done_out;
wire _guard40 = _guard38 & _guard39;
wire _guard41 = tdcc_go_out;
wire _guard42 = _guard40 & _guard41;
wire _guard43 = fsm_out == 3'd2;
wire _guard44 = par0_done_out;
wire _guard45 = _guard43 & _guard44;
wire _guard46 = tdcc_go_out;
wire _guard47 = _guard45 & _guard46;
wire _guard48 = invoke4_done_out;
wire _guard49 = ~_guard48;
wire _guard50 = fsm_out == 3'd3;
wire _guard51 = _guard49 & _guard50;
wire _guard52 = tdcc_go_out;
wire _guard53 = _guard51 & _guard52;
wire _guard54 = invoke4_go_out;
wire _guard55 = invoke4_go_out;
wire _guard56 = pd_out;
wire _guard57 = invoke2_done_out;
wire _guard58 = _guard56 | _guard57;
wire _guard59 = ~_guard58;
wire _guard60 = par0_go_out;
wire _guard61 = _guard59 & _guard60;
wire _guard62 = invoke3_go_out;
wire _guard63 = invoke3_go_out;
wire _guard64 = invoke0_go_out;
wire _guard65 = invoke0_go_out;
wire _guard66 = invoke0_done_out;
wire _guard67 = ~_guard66;
wire _guard68 = fsm_out == 3'd0;
wire _guard69 = _guard67 & _guard68;
wire _guard70 = tdcc_go_out;
wire _guard71 = _guard69 & _guard70;
wire _guard72 = invoke1_go_out;
wire _guard73 = invoke1_go_out;
wire _guard74 = invoke2_go_out;
wire _guard75 = invoke2_go_out;
wire _guard76 = invoke3_go_out;
wire _guard77 = invoke3_go_out;
wire _guard78 = pd_out;
wire _guard79 = pd0_out;
wire _guard80 = _guard78 & _guard79;
wire _guard81 = invoke1_done_out;
wire _guard82 = ~_guard81;
wire _guard83 = fsm_out == 3'd1;
wire _guard84 = _guard82 & _guard83;
wire _guard85 = tdcc_go_out;
wire _guard86 = _guard84 & _guard85;
wire _guard87 = invoke1_go_out;
wire _guard88 = invoke1_go_out;
wire _guard89 = pd_out;
wire _guard90 = pd0_out;
wire _guard91 = _guard89 & _guard90;
wire _guard92 = invoke2_done_out;
wire _guard93 = par0_go_out;
wire _guard94 = _guard92 & _guard93;
wire _guard95 = _guard91 | _guard94;
wire _guard96 = invoke2_done_out;
wire _guard97 = par0_go_out;
wire _guard98 = _guard96 & _guard97;
wire _guard99 = pd_out;
wire _guard100 = pd0_out;
wire _guard101 = _guard99 & _guard100;
wire _guard102 = pd_out;
wire _guard103 = pd0_out;
wire _guard104 = _guard102 & _guard103;
wire _guard105 = invoke3_done_out;
wire _guard106 = par0_go_out;
wire _guard107 = _guard105 & _guard106;
wire _guard108 = _guard104 | _guard107;
wire _guard109 = invoke3_done_out;
wire _guard110 = par0_go_out;
wire _guard111 = _guard109 & _guard110;
wire _guard112 = pd_out;
wire _guard113 = pd0_out;
wire _guard114 = _guard112 & _guard113;
wire _guard115 = invoke2_go_out;
wire _guard116 = invoke2_go_out;
wire _guard117 = invoke4_go_out;
wire _guard118 = invoke4_go_out;
wire _guard119 = fsm_out == 3'd4;
wire _guard120 = invoke0_go_out;
wire _guard121 = invoke0_go_out;
wire _guard122 = pd0_out;
wire _guard123 = invoke3_done_out;
wire _guard124 = _guard122 | _guard123;
wire _guard125 = ~_guard124;
wire _guard126 = par0_go_out;
wire _guard127 = _guard125 & _guard126;
wire _guard128 = par0_done_out;
wire _guard129 = ~_guard128;
wire _guard130 = fsm_out == 3'd2;
wire _guard131 = _guard129 & _guard130;
wire _guard132 = tdcc_go_out;
wire _guard133 = _guard131 & _guard132;
assign done = _guard1;
assign is_singleton_chirho = result_reg_chirho_out;
assign fsm_write_en = _guard26;
assign fsm_clk = clk;
assign fsm_reset = reset;
assign fsm_in =
  _guard31 ? 3'd2 :
  _guard36 ? 3'd4 :
  _guard37 ? 3'd0 :
  _guard42 ? 3'd1 :
  _guard47 ? 3'd3 :
  3'd0;
always_comb begin
  if(~$onehot0({_guard47, _guard42, _guard37, _guard36, _guard31})) begin
    $fatal(2, "Multiple assignment to port `fsm.in'.");
end
end
assign invoke4_go_in = _guard53;
assign result_reg_chirho_write_en = _guard54;
assign result_reg_chirho_clk = clk;
assign result_reg_chirho_reset = reset;
assign result_reg_chirho_in = final_and_chirho_out;
assign invoke2_go_in = _guard61;
assign neq_zero_chirho_left = domain_chirho;
assign neq_zero_chirho_right = zero_chirho_out;
assign sub_one_chirho_left = domain_chirho;
assign sub_one_chirho_right = one_chirho_out;
assign invoke0_go_in = _guard71;
assign tdcc_go_in = go;
assign and_result_chirho_write_en = _guard72;
assign and_result_chirho_clk = clk;
assign and_result_chirho_reset = reset;
assign and_result_chirho_in = and_chirho_out;
assign is_power_of_2_chirho_write_en = _guard74;
assign is_power_of_2_chirho_clk = clk;
assign is_power_of_2_chirho_reset = reset;
assign is_power_of_2_chirho_in = eq_zero_chirho_out;
assign is_nonzero_chirho_write_en = _guard76;
assign is_nonzero_chirho_clk = clk;
assign is_nonzero_chirho_reset = reset;
assign is_nonzero_chirho_in = neq_zero_chirho_out;
assign invoke3_done_in = is_nonzero_chirho_done;
assign par0_done_in = _guard80;
assign invoke0_done_in = x_minus_1_chirho_done;
assign invoke1_go_in = _guard86;
assign invoke2_done_in = is_power_of_2_chirho_done;
assign and_chirho_left = domain_chirho;
assign and_chirho_right = x_minus_1_chirho_out;
assign pd_write_en = _guard95;
assign pd_clk = clk;
assign pd_reset = reset;
assign pd_in =
  _guard98 ? 1'd1 :
  _guard101 ? 1'd0 :
  1'd0;
always_comb begin
  if(~$onehot0({_guard101, _guard98})) begin
    $fatal(2, "Multiple assignment to port `pd.in'.");
end
end
assign pd0_write_en = _guard108;
assign pd0_clk = clk;
assign pd0_reset = reset;
assign pd0_in =
  _guard111 ? 1'd1 :
  _guard114 ? 1'd0 :
  1'd0;
always_comb begin
  if(~$onehot0({_guard114, _guard111})) begin
    $fatal(2, "Multiple assignment to port `pd0.in'.");
end
end
assign eq_zero_chirho_left = and_result_chirho_out;
assign eq_zero_chirho_right = zero_chirho_out;
assign final_and_chirho_left = is_power_of_2_chirho_out;
assign final_and_chirho_right = is_nonzero_chirho_out;
assign tdcc_done_in = _guard119;
assign x_minus_1_chirho_write_en = _guard120;
assign x_minus_1_chirho_clk = clk;
assign x_minus_1_chirho_reset = reset;
assign x_minus_1_chirho_in = sub_one_chirho_out;
assign invoke3_go_in = _guard127;
assign invoke4_done_in = result_reg_chirho_done;
assign invoke1_done_in = and_result_chirho_done;
assign par0_go_in = _guard133;
// COMPONENT END: singleton_check_chirho
endmodule
module branch_unit_chirho(
  input logic go,
  input logic [63:0] domain_chirho,
  output logic done,
  output logic [63:0] lowest_bit_chirho,
  output logic [63:0] rest_chirho,
  input logic clk,
  input logic reset
);
// COMPONENT START: branch_unit_chirho
logic [63:0] neg_chirho_left;
logic [63:0] neg_chirho_right;
logic [63:0] neg_chirho_out;
logic [63:0] sub_chirho_left;
logic [63:0] sub_chirho_right;
logic [63:0] sub_chirho_out;
logic [63:0] and1_chirho_left;
logic [63:0] and1_chirho_right;
logic [63:0] and1_chirho_out;
logic [63:0] and2_chirho_left;
logic [63:0] and2_chirho_right;
logic [63:0] and2_chirho_out;
logic [63:0] zero_chirho_out;
logic [63:0] one_chirho_out;
logic [63:0] lowest_reg_chirho_in;
logic lowest_reg_chirho_write_en;
logic lowest_reg_chirho_clk;
logic lowest_reg_chirho_reset;
logic [63:0] lowest_reg_chirho_out;
logic lowest_reg_chirho_done;
logic [63:0] rest_reg_chirho_in;
logic rest_reg_chirho_write_en;
logic rest_reg_chirho_clk;
logic rest_reg_chirho_reset;
logic [63:0] rest_reg_chirho_out;
logic rest_reg_chirho_done;
logic [63:0] neg_x_chirho_in;
logic neg_x_chirho_write_en;
logic neg_x_chirho_clk;
logic neg_x_chirho_reset;
logic [63:0] neg_x_chirho_out;
logic neg_x_chirho_done;
logic [63:0] x_minus_1_chirho_in;
logic x_minus_1_chirho_write_en;
logic x_minus_1_chirho_clk;
logic x_minus_1_chirho_reset;
logic [63:0] x_minus_1_chirho_out;
logic x_minus_1_chirho_done;
logic pd_in;
logic pd_write_en;
logic pd_clk;
logic pd_reset;
logic pd_out;
logic pd_done;
logic pd0_in;
logic pd0_write_en;
logic pd0_clk;
logic pd0_reset;
logic pd0_out;
logic pd0_done;
logic pd1_in;
logic pd1_write_en;
logic pd1_clk;
logic pd1_reset;
logic pd1_out;
logic pd1_done;
logic pd2_in;
logic pd2_write_en;
logic pd2_clk;
logic pd2_reset;
logic pd2_out;
logic pd2_done;
logic [1:0] fsm_in;
logic fsm_write_en;
logic fsm_clk;
logic fsm_reset;
logic [1:0] fsm_out;
logic fsm_done;
logic invoke0_go_in;
logic invoke0_go_out;
logic invoke0_done_in;
logic invoke0_done_out;
logic invoke1_go_in;
logic invoke1_go_out;
logic invoke1_done_in;
logic invoke1_done_out;
logic invoke2_go_in;
logic invoke2_go_out;
logic invoke2_done_in;
logic invoke2_done_out;
logic invoke3_go_in;
logic invoke3_go_out;
logic invoke3_done_in;
logic invoke3_done_out;
logic par0_go_in;
logic par0_go_out;
logic par0_done_in;
logic par0_done_out;
logic par1_go_in;
logic par1_go_out;
logic par1_done_in;
logic par1_done_out;
logic tdcc_go_in;
logic tdcc_go_out;
logic tdcc_done_in;
logic tdcc_done_out;
std_sub # (
    .WIDTH(64)
) neg_chirho (
    .left(neg_chirho_left),
    .out(neg_chirho_out),
    .right(neg_chirho_right)
);
std_sub # (
    .WIDTH(64)
) sub_chirho (
    .left(sub_chirho_left),
    .out(sub_chirho_out),
    .right(sub_chirho_right)
);
std_and # (
    .WIDTH(64)
) and1_chirho (
    .left(and1_chirho_left),
    .out(and1_chirho_out),
    .right(and1_chirho_right)
);
std_and # (
    .WIDTH(64)
) and2_chirho (
    .left(and2_chirho_left),
    .out(and2_chirho_out),
    .right(and2_chirho_right)
);
std_const # (
    .VALUE(64'd0),
    .WIDTH(64)
) zero_chirho (
    .out(zero_chirho_out)
);
std_const # (
    .VALUE(64'd1),
    .WIDTH(64)
) one_chirho (
    .out(one_chirho_out)
);
std_reg # (
    .WIDTH(64)
) lowest_reg_chirho (
    .clk(lowest_reg_chirho_clk),
    .done(lowest_reg_chirho_done),
    .in(lowest_reg_chirho_in),
    .out(lowest_reg_chirho_out),
    .reset(lowest_reg_chirho_reset),
    .write_en(lowest_reg_chirho_write_en)
);
std_reg # (
    .WIDTH(64)
) rest_reg_chirho (
    .clk(rest_reg_chirho_clk),
    .done(rest_reg_chirho_done),
    .in(rest_reg_chirho_in),
    .out(rest_reg_chirho_out),
    .reset(rest_reg_chirho_reset),
    .write_en(rest_reg_chirho_write_en)
);
std_reg # (
    .WIDTH(64)
) neg_x_chirho (
    .clk(neg_x_chirho_clk),
    .done(neg_x_chirho_done),
    .in(neg_x_chirho_in),
    .out(neg_x_chirho_out),
    .reset(neg_x_chirho_reset),
    .write_en(neg_x_chirho_write_en)
);
std_reg # (
    .WIDTH(64)
) x_minus_1_chirho (
    .clk(x_minus_1_chirho_clk),
    .done(x_minus_1_chirho_done),
    .in(x_minus_1_chirho_in),
    .out(x_minus_1_chirho_out),
    .reset(x_minus_1_chirho_reset),
    .write_en(x_minus_1_chirho_write_en)
);
std_reg # (
    .WIDTH(1)
) pd (
    .clk(pd_clk),
    .done(pd_done),
    .in(pd_in),
    .out(pd_out),
    .reset(pd_reset),
    .write_en(pd_write_en)
);
std_reg # (
    .WIDTH(1)
) pd0 (
    .clk(pd0_clk),
    .done(pd0_done),
    .in(pd0_in),
    .out(pd0_out),
    .reset(pd0_reset),
    .write_en(pd0_write_en)
);
std_reg # (
    .WIDTH(1)
) pd1 (
    .clk(pd1_clk),
    .done(pd1_done),
    .in(pd1_in),
    .out(pd1_out),
    .reset(pd1_reset),
    .write_en(pd1_write_en)
);
std_reg # (
    .WIDTH(1)
) pd2 (
    .clk(pd2_clk),
    .done(pd2_done),
    .in(pd2_in),
    .out(pd2_out),
    .reset(pd2_reset),
    .write_en(pd2_write_en)
);
std_reg # (
    .WIDTH(2)
) fsm (
    .clk(fsm_clk),
    .done(fsm_done),
    .in(fsm_in),
    .out(fsm_out),
    .reset(fsm_reset),
    .write_en(fsm_write_en)
);
std_wire # (
    .WIDTH(1)
) invoke0_go (
    .in(invoke0_go_in),
    .out(invoke0_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke0_done (
    .in(invoke0_done_in),
    .out(invoke0_done_out)
);
std_wire # (
    .WIDTH(1)
) invoke1_go (
    .in(invoke1_go_in),
    .out(invoke1_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke1_done (
    .in(invoke1_done_in),
    .out(invoke1_done_out)
);
std_wire # (
    .WIDTH(1)
) invoke2_go (
    .in(invoke2_go_in),
    .out(invoke2_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke2_done (
    .in(invoke2_done_in),
    .out(invoke2_done_out)
);
std_wire # (
    .WIDTH(1)
) invoke3_go (
    .in(invoke3_go_in),
    .out(invoke3_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke3_done (
    .in(invoke3_done_in),
    .out(invoke3_done_out)
);
std_wire # (
    .WIDTH(1)
) par0_go (
    .in(par0_go_in),
    .out(par0_go_out)
);
std_wire # (
    .WIDTH(1)
) par0_done (
    .in(par0_done_in),
    .out(par0_done_out)
);
std_wire # (
    .WIDTH(1)
) par1_go (
    .in(par1_go_in),
    .out(par1_go_out)
);
std_wire # (
    .WIDTH(1)
) par1_done (
    .in(par1_done_in),
    .out(par1_done_out)
);
std_wire # (
    .WIDTH(1)
) tdcc_go (
    .in(tdcc_go_in),
    .out(tdcc_go_out)
);
std_wire # (
    .WIDTH(1)
) tdcc_done (
    .in(tdcc_done_in),
    .out(tdcc_done_out)
);
wire _guard0 = 1;
wire _guard1 = tdcc_done_out;
wire _guard2 = fsm_out == 2'd2;
wire _guard3 = fsm_out == 2'd0;
wire _guard4 = par0_done_out;
wire _guard5 = _guard3 & _guard4;
wire _guard6 = tdcc_go_out;
wire _guard7 = _guard5 & _guard6;
wire _guard8 = _guard2 | _guard7;
wire _guard9 = fsm_out == 2'd1;
wire _guard10 = par1_done_out;
wire _guard11 = _guard9 & _guard10;
wire _guard12 = tdcc_go_out;
wire _guard13 = _guard11 & _guard12;
wire _guard14 = _guard8 | _guard13;
wire _guard15 = fsm_out == 2'd0;
wire _guard16 = par0_done_out;
wire _guard17 = _guard15 & _guard16;
wire _guard18 = tdcc_go_out;
wire _guard19 = _guard17 & _guard18;
wire _guard20 = fsm_out == 2'd2;
wire _guard21 = fsm_out == 2'd1;
wire _guard22 = par1_done_out;
wire _guard23 = _guard21 & _guard22;
wire _guard24 = tdcc_go_out;
wire _guard25 = _guard23 & _guard24;
wire _guard26 = invoke0_go_out;
wire _guard27 = invoke0_go_out;
wire _guard28 = pd1_out;
wire _guard29 = invoke2_done_out;
wire _guard30 = _guard28 | _guard29;
wire _guard31 = ~_guard30;
wire _guard32 = par1_go_out;
wire _guard33 = _guard31 & _guard32;
wire _guard34 = invoke0_go_out;
wire _guard35 = invoke0_go_out;
wire _guard36 = par1_done_out;
wire _guard37 = ~_guard36;
wire _guard38 = fsm_out == 2'd1;
wire _guard39 = _guard37 & _guard38;
wire _guard40 = tdcc_go_out;
wire _guard41 = _guard39 & _guard40;
wire _guard42 = invoke2_go_out;
wire _guard43 = invoke2_go_out;
wire _guard44 = pd1_out;
wire _guard45 = pd2_out;
wire _guard46 = _guard44 & _guard45;
wire _guard47 = invoke2_done_out;
wire _guard48 = par1_go_out;
wire _guard49 = _guard47 & _guard48;
wire _guard50 = _guard46 | _guard49;
wire _guard51 = invoke2_done_out;
wire _guard52 = par1_go_out;
wire _guard53 = _guard51 & _guard52;
wire _guard54 = pd1_out;
wire _guard55 = pd2_out;
wire _guard56 = _guard54 & _guard55;
wire _guard57 = pd_out;
wire _guard58 = invoke0_done_out;
wire _guard59 = _guard57 | _guard58;
wire _guard60 = ~_guard59;
wire _guard61 = par0_go_out;
wire _guard62 = _guard60 & _guard61;
wire _guard63 = pd_out;
wire _guard64 = pd0_out;
wire _guard65 = _guard63 & _guard64;
wire _guard66 = invoke3_go_out;
wire _guard67 = invoke3_go_out;
wire _guard68 = pd1_out;
wire _guard69 = pd2_out;
wire _guard70 = _guard68 & _guard69;
wire _guard71 = invoke3_done_out;
wire _guard72 = par1_go_out;
wire _guard73 = _guard71 & _guard72;
wire _guard74 = _guard70 | _guard73;
wire _guard75 = invoke3_done_out;
wire _guard76 = par1_go_out;
wire _guard77 = _guard75 & _guard76;
wire _guard78 = pd1_out;
wire _guard79 = pd2_out;
wire _guard80 = _guard78 & _guard79;
wire _guard81 = pd0_out;
wire _guard82 = invoke1_done_out;
wire _guard83 = _guard81 | _guard82;
wire _guard84 = ~_guard83;
wire _guard85 = par0_go_out;
wire _guard86 = _guard84 & _guard85;
wire _guard87 = invoke2_go_out;
wire _guard88 = invoke2_go_out;
wire _guard89 = pd1_out;
wire _guard90 = pd2_out;
wire _guard91 = _guard89 & _guard90;
wire _guard92 = invoke1_go_out;
wire _guard93 = invoke1_go_out;
wire _guard94 = pd_out;
wire _guard95 = pd0_out;
wire _guard96 = _guard94 & _guard95;
wire _guard97 = invoke0_done_out;
wire _guard98 = par0_go_out;
wire _guard99 = _guard97 & _guard98;
wire _guard100 = _guard96 | _guard99;
wire _guard101 = invoke0_done_out;
wire _guard102 = par0_go_out;
wire _guard103 = _guard101 & _guard102;
wire _guard104 = pd_out;
wire _guard105 = pd0_out;
wire _guard106 = _guard104 & _guard105;
wire _guard107 = pd_out;
wire _guard108 = pd0_out;
wire _guard109 = _guard107 & _guard108;
wire _guard110 = invoke1_done_out;
wire _guard111 = par0_go_out;
wire _guard112 = _guard110 & _guard111;
wire _guard113 = _guard109 | _guard112;
wire _guard114 = invoke1_done_out;
wire _guard115 = par0_go_out;
wire _guard116 = _guard114 & _guard115;
wire _guard117 = pd_out;
wire _guard118 = pd0_out;
wire _guard119 = _guard117 & _guard118;
wire _guard120 = fsm_out == 2'd2;
wire _guard121 = invoke1_go_out;
wire _guard122 = invoke1_go_out;
wire _guard123 = invoke3_go_out;
wire _guard124 = invoke3_go_out;
wire _guard125 = pd2_out;
wire _guard126 = invoke3_done_out;
wire _guard127 = _guard125 | _guard126;
wire _guard128 = ~_guard127;
wire _guard129 = par1_go_out;
wire _guard130 = _guard128 & _guard129;
wire _guard131 = par0_done_out;
wire _guard132 = ~_guard131;
wire _guard133 = fsm_out == 2'd0;
wire _guard134 = _guard132 & _guard133;
wire _guard135 = tdcc_go_out;
wire _guard136 = _guard134 & _guard135;
assign done = _guard1;
assign rest_chirho = rest_reg_chirho_out;
assign lowest_bit_chirho = lowest_reg_chirho_out;
assign fsm_write_en = _guard14;
assign fsm_clk = clk;
assign fsm_reset = reset;
assign fsm_in =
  _guard19 ? 2'd1 :
  _guard20 ? 2'd0 :
  _guard25 ? 2'd2 :
  2'd0;
always_comb begin
  if(~$onehot0({_guard25, _guard20, _guard19})) begin
    $fatal(2, "Multiple assignment to port `fsm.in'.");
end
end
assign neg_x_chirho_write_en = _guard26;
assign neg_x_chirho_clk = clk;
assign neg_x_chirho_reset = reset;
assign neg_x_chirho_in = neg_chirho_out;
assign invoke2_go_in = _guard33;
assign neg_chirho_left = zero_chirho_out;
assign neg_chirho_right = domain_chirho;
assign par1_go_in = _guard41;
assign and1_chirho_left = domain_chirho;
assign and1_chirho_right = neg_x_chirho_out;
assign pd1_write_en = _guard50;
assign pd1_clk = clk;
assign pd1_reset = reset;
assign pd1_in =
  _guard53 ? 1'd1 :
  _guard56 ? 1'd0 :
  1'd0;
always_comb begin
  if(~$onehot0({_guard56, _guard53})) begin
    $fatal(2, "Multiple assignment to port `pd1.in'.");
end
end
assign invoke0_go_in = _guard62;
assign tdcc_go_in = go;
assign invoke3_done_in = rest_reg_chirho_done;
assign par0_done_in = _guard65;
assign rest_reg_chirho_write_en = _guard66;
assign rest_reg_chirho_clk = clk;
assign rest_reg_chirho_reset = reset;
assign rest_reg_chirho_in = and2_chirho_out;
assign pd2_write_en = _guard74;
assign pd2_clk = clk;
assign pd2_reset = reset;
assign pd2_in =
  _guard77 ? 1'd1 :
  _guard80 ? 1'd0 :
  1'd0;
always_comb begin
  if(~$onehot0({_guard80, _guard77})) begin
    $fatal(2, "Multiple assignment to port `pd2.in'.");
end
end
assign invoke0_done_in = neg_x_chirho_done;
assign invoke1_go_in = _guard86;
assign lowest_reg_chirho_write_en = _guard87;
assign lowest_reg_chirho_clk = clk;
assign lowest_reg_chirho_reset = reset;
assign lowest_reg_chirho_in = and1_chirho_out;
assign invoke2_done_in = lowest_reg_chirho_done;
assign par1_done_in = _guard91;
assign sub_chirho_left = domain_chirho;
assign sub_chirho_right = one_chirho_out;
assign pd_write_en = _guard100;
assign pd_clk = clk;
assign pd_reset = reset;
assign pd_in =
  _guard103 ? 1'd1 :
  _guard106 ? 1'd0 :
  1'd0;
always_comb begin
  if(~$onehot0({_guard106, _guard103})) begin
    $fatal(2, "Multiple assignment to port `pd.in'.");
end
end
assign pd0_write_en = _guard113;
assign pd0_clk = clk;
assign pd0_reset = reset;
assign pd0_in =
  _guard116 ? 1'd1 :
  _guard119 ? 1'd0 :
  1'd0;
always_comb begin
  if(~$onehot0({_guard119, _guard116})) begin
    $fatal(2, "Multiple assignment to port `pd0.in'.");
end
end
assign tdcc_done_in = _guard120;
assign x_minus_1_chirho_write_en = _guard121;
assign x_minus_1_chirho_clk = clk;
assign x_minus_1_chirho_reset = reset;
assign x_minus_1_chirho_in = sub_chirho_out;
assign and2_chirho_left = domain_chirho;
assign and2_chirho_right = x_minus_1_chirho_out;
assign invoke3_go_in = _guard130;
assign invoke1_done_in = x_minus_1_chirho_done;
assign par0_go_in = _guard136;
// COMPONENT END: branch_unit_chirho
endmodule
module main(
  input logic go,
  input logic clk,
  input logic reset,
  output logic done
);
// COMPONENT START: main
logic [63:0] var0_chirho_in;
logic var0_chirho_write_en;
logic var0_chirho_clk;
logic var0_chirho_reset;
logic [63:0] var0_chirho_out;
logic var0_chirho_done;
logic [63:0] var3_chirho_in;
logic var3_chirho_write_en;
logic var3_chirho_clk;
logic var3_chirho_reset;
logic [63:0] var3_chirho_out;
logic var3_chirho_done;
logic unify_chirho_go;
logic [63:0] unify_chirho_d1_chirho;
logic [63:0] unify_chirho_d2_chirho;
logic unify_chirho_done;
logic [63:0] unify_chirho_result_chirho;
logic unify_chirho_failed_chirho;
logic unify_chirho_clk;
logic unify_chirho_reset;
logic [63:0] full_chirho_out;
logic [63:0] test1_chirho_out;
logic [63:0] test2_chirho_out;
logic [1:0] fsm_in;
logic fsm_write_en;
logic fsm_clk;
logic fsm_reset;
logic [1:0] fsm_out;
logic fsm_done;
logic init_vars_chirho_go_in;
logic init_vars_chirho_go_out;
logic init_vars_chirho_done_in;
logic init_vars_chirho_done_out;
logic invoke0_go_in;
logic invoke0_go_out;
logic invoke0_done_in;
logic invoke0_done_out;
logic invoke1_go_in;
logic invoke1_go_out;
logic invoke1_done_in;
logic invoke1_done_out;
logic tdcc_go_in;
logic tdcc_go_out;
logic tdcc_done_in;
logic tdcc_done_out;
std_reg # (
    .WIDTH(64)
) var0_chirho (
    .clk(var0_chirho_clk),
    .done(var0_chirho_done),
    .in(var0_chirho_in),
    .out(var0_chirho_out),
    .reset(var0_chirho_reset),
    .write_en(var0_chirho_write_en)
);
std_reg # (
    .WIDTH(64)
) var3_chirho (
    .clk(var3_chirho_clk),
    .done(var3_chirho_done),
    .in(var3_chirho_in),
    .out(var3_chirho_out),
    .reset(var3_chirho_reset),
    .write_en(var3_chirho_write_en)
);
unify_unit_chirho unify_chirho (
    .clk(unify_chirho_clk),
    .d1_chirho(unify_chirho_d1_chirho),
    .d2_chirho(unify_chirho_d2_chirho),
    .done(unify_chirho_done),
    .failed_chirho(unify_chirho_failed_chirho),
    .go(unify_chirho_go),
    .reset(unify_chirho_reset),
    .result_chirho(unify_chirho_result_chirho)
);
std_const # (
    .VALUE(64'd18446744073709551615),
    .WIDTH(64)
) full_chirho (
    .out(full_chirho_out)
);
std_const # (
    .VALUE(64'd255),
    .WIDTH(64)
) test1_chirho (
    .out(test1_chirho_out)
);
std_const # (
    .VALUE(64'd240),
    .WIDTH(64)
) test2_chirho (
    .out(test2_chirho_out)
);
std_reg # (
    .WIDTH(2)
) fsm (
    .clk(fsm_clk),
    .done(fsm_done),
    .in(fsm_in),
    .out(fsm_out),
    .reset(fsm_reset),
    .write_en(fsm_write_en)
);
std_wire # (
    .WIDTH(1)
) init_vars_chirho_go (
    .in(init_vars_chirho_go_in),
    .out(init_vars_chirho_go_out)
);
std_wire # (
    .WIDTH(1)
) init_vars_chirho_done (
    .in(init_vars_chirho_done_in),
    .out(init_vars_chirho_done_out)
);
std_wire # (
    .WIDTH(1)
) invoke0_go (
    .in(invoke0_go_in),
    .out(invoke0_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke0_done (
    .in(invoke0_done_in),
    .out(invoke0_done_out)
);
std_wire # (
    .WIDTH(1)
) invoke1_go (
    .in(invoke1_go_in),
    .out(invoke1_go_out)
);
std_wire # (
    .WIDTH(1)
) invoke1_done (
    .in(invoke1_done_in),
    .out(invoke1_done_out)
);
std_wire # (
    .WIDTH(1)
) tdcc_go (
    .in(tdcc_go_in),
    .out(tdcc_go_out)
);
std_wire # (
    .WIDTH(1)
) tdcc_done (
    .in(tdcc_done_in),
    .out(tdcc_done_out)
);
wire _guard0 = 1;
wire _guard1 = tdcc_done_out;
wire _guard2 = fsm_out == 2'd3;
wire _guard3 = fsm_out == 2'd0;
wire _guard4 = init_vars_chirho_done_out;
wire _guard5 = _guard3 & _guard4;
wire _guard6 = tdcc_go_out;
wire _guard7 = _guard5 & _guard6;
wire _guard8 = _guard2 | _guard7;
wire _guard9 = fsm_out == 2'd1;
wire _guard10 = invoke0_done_out;
wire _guard11 = _guard9 & _guard10;
wire _guard12 = tdcc_go_out;
wire _guard13 = _guard11 & _guard12;
wire _guard14 = _guard8 | _guard13;
wire _guard15 = fsm_out == 2'd2;
wire _guard16 = invoke1_done_out;
wire _guard17 = _guard15 & _guard16;
wire _guard18 = tdcc_go_out;
wire _guard19 = _guard17 & _guard18;
wire _guard20 = _guard14 | _guard19;
wire _guard21 = fsm_out == 2'd0;
wire _guard22 = init_vars_chirho_done_out;
wire _guard23 = _guard21 & _guard22;
wire _guard24 = tdcc_go_out;
wire _guard25 = _guard23 & _guard24;
wire _guard26 = fsm_out == 2'd3;
wire _guard27 = fsm_out == 2'd2;
wire _guard28 = invoke1_done_out;
wire _guard29 = _guard27 & _guard28;
wire _guard30 = tdcc_go_out;
wire _guard31 = _guard29 & _guard30;
wire _guard32 = fsm_out == 2'd1;
wire _guard33 = invoke0_done_out;
wire _guard34 = _guard32 & _guard33;
wire _guard35 = tdcc_go_out;
wire _guard36 = _guard34 & _guard35;
wire _guard37 = init_vars_chirho_go_out;
wire _guard38 = invoke1_go_out;
wire _guard39 = _guard37 | _guard38;
wire _guard40 = init_vars_chirho_go_out;
wire _guard41 = invoke1_go_out;
wire _guard42 = invoke0_done_out;
wire _guard43 = ~_guard42;
wire _guard44 = fsm_out == 2'd1;
wire _guard45 = _guard43 & _guard44;
wire _guard46 = tdcc_go_out;
wire _guard47 = _guard45 & _guard46;
wire _guard48 = invoke1_done_out;
wire _guard49 = ~_guard48;
wire _guard50 = fsm_out == 2'd2;
wire _guard51 = _guard49 & _guard50;
wire _guard52 = tdcc_go_out;
wire _guard53 = _guard51 & _guard52;
wire _guard54 = init_vars_chirho_go_out;
wire _guard55 = init_vars_chirho_go_out;
wire _guard56 = fsm_out == 2'd3;
wire _guard57 = invoke0_go_out;
wire _guard58 = invoke0_go_out;
wire _guard59 = invoke0_go_out;
wire _guard60 = init_vars_chirho_done_out;
wire _guard61 = ~_guard60;
wire _guard62 = fsm_out == 2'd0;
wire _guard63 = _guard61 & _guard62;
wire _guard64 = tdcc_go_out;
wire _guard65 = _guard63 & _guard64;
assign done = _guard1;
assign fsm_write_en = _guard20;
assign fsm_clk = clk;
assign fsm_reset = reset;
assign fsm_in =
  _guard25 ? 2'd1 :
  _guard26 ? 2'd0 :
  _guard31 ? 2'd3 :
  _guard36 ? 2'd2 :
  2'd0;
always_comb begin
  if(~$onehot0({_guard36, _guard31, _guard26, _guard25})) begin
    $fatal(2, "Multiple assignment to port `fsm.in'.");
end
end
assign var0_chirho_write_en = _guard39;
assign var0_chirho_clk = clk;
assign var0_chirho_reset = reset;
assign var0_chirho_in =
  _guard40 ? full_chirho_out :
  _guard41 ? unify_chirho_result_chirho :
  'x;
always_comb begin
  if(~$onehot0({_guard41, _guard40})) begin
    $fatal(2, "Multiple assignment to port `var0_chirho.in'.");
end
end
assign invoke0_go_in = _guard47;
assign tdcc_go_in = go;
assign invoke0_done_in = unify_chirho_done;
assign invoke1_go_in = _guard53;
assign var3_chirho_write_en = _guard54;
assign var3_chirho_clk = clk;
assign var3_chirho_reset = reset;
assign var3_chirho_in = full_chirho_out;
assign tdcc_done_in = _guard56;
assign unify_chirho_clk = clk;
assign unify_chirho_go = _guard57;
assign unify_chirho_reset = reset;
assign unify_chirho_d1_chirho =
  _guard58 ? test1_chirho_out :
  64'd0;
assign unify_chirho_d2_chirho =
  _guard59 ? test2_chirho_out :
  64'd0;
assign invoke1_done_in = var0_chirho_done;
assign init_vars_chirho_go_in = _guard65;
assign init_vars_chirho_done_in = var3_chirho_done;
// COMPONENT END: main
endmodule
