(defun fak-tco (n acc)
  (if (= n 1)
      acc
      (fak-tco (- n 1) (* n acc))))

(format t "~A~%" (fak-tco 100000 1))
