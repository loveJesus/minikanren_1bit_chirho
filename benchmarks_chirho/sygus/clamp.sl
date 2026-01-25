; clamp.sl - Synthesize clamp(x, lo, hi) = max(lo, min(hi, x))
; Clamps x to range [lo, hi]

(set-logic LIA)
(synth-fun clamp ((x Int) (lo Int) (hi Int)) Int
    ((Start Int) (StartBool Bool))
    ((Start Int (x lo hi (ite StartBool Start Start)))
     (StartBool Bool ((<= Start Start) (>= Start Start)))))

; I/O examples: clamp to [0, 10]
(constraint (= (clamp 5 0 10) 5))   ; in range
(constraint (= (clamp -3 0 10) 0))  ; below lo
(constraint (= (clamp 15 0 10) 10)) ; above hi
(constraint (= (clamp 0 0 10) 0))   ; at lo
(constraint (= (clamp 10 0 10) 10)) ; at hi

; Different range [5, 15]
(constraint (= (clamp 3 5 15) 5))
(constraint (= (clamp 20 5 15) 15))
(constraint (= (clamp 10 5 15) 10))

(check-synth)
