;; 引用和特殊形式示例

;; 引用
'hello                 ; 符号: hello
'(1 2 3)               ; 列表: (1 2 3)
(quote foo)            ; 符号: foo

;; 条件表达式
(if #t "yes" "no")     ; 结果: "yes"
(if #f "yes" "no")     ; 结果: "no"
(if (< 3 5) 
    "3 is less than 5" 
    "3 is not less than 5")  ; 结果: "3 is less than 5"

;; 目前还未实现的特性（计划中）:
;; Lambda 表达式
;; (lambda (x) (* x x))
;; 
;; 变量定义
;; (define pi 3.14159)
;; 
;; Let 绑定
;; (let ((x 10) (y 20)) (+ x y))
