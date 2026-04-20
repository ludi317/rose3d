;*******************************************************************************************
;* Rose-Shaped Parametric Surface, by Paul Nylander, bugman123.com, 3/11/09
;*******************************************************************************************

;Basic Functions
(defun mod (a b / c) (setq c (rem a b)) (if (> c 0) c (+ c b))); well-behaved (always return positive)
(defun ² (x) (* x x))

;Converts Polygon into AutoCAD's Triangle Elements, only works for convex polygons
;(TriangulateElem '(1 2 3 4 5 6 7 8 9 10)) -> '((1 2 -3) (-1 3 -4) (-1 4 -5) (-1 5 -6) (-1 6 -7) (-1 7 -8) (-1 8 -9) (-1 9 10))
(defun TriangulateElem (elem / n i i0 i1 i2 elems) (setq n (length elem))
 (if (= n 3) (list elem) (progn
  (setq i0 (car elem) i1 (caddr elem) elems (list (list i0 (cadr elem) (- i1))) i 3)
  (while (< i (1- n)) (setq i2 (nth i elem) elems (append elems (list (list (- i0) i1 (- i2)))) i1 i2 i (1+ i)))
  (append elems (list (list (- i0) i1 (last elem))))
 ));i
);d

(defun mesh (nodes elems1 / elems2 nnode nelem elem)
 (setq elems2 (apply 'append (mapcar 'TriangulateElem elems1))); introduces AutoCAD's negative index notation
 (setq nnode (length nodes) nelem (length elems2))
 (entmake (list '(0 . "POLYLINE") '(70 . 64) (cons 71 nnode) (cons 72 nelem)))
 (foreach p nodes (entmake (list '(0 . "VERTEX") (cons 10 p) '(70 . 192))))
 (foreach elem elems2 (entmake (list '(0 . "VERTEX") '(10 0.0 0.0 0.0) '(70 . 128)
  (cons 71 (car elem)) (cons 72 (cadr elem)) (cons 73 (caddr elem))
  (cons 74 (- (abs (car elem))))
 )));f
 (entmake '((0 . "SEQEND")))
);d

(defun ParametricPlot3D (f u1 u2 du v1 v2 dv / u v i j i1 i2 i3 i4 imax jmax nodes elems)
 (setq imax (fix (+ (/ (- u2 u1) du) 0.5)) jmax (fix (+ (/ (- v2 v1) dv) 0.5)))
 (setq j 0); calculate nodes
 (while (<= j jmax) (setq v (+ v1 (* j dv)) i 0)
  (while (<= i imax) (setq u (+ u1 (* i du)) nodes (append nodes (list (f u v))) i (1+ i)))
  (setq j (1+ j))
 );w
 (setq i 1); assemble quad elements
 (while (<= i imax) (setq j 0)
  (while (< j jmax) (setq
   i1 (+ i (* j (1+ imax))) i2 (1+ i1) i3 (+ i1 (1+ imax) 1) i4 (+ i1 (1+ imax))
   elems (append elems (list (list i1 i2 i3 i4)))
   j (1+ j)
  ));w
  (setq i (1+ i))
 );w
 (mesh nodes elems)
);d

;Rose-Shaped Parametric Surface
(setq theta1 (/ (* -20 pi) 9) theta2 (* 15 pi) x0 0.7831546645625248)
(defun Rose (x1 theta / Ø y1 x r)
 (setq Ø (* 0.5 pi (exp (- (/ theta 8 pi)))) thetanew theta)
 (setq y1 (* 1.9565284531299512 (² x1) (² (- (* 1.2768869870150188 x1) 1.0)) (sin Ø)))
 (setq x (- 1.0 (* 0.5 (² (- (* 1.25 (² (- 1.0 (/ (mod (* 3.6 theta) (* 2 pi)) pi)))) 0.25)))))
 (setq r (* x (+ (* x1 (sin Ø)) (* y1 (cos Ø)))))
 (list (* r (sin thetanew)) (* r (cos thetanew)) (* x (- (* x1 (cos Ø)) (* y1 (sin Ø)))))
);d
(ParametricPlot3D Rose 1.0e-6 1.0 (/ 1.0 24) theta1 theta2 (/ (- theta2 theta1) 575))

(command "vpoint" (list -1.0 -1.0 1.0)); (command "zoom" "extents")