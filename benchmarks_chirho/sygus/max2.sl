; max2.sl - Synthesize max of two integers
; Expected solution: (ite (<= x y) y x)

(set-logic LIA)
(synth-fun max2 ((x Int) (y Int)) Int
    ((Start Int) (StartBool Bool))
    ((Start Int (x y (ite StartBool Start Start)))
     (StartBool Bool ((<= x y) (<= y x)))))

(declare-var x Int)
(declare-var y Int)

; I/O examples
(constraint (= (max2 0 1) 1))
(constraint (= (max2 1 0) 1))
(constraint (= (max2 3 5) 5))
(constraint (= (max2 5 3) 5))
(constraint (= (max2 7 7) 7))

; Semantic constraint
(constraint (>= (max2 x y) x))
(constraint (>= (max2 x y) y))

(check-synth)
