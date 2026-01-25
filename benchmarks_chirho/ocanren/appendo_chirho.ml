(* appendo_chirho.ml - OCanren appendo benchmark ☧ *)

open OCanren
open OCanren.Std

(* Define appendo relation *)
let rec appendo_chirho l_chirho s_chirho out_chirho =
  conde [
    (l_chirho === nil ()) &&& (s_chirho === out_chirho);
    fresh (h_chirho t_chirho res_chirho)
      (l_chirho === h_chirho % t_chirho)
      (out_chirho === h_chirho % res_chirho)
      (appendo_chirho t_chirho s_chirho res_chirho)
  ]

(* Benchmark: forward - appendo [1;2;3] [4;5;6] q *)
let benchmark_forward_chirho () =
  let start_chirho = Unix.gettimeofday () in
  let results_chirho =
    run q (fun q_chirho ->
      appendo_chirho (list (!![1; 2; 3])) (list (!![4; 5; 6])) q_chirho
    ) (fun q_chirho -> q_chirho#reify (Std.List.reify OCanren.reify))
  in
  let elapsed_chirho = Unix.gettimeofday () -. start_chirho in
  Printf.printf "Forward: %d results in %.6f s\n"
    (Stream.length results_chirho) elapsed_chirho

(* Benchmark: backward - appendo a b [1;2;3;4;5] *)
let benchmark_backward_chirho () =
  let start_chirho = Unix.gettimeofday () in
  let results_chirho =
    run qr (fun a_chirho b_chirho ->
      appendo_chirho a_chirho b_chirho (list (!![1; 2; 3; 4; 5]))
    ) (fun a_chirho b_chirho ->
      (a_chirho#reify (Std.List.reify OCanren.reify),
       b_chirho#reify (Std.List.reify OCanren.reify))
    )
  in
  let elapsed_chirho = Unix.gettimeofday () -. start_chirho in
  Printf.printf "Backward: %d results in %.6f s\n"
    (Stream.length results_chirho) elapsed_chirho

let () =
  Printf.printf "=== OCanren appendo Benchmark ☧ ===\n";
  for _ = 1 to 10 do
    benchmark_forward_chirho ();
    benchmark_backward_chirho ()
  done
