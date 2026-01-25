;;; appendo_chirho.scm - faster-miniKanren appendo benchmark ☧

(load "faster-miniKanren/mk.scm")

;;; appendo relation
(define (appendo-chirho l-chirho s-chirho out-chirho)
  (conde
    [(== '() l-chirho) (== s-chirho out-chirho)]
    [(fresh (h-chirho t-chirho res-chirho)
       (== `(,h-chirho . ,t-chirho) l-chirho)
       (== `(,h-chirho . ,res-chirho) out-chirho)
       (appendo-chirho t-chirho s-chirho res-chirho))]))

;;; Timing helper
(define (time-it-chirho thunk-chirho)
  (let ((start-chirho (current-inexact-milliseconds)))
    (let ((result-chirho (thunk-chirho)))
      (let ((elapsed-chirho (- (current-inexact-milliseconds) start-chirho)))
        (values result-chirho elapsed-chirho)))))

;;; Forward benchmark: appendo '(1 2 3) '(4 5 6) q
(define (benchmark-forward-chirho)
  (let-values ([(results-chirho elapsed-chirho)
                (time-it-chirho
                  (lambda ()
                    (run* (q-chirho)
                      (appendo-chirho '(1 2 3) '(4 5 6) q-chirho))))])
    (printf "Forward: ~a results in ~a ms~n"
            (length results-chirho) elapsed-chirho)))

;;; Backward benchmark: appendo a b '(1 2 3 4 5)
(define (benchmark-backward-chirho)
  (let-values ([(results-chirho elapsed-chirho)
                (time-it-chirho
                  (lambda ()
                    (run* (q-chirho)
                      (fresh (a-chirho b-chirho)
                        (== q-chirho `(,a-chirho ,b-chirho))
                        (appendo-chirho a-chirho b-chirho '(1 2 3 4 5))))))])
    (printf "Backward: ~a results in ~a ms~n"
            (length results-chirho) elapsed-chirho)))

;;; Run benchmarks
(printf "=== faster-miniKanren appendo Benchmark ☧ ===~n")
(do ((i-chirho 0 (+ i-chirho 1)))
    ((= i-chirho 10))
  (benchmark-forward-chirho)
  (benchmark-backward-chirho))
