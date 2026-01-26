%% CLP(FD) Benchmark Suite ☧
%% Comparison baseline for miniKanren 1-bit matrix operations
%%
%% For God so loved the world that he gave his only begotten Son,
%% that whoever believes in him should not perish but have eternal life.
%% John 3:16
%%
%% Usage: swipl -g "run_all_benchmarks_chirho, halt." clpfd_bench_chirho.pl
%%
%% Requires SWI-Prolog with CLP(FD): sudo apt install swi-prolog

:- use_module(library(clpfd)).
:- use_module(library(statistics)).

%% ============================================================================
%% N-Queens Benchmark
%% ============================================================================

n_queens_chirho(N, Qs) :-
    length(Qs, N),
    Qs ins 1..N,
    safe_queens_chirho(Qs),
    label(Qs).

safe_queens_chirho([]).
safe_queens_chirho([Q|Qs]) :-
    safe_from_chirho(Qs, Q, 1),
    safe_queens_chirho(Qs).

safe_from_chirho([], _, _).
safe_from_chirho([Q|Qs], Q0, D) :-
    Q0 #\= Q,
    abs(Q0 - Q) #\= D,
    D1 #= D + 1,
    safe_from_chirho(Qs, Q0, D1).

bench_nqueens_chirho(N, Count, TimeMs) :-
    statistics(walltime, _),
    findall(Qs, n_queens_chirho(N, Qs), Solutions),
    statistics(walltime, [_, TimeMs]),
    length(Solutions, Count).

%% ============================================================================
%% Sudoku Benchmark
%% ============================================================================

sudoku_chirho(Rows) :-
    length(Rows, 9),
    maplist(same_length(Rows), Rows),
    append(Rows, Vs), Vs ins 1..9,
    maplist(all_distinct, Rows),
    transpose(Rows, Columns),
    maplist(all_distinct, Columns),
    Rows = [As,Bs,Cs,Ds,Es,Fs,Gs,Hs,Is],
    blocks_chirho(As, Bs, Cs),
    blocks_chirho(Ds, Es, Fs),
    blocks_chirho(Gs, Hs, Is),
    maplist(label, Rows).

blocks_chirho([], [], []).
blocks_chirho([N1,N2,N3|Ns1], [N4,N5,N6|Ns2], [N7,N8,N9|Ns3]) :-
    all_distinct([N1,N2,N3,N4,N5,N6,N7,N8,N9]),
    blocks_chirho(Ns1, Ns2, Ns3).

%% Easy puzzle (many clues)
easy_puzzle_chirho([
    [5,3,_,_,7,_,_,_,_],
    [6,_,_,1,9,5,_,_,_],
    [_,9,8,_,_,_,_,6,_],
    [8,_,_,_,6,_,_,_,3],
    [4,_,_,8,_,3,_,_,1],
    [7,_,_,_,2,_,_,_,6],
    [_,6,_,_,_,_,2,8,_],
    [_,_,_,4,1,9,_,_,5],
    [_,_,_,_,8,_,_,7,9]
]).

%% Hard puzzle (17 clues - minimal)
hard_puzzle_chirho([
    [_,_,_,_,_,_,_,1,_],
    [_,_,_,_,_,2,_,_,3],
    [_,_,_,4,_,_,_,_,_],
    [_,_,_,_,_,_,5,_,_],
    [4,_,1,6,_,_,_,_,_],
    [_,_,7,1,_,_,_,_,_],
    [_,5,_,_,_,_,2,_,_],
    [_,_,_,_,8,_,_,4,_],
    [_,3,_,9,1,_,_,_,_]
]).

%% Escargot (one of the hardest)
escargot_puzzle_chirho([
    [1,_,_,_,_,7,_,9,_],
    [_,3,_,_,2,_,_,_,8],
    [_,_,9,6,_,_,5,_,_],
    [_,_,5,3,_,_,9,_,_],
    [_,1,_,_,8,_,_,_,2],
    [6,_,_,_,_,4,_,_,_],
    [3,_,_,_,_,_,_,1,_],
    [_,4,_,_,_,_,_,_,7],
    [_,_,7,_,_,_,3,_,_]
]).

bench_sudoku_chirho(Name, Puzzle, TimeMs) :-
    copy_term(Puzzle, P),
    statistics(walltime, _),
    sudoku_chirho(P),
    statistics(walltime, [_, TimeMs]),
    format("~w solved in ~wms~n", [Name, TimeMs]).

%% ============================================================================
%% Send More Money
%% ============================================================================

send_more_money_chirho([S,E,N,D,M,O,R,Y]) :-
    Digits = [S,E,N,D,M,O,R,Y],
    Digits ins 0..9,
    all_distinct(Digits),
    S #\= 0, M #\= 0,
                 1000*S + 100*E + 10*N + D
    +            1000*M + 100*O + 10*R + E
    #= 10000*M + 1000*O + 100*N + 10*E + Y,
    label(Digits).

bench_send_more_money_chirho(TimeMs) :-
    statistics(walltime, _),
    findall(Sol, send_more_money_chirho(Sol), Solutions),
    statistics(walltime, [_, TimeMs]),
    length(Solutions, Count),
    format("SEND+MORE=MONEY: ~w solutions in ~wms~n", [Count, TimeMs]).

%% ============================================================================
%% Main benchmark runner
%% ============================================================================

run_all_benchmarks_chirho :-
    format("~n=== CLP(FD) Benchmark Suite ===~n~n"),

    %% N-Queens
    format("--- N-Queens ---~n"),
    bench_nqueens_chirho(8, Count8, Time8),
    format("8-Queens: ~w solutions in ~wms~n", [Count8, Time8]),

    bench_nqueens_chirho(10, Count10, Time10),
    format("10-Queens: ~w solutions in ~wms~n", [Count10, Time10]),

    bench_nqueens_chirho(12, Count12, Time12),
    format("12-Queens: ~w solutions in ~wms~n", [Count12, Time12]),

    %% Sudoku
    format("~n--- Sudoku ---~n"),
    easy_puzzle_chirho(Easy),
    bench_sudoku_chirho(easy, Easy, _),

    hard_puzzle_chirho(Hard),
    bench_sudoku_chirho(hard_17clue, Hard, _),

    escargot_puzzle_chirho(Escargot),
    bench_sudoku_chirho(escargot, Escargot, _),

    %% Send More Money
    format("~n--- SEND+MORE=MONEY ---~n"),
    bench_send_more_money_chirho(_),

    format("~n=== Benchmark Complete ===~n"),
    format("Compare these times with rust_chirho benchmarks.~n"),
    format("Run: cargo bench --bench criterion_bench_chirho -- 'NQueens|Sudoku'~n~n").

%% For interactive use
:- initialization((
    format("CLP(FD) Benchmark Suite loaded.~n"),
    format("Run: run_all_benchmarks_chirho.~n")
)).

%% Soli Deo Gloria ☧
