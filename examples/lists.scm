;; 列表操作示例
(list 1 2 3 4)         ; 创建列表: (1 2 3 4)
(cons 1 (cons 2 3))    ; 创建 cons 对: (1 2 . 3)
(cons 'a (cons 'b '()))  ; 创建列表: (a b)

;; 访问列表元素
(car (list 1 2 3))     ; 获取第一个元素: 1
(cdr (list 1 2 3))     ; 获取剩余元素: (2 3)

;; 嵌套列表
(list (list 1 2) (list 3 4))  ; ((1 2) (3 4))

;; 类型谓词
(null? '())            ; #t
(pair? (cons 1 2))     ; #t
(number? 42)           ; #t
(symbol? 'hello)       ; #t
(string? "world")      ; #t
