(* nqueens_chirho.ml - OCanren N-Queens benchmark ☧ *)

open OCanren
open OCanren.Std

(* Check if queens don't attack each other *)
let rec safe_chirho q_chirho qs_chirho d_chirho =
  conde [
    (qs_chirho === nil ());
    fresh (q1_chirho rest_chirho)
      (qs_chirho === q1_chirho % rest_chirho)
      (* q != q1 *)
      (q_chirho =/= q1_chirho)
      (* q != q1 + d (diagonal) *)
      (fresh (sum_chirho)
        (Nat.addo q1_chirho (Nat.of_int d_chirho) sum_chirho)
        (q_chirho =/= sum_chirho))
      (* q != q1 - d (other diagonal) *)
      (conde [
        (Nat.of_int d_chirho > q1_chirho);
        fresh (diff_chirho)
          (Nat.subo q1_chirho (Nat.of_int d_chirho) diff_chirho)
          (q_chirho =/= diff_chirho)
      ])
      (safe_chirho q_chirho rest_chirho (d_chirho + 1))
  ]

(* Place N queens *)
let rec queens_chirho n_chirho qs_chirho =
  if n_chirho = 0 then
    qs_chirho === nil ()
  else
    fresh (q_chirho rest_chirho)
      (queens_chirho (n_chirho - 1) rest_chirho)
      (* q in 1..8 *)
      (conde (List.init 8 (fun i_chirho -> q_chirho === Nat.of_int (i_chirho + 1))))
      (safe_chirho q_chirho rest_chirho 1)
      (qs_chirho === q_chirho % rest_chirho)

(* Benchmark *)
let benchmark_nqueens_chirho n_chirho =
  let start_chirho = Unix.gettimeofday () in
  let results_chirho =
    run q (fun q_chirho -> queens_chirho n_chirho q_chirho)
      (fun q_chirho -> q_chirho#reify (Std.List.reify Nat.reify))
  in
  let count_chirho = Stream.length results_chirho in
  let elapsed_chirho = Unix.gettimeofday () -. start_chirho in
  Printf.printf "N-Queens %d: %d solutions in %.6f s\n" n_chirho count_chirho elapsed_chirho;
  count_chirho

let () =
  Printf.printf "=== OCanren N-Queens Benchmark ☧ ===\n";
  let count_chirho = benchmark_nqueens_chirho 8 in
  assert (count_chirho = 92)
