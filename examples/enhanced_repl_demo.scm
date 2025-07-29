;; Enhanced REPL 演示文件
;; 使用: arbores examples/enhanced_repl_demo.scm

;; 基础算术
(+ 1 2 3 4 5)

;; 定义变量
(define pi 3.14159)
(define radius 5)

;; 使用变量
(* pi radius radius)

;; 定义函数
(define square (lambda (x) (* x x)))
(square 7)

;; 条件表达式
(define factorial 
  (lambda (n)
    (if (= n 0)
        1
        (* n (factorial (- n 1))))))

(factorial 5)

;; 列表操作
(define my-list (list 1 2 3 4 5))
(car my-list)
(cdr my-list)

;; 复杂嵌套表达式
(begin
  (define x 10)
  (define y 20)
  (+ x y (* x y)))
