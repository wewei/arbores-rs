;; 测试详细 callstack 的示例代码
;; 每个函数定义在不同的行，调用也在不同的行

;; 第5行：定义一个会调用另一个函数的函数
(define (func-a x)
  (func-b x))

;; 第9行：定义一个会调用第三个函数的函数  
(define (func-b x)
  (func-c x))

;; 第13行：定义一个会产生错误的函数
(define (func-c x)
  (/ x 0))

;; 第17行：调用 func-a 来触发调用链错误
(func-a 42)
