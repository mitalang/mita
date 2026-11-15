(defun countdown (n)
  (if (= n 1)
      n
      (countdown (- n 1))))

(format t "~A~%" (countdown 100000))
