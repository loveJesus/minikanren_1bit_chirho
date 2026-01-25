; sign.sl - Synthesize sign function
; sign(x) = -1 if x < 0, 0 if x == 0, 1 if x > 0

(set-logic LIA)
(synth-fun sign ((x Int)) Int
    ((Start Int) (StartBool Bool))
    ((Start Int (x 0 1 -1 (ite StartBool Start Start)))
     (StartBool Bool ((<= Start Start) (>= Start Start) (= Start Start)))))

; I/O examples
(constraint (= (sign 5) 1))
(constraint (= (sign -5) -1))
(constraint (= (sign 0) 0))
(constraint (= (sign 100) 1))
(constraint (= (sign -100) -1))

(check-synth)
