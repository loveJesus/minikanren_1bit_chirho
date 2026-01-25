; abs.sl - Synthesize absolute value
; Expected solution: (ite (>= x 0) x (- 0 x))

(set-logic LIA)
(synth-fun abs ((x Int)) Int
    ((Start Int) (StartBool Bool))
    ((Start Int (x 0 (- 0 Start) (ite StartBool Start Start)))
     (StartBool Bool ((>= Start Start) (<= Start Start)))))

; I/O examples
(constraint (= (abs 5) 5))
(constraint (= (abs -5) 5))
(constraint (= (abs 0) 0))
(constraint (= (abs -10) 10))
(constraint (= (abs 10) 10))

(check-synth)
