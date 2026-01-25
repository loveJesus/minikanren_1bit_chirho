; min2.sl - Synthesize min of two integers
; Expected solution: (ite (<= x y) x y)

(set-logic LIA)
(synth-fun min2 ((x Int) (y Int)) Int
    ((Start Int) (StartBool Bool))
    ((Start Int (x y (ite StartBool Start Start)))
     (StartBool Bool ((<= x y) (<= y x)))))

(declare-var x Int)
(declare-var y Int)

; I/O examples
(constraint (= (min2 0 1) 0))
(constraint (= (min2 1 0) 0))
(constraint (= (min2 3 5) 3))
(constraint (= (min2 5 3) 3))
(constraint (= (min2 7 7) 7))

(check-synth)
