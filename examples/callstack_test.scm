;; 测试 callstack 的示例代码
;; 创建一个函数调用链来触发错误

;; 定义一个会调用另一个函数的函数
(define (func-a x)
  (func-b x))

;; 定义一个会调用第三个函数的函数  
(define (func-b x)
  (func-c x))

;; 定义一个会产生错误的函数
(define (func-c x)
  (/ x 0))

;; 调用 func-a 来触发调用链错误
(func-a 42)
