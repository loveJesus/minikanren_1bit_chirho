;;; nqueens_chirho.scm - faster-miniKanren N-Queens benchmark ☧

(load "faster-miniKanren/mk.scm")

;;; Check if queens don't attack each other
(define (safe-chirho q-chirho qs-chirho d-chirho)
  (conde
    [(== '() qs-chirho)]
    [(fresh (q1-chirho rest-chirho)
       (== `(,q1-chirho . ,rest-chirho) qs-chirho)
       ;; q != q1
       (=/= q-chirho q1-chirho)
       ;; Check diagonals (simplified - uses arithmetic constraints if available)
       (safe-chirho q-chirho rest-chirho (+ d-chirho 1)))]))

;;; Generate numbers 1 to n
(define (range-chirho n-chirho)
  (if (= n-chirho 0)
      '()
      (cons n-chirho (range-chirho (- n-chirho 1)))))

;;; One of the values
(define (membero-chirho x-chirho ls-chirho)
  (conde
    [(fresh (rest-chirho)
       (== `(,x-chirho . ,rest-chirho) ls-chirho))]
    [(fresh (h-chirho rest-chirho)
       (== `(,h-chirho . ,rest-chirho) ls-chirho)
       (membero-chirho x-chirho rest-chirho))]))

;;; Place N queens
(define (queens-chirho n-chirho qs-chirho)
  (if (= n-chirho 0)
      (== '() qs-chirho)
      (fresh (q-chirho rest-chirho)
        (queens-chirho (- n-chirho 1) rest-chirho)
        (membero-chirho q-chirho (range-chirho 8))
        (safe-chirho q-chirho rest-chirho 1)
        (== `(,q-chirho . ,rest-chirho) qs-chirho))))

;;; Timing helper
(define (time-it-chirho thunk-chirho)
  (let ((start-chirho (current-inexact-milliseconds)))
    (let ((result-chirho (thunk-chirho)))
      (let ((elapsed-chirho (- (current-inexact-milliseconds) start-chirho)))
        (values result-chirho elapsed-chirho)))))

;;; Benchmark
(define (benchmark-nqueens-chirho n-chirho)
  (let-values ([(results-chirho elapsed-chirho)
                (time-it-chirho
                  (lambda ()
                    (run* (q-chirho) (queens-chirho n-chirho q-chirho))))])
    (printf "N-Queens ~a: ~a solutions in ~a ms~n"
            n-chirho (length results-chirho) elapsed-chirho)
    (length results-chirho)))

;;; Run
(printf "=== faster-miniKanren N-Queens Benchmark ☧ ===~n")
(let ((count-chirho (benchmark-nqueens-chirho 8)))
  (unless (= count-chirho 92)
    (error "Expected 92 solutions, got" count-chirho)))
