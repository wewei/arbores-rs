;; 演示扩展符号字符支持，特别是冒号
;; 这对于实现 Arbores 接口非常重要

;; 定义一些带命名空间的函数（使用冒号）
(define arb:create (lambda (code desc) 
    (cons code desc)))

(define arb:search (lambda (pattern)
    (list pattern "found")))

(define math:square (lambda (x) (* x x)))
(define math:cube (lambda (x) (* x x x)))

;; 测试其他 R5RS 允许的特殊字符
(define my-func! (lambda (x) (+ x 1)))
(define is-zero? (lambda (x) (= x 0)))
(define var@ 42)
(define test_var 100)

;; 使用这些函数
(arb:create "(define factorial ...)" "阶乘函数")
(arb:search "math")

(math:square 5)
(math:cube 3)

(my-func! 10)
(is-zero? 0)
(is-zero? 5)

var@
test_var
