# Project Arbores

## Arbores 是什么？

Arbores 是 Lisp 方言 Scheme 的一个 Rust 实现。
不同于一般的解释器，Arbores 是一个有状态的解释器，它维护着一个本地的（未来或许可以支持远程的）代码数据库，并提供 API 查询，引用，编辑库中的 Scheme 代码。
在人工智能快速发展的时代，Arbores 旨在给 AI Agent 提供一个存储、积累、查询结构化知识的仓库，Scheme 语言是这种知识的载体。从 AI 的视角看，Arbores 是一个特殊的 RAG，专门保存可以用 Scheme S-表达式描述的结构化知识。

> Arbores 并不是一门 Lisp 方言的名字，Arbores 支持的编程语言就是 Scheme (目标支持 R7RS)。

## 整体架构

Arbores 整体为三层架构

* Arbores CLI (Command Line Interface)，提供用户或 AI 可以调用的 CLI 接口以及 Repl 运行环境。
* Arbores Interpreter, Arbores 的 Scheme 解释器。
* Arbores Repository Manager，Arbores 的 Scheme 代码仓库管理器。

由于采用分层架构，CLI 并不会直接和 Repository Manager 通讯，而是通过 Interpreter 执行特定的 Scheme 代码实现对 Repository 的读写。

## 代码库结构

```text
arbores-rs/
├── Cargo.toml
├── docs/
│   │── Project_Arbores.md      # 技术文档
│   └── ...
├── src/
│   ├── main.rs                 # 主入口
│   ├── cli/                    # CLI 模块
│   │   ├── mod.rs
│   │   └── ...
│   ├── interpreter/            # 解释器模块
│   │   ├── mod.rs
│   │   └── ...
│   └── repo_manager/           # 仓库管理器模块
│       ├── mod.rs
│       └── ...
├── tests/                      # 集成测试目录
│   └── ...
└── README.md                   # 项目对外介绍
```

## 代码规范

Project Arbores 采用函数式风格 Rust。详细代码规范参考 [Coding_Conventions.md](./Coding_Conventions.md)。



## Arbores 知识库的结构
### S-Expression 为核心的存储
Arbores 知识库以 S-Expression 为原子节点，每个 S-Expression 有

* 全局的唯一 ID (64位整数)
* Optional 语义描述（字符串）
* Optional 建议的 symbol names（字符串列表）
* Optional 类型描述

### S-Expression 索引
对于 S-Expressions 建立以下索引

* 对于有语义描述的 S-Expression 节点，建立基于描述的语义索引，支持语义检索
* 对于有建议 symbol names 的节点，建立基于 symbol names 的语义和倒排索引，支持语义检索和模糊匹配搜索
* 建立 S-Expression 之间的引用关系表，可以正向、反向查一个 S-Expression 之间的依赖关系

### Builtin 函数集成
* **ID 分配策略：** ID % 65536 < 256 的范围预留给 builtin 函数（如 `arb:xxx` 接口），为未来扩展预留空间
* **符号注册：** 所有 builtin 函数都在 Arbores 注册表中注册，可通过 symbol names 搜索（如搜索 "arb:" 前缀）
* **元数据特征：** builtin 函数的 `code` 字段为 `#f`，表示其为系统内置实现而非用户定义代码

### 知识库版本管理
采用 Copy on Write 的 Immutable 存储
* 用森林结构维护所有的 S-Expression ID 表
* 每次更新，创建新的根节点，复用不变的子节点，更新改变的子节点
* 保留所有版本的更新关系，也可能呈现树结构，如 v3 -> v4, v3 -> v5 -> v6
* 保留当前版本的 trace（类似 Git 的 reflog），例如 v3 -> v4 -> v3 -> v5 -> v6
* 保留每个版本之间的 delta 和 reverse delta (修改指令的 S-Expression)，用于回滚索引
* 索引仅保持当前版本，当版本回滚时，可以用 reverse delta 指令调整索引


## Arbores 接口设计
### 接口设计原则
**Scheme code is first citizen:** 执行 Scheme 代码是 Arbores 接口的 Single source of truth，一切的 CLI 接口和 HTTP 接口都是对 Scheme 接口的封装。
**Primitive input:** 接口输入不要引入复杂的数据结构，尽量使用 Primitive types，以便 CLI 和 HTTP 接口编码。
**S-Expression output:** Arbores 使用 S-Expression 作为主要数据结构，可以表达复杂的嵌套数据，例如：
```scheme
;; 类似 JSON 对象的结构示例
'(("name" . "John")
  ("age" . 30)
  ("email" . "john@example.com")
  ("address" . (("street" . "123 Main St")
                ("city" . "Anytown")
                ("zipcode" . "12345")))
  ("skills" . ("programming" "design" "communication")))
```

这种结构类似于 JSON，但使用 S-Expression 语法，便于在 Scheme 环境中自然操作。
**Immutable storage:** Arbores 的数据存储采用基于 copy on write 的只读设计。所有的读接口支持 optional 的版本号。
**权限层级设计:** 所有接口按照权限级别分为三层：
* **T0 (系统级):** 禁止通过 Scheme 脚本执行，仅允许直接调用 (如版本切换)
* **T1 (读写级):** 允许通过读写执行接口调用，可修改知识库 (如创建、更新 S-Expression)  
* **T2 (只读级):** 允许通过只读执行接口调用，仅能查询数据 (如语义查询、元信息查询)

**接口命名约定:** 所有 Arbores 接口都使用 `arb:` 前缀，以区别于标准 Scheme 函数。

### 接口设计
#### 版本管理接口
##### arb:current-version [T2]
**功能：** 获取当前活跃的知识库版本号
**输入：** 无
**输出：** 当前版本的 ID (64位整数)
**示例：**
```scheme
(arb:current-version)
;; => 12345
```

##### arb:reflog [T2]
**功能：** 获取当前版本的变动轨迹，类似 Git 的 reflog
**输入：** 
  * 可选：跳过条目数 (整数，默认为0)
  * 可选：最大返回条目数 (整数，默认为50)
**输出：** 版本变动轨迹列表，每个条目包含：
```scheme
'(("version-id" . 版本ID)
  ("timestamp" . 时间戳字符串)
  ("description" . 变动描述字符串))
```
**示例：**
```scheme
(arb:reflog)
;; => '((("version-id" . 12345)
;;        ("timestamp" . "2025-07-29T10:30:00Z")
;;        ("description" . "创建新的数学函数库"))
;;       (("version-id" . 12344)
;;        ("timestamp" . "2025-07-29T10:25:00Z")
;;        ("description" . "更新排序算法实现"))
;;       (("version-id" . 12343)
;;        ("timestamp" . "2025-07-29T10:20:00Z")
;;        ("description" . "回滚到稳定版本")))

(arb:reflog 10 5)
;; => 跳过前10条记录，返回接下来的最多5条记录

;; reflog 可能包含重复版本号的示例
(arb:reflog 0 10)
;; => '((("version-id" . 12345) ...)
;;       (("version-id" . 12344) ...)
;;       (("version-id" . 12343) ...)
;;       (("version-id" . 12344) ...)  ;; 切换回12344
;;       (("version-id" . 12342) ...)
;;       ...)
```

##### arb:version-info [T2]
**功能：** 查询指定版本的前置版本及其间的修改指令
**输入：** 版本 ID (64位整数)
**输出：** 版本依赖信息
```scheme
'(("parent-version" . 父版本ID或#f)
  ("forward-delta" . 正向修改指令S-Expression或#f)
  ("reverse-delta" . 反向修改指令S-Expression或#f))
```
**示例：**
```scheme
(arb:version-info 12345)
;; => '(("parent-version" . 12344)
;;       ("forward-delta" . '(arb:create 
;;                             "(define (new-func x) (* x 2))"
;;                             '()
;;                             "新创建的函数"
;;                             "function"
;;                             '("new-func")))
;;       ("reverse-delta" . '(arb:delete 789)))

;; 更新操作的示例
(arb:version-info 12346)
;; => '(("parent-version" . 12345)
;;       ("forward-delta" . '(arb:update 123
;;                             '(("description" . "更新后的描述")
;;                               ("code" . "(define (updated-func) ...)"))))
;;       ("reverse-delta" . '(arb:update 123
;;                             '(("description" . "原始描述")
;;                               ("code" . "(define (original-func) ...)")))))
```

##### arb:version-chain [T2]
**功能：** 查询指定版本的完整依赖链，追溯到根版本
**输入：** 
  * 版本 ID (64位整数)
  * 可选：最大返回条目数 (整数，默认为100)
**输出：** 从根版本到指定版本的完整路径列表
```scheme
'(版本ID1 版本ID2 ... 目标版本ID)
```
**示例：**
```scheme
(arb:version-chain 12345)
;; => '(1 123 456 789 12344 12345)

(arb:version-chain 12345 3)
;; => '(12344 12345) ;; 只返回最近3个版本
```

##### arb:version-successors [T2]
**功能：** 查询指定版本的所有直接后继版本
**输入：** 版本 ID (64位整数)
**输出：** 后继版本 ID 列表
```scheme
'(后继版本ID1 后继版本ID2 ...)
```
**示例：**
```scheme
(arb:version-successors 12344)
;; => '(12345 12346) ;; 版本12344有两个分支
```

##### arb:switch-version [T0]
**功能：** 将当前活跃版本切换到指定版本，并更新相关索引
**输入：** 目标版本 ID (64位整数)
**输出：**
  * 成功：新的当前版本 ID
  * 失败：错误信息 S-Expression
```scheme
;; 成功示例
目标版本ID

;; 失败示例  
'(("error" . "version-not-found")
  ("message" . "指定的版本不存在")
  ("version-id" . 请求的版本ID))
```
**示例：**
```scheme
(arb:switch-version 12340)
;; => 12340 ;; 成功切换

(arb:switch-version 99999)
;; => '(("error" . "version-not-found")
;;       ("message" . "指定的版本不存在")
;;       ("version-id" . 99999))
```

#### 只读访问接口
所有的只读访问接口都是基于当前版本。

##### arb:semantic-search [T2]
**功能：** 根据语义查询 Arbores 知识库中的 S-Expressions（基于描述内容的语义理解）
**输入：** Query 字符串
**输出：** 一组最相关的 S-Expression，包含 ID 和相关度
**示例：**
```scheme
(arb:semantic-search "排序算法")
;; => '((("id" . 123) ("score" . 0.95) ("description" . "快速排序实现"))
;;       (("id" . 456) ("score" . 0.87) ("description" . "归并排序算法"))
;;       (("id" . 789) ("score" . 0.72) ("description" . "冒泡排序示例")))
```

##### arb:search-by-symbol [T2]
**功能：** 根据 symbol name 模式匹配查询 S-Expressions（支持前缀匹配、通配符等）
**输入：** 
  * Pattern 字符串（支持通配符 * 和 ?）
  * 可选：匹配模式 ("exact", "prefix", "wildcard", "regex")，默认为 "prefix"
**输出：** 匹配的 S-Expression 列表，按匹配质量排序
**示例：**
```scheme
;; 前缀匹配（默认）
(arb:search-by-symbol "arb:")
;; => '((("id" . 1) ("symbol-names" . ("arb:current-version")) ("description" . "获取当前版本"))
;;       (("id" . 2) ("symbol-names" . ("arb:reflog")) ("description" . "版本历史查询"))
;;       (("id" . 17) ("symbol-names" . ("arb:semantic-search")) ("description" . "语义搜索接口")))

;; 通配符匹配
(arb:search-by-symbol "quick*" "wildcard")
;; => '((("id" . 123) ("symbol-names" . ("quicksort" "qsort")) ("description" . "快速排序实现")))

;; 精确匹配
(arb:search-by-symbol "factorial" "exact")
;; => '((("id" . 456) ("symbol-names" . ("factorial" "fact")) ("description" . "阶乘函数")))

;; 正则表达式匹配
(arb:search-by-symbol "sort$" "regex")
;; => '((("id" . 123) ("symbol-names" . ("quicksort")) ("description" . "快速排序实现"))
;;       (("id" . 789) ("symbol-names" . ("bubble-sort")) ("description" . "冒泡排序实现")))
```

##### arb:get-metadata [T2]
**功能：** 根据 ID 查询 S-Expression 的元数据
**输入：** 给定 S-Expression ID
**输出：** 元信息，包括描述，类型，依赖，代码等
**示例：**
```scheme
;; 用户定义的 S-Expression
(arb:get-metadata 123)
;; => '(("id" . 123)
;;       ("description" . "快速排序算法实现")
;;       ("type" . "function")
;;       ("symbol-names" . ("quicksort" "qsort"))
;;       ("dependencies" . (456 789))
;;       ("code" . "(define (quicksort lst) ...)"))

;; builtin 函数 S-Expression
(arb:get-metadata 1)
;; => '(("id" . 1)
;;       ("description" . "获取当前活跃的知识库版本号")
;;       ("type" . "builtin-function")
;;       ("symbol-names" . ("arb:current-version"))
;;       ("dependencies" . ())
;;       ("code" . #f))  ;; builtin 函数的 code 为 #f
```

##### arb:get-dependencies [T2]
**功能：** 根据 ID 查询 S-Expression 依赖的 S-Expressions
**输入：** S-Expression ID
**输出：** 该 S-Expression 依赖的 S-Expression 的 ID 列表
**示例：**
```scheme
(arb:get-dependencies 123)
;; => '(456 789 101) ;; 快速排序依赖这些S-Expression
```

##### arb:get-dependents [T2]
**功能：** 根据 ID 查询依赖该 S-Expression 的 S-Expressions
**输入：** S-Expression ID
**输出：** 依赖该 S-Expression 的 S-Expression 的 ID 列表
**示例：**
```scheme
(arb:get-dependents 456)
;; => '(123 202 303) ;; 这些S-Expression依赖ID为456的表达式
```

##### arb:closure [T2]
**功能：** 给定一组 S-Expression，生成包含所有依赖闭包的完整代码
**输入：** S-Expression ID list
**输出：** 完整的闭包 Scheme 代码
**示例：**
```scheme
(arb:closure '(123 456))
;; => "(define (helper-func x) ...)
;;     (define (partition lst pivot) ...)
;;     (define (quicksort lst) ...)
;;     (define (merge-sort lst) ...)"
```


#### 修改接口
所有的修改接口都是基于当前版本，执行后会产生新版本。

##### arb:create [T1]
**功能：** 创建一个新的 S-Expression
**输入：** 给定 Scheme 代码，依赖的 S-Expression（ID 到局部 symbol name 的映射），语义描述，类型描述，建议的 symbol names
**输出：**
  * 成功：返回包含新 S-Expression ID 和新版本 ID 的结果
    ```scheme
    '(("s-expression-id" . 新S-ExpressionID)
      ("new-version-id" . 新版本ID))
    ```
  * 失败：返回错误列表
**示例：**
```scheme
(arb:create 
  "(define (factorial n) (if (<= n 1) 1 (* n (factorial (- n 1)))))"
  '((456 . "multiplication"))  ;; 依赖映射
  "计算阶乘的递归函数"           ;; 语义描述
  "function"                   ;; 类型
  '("factorial" "fact"))       ;; 建议的symbol names
;; => '(("s-expression-id" . 12346)
;;       ("new-version-id" . 12347))
```

##### arb:update [T1]
**功能：** 更新一个已经存在的 S-Expression
**输入：** S-Expression 的 ID，和要修改的内容的 key/value pairs
**输出：**
  * 成功：返回包含 S-Expression ID 和新版本 ID 的结果
    ```scheme
    '(("s-expression-id" . S-ExpressionID)
      ("new-version-id" . 新版本ID))
    ```
  * 失败：返回错误列表
**示例：**
```scheme
(arb:update 123 
  '(("description" . "优化后的快速排序算法")
    ("code" . "(define (quicksort lst) ;; 新的实现...")))
;; => '(("s-expression-id" . 123)
;;       ("new-version-id" . 12348))
```

##### arb:delete [T1]
**功能：** 删除一个已经存在的 S-Expression
**输入：** S-Expression 的 ID (64位整数)
**输出：**
  * 成功：返回包含被删除的 S-Expression ID 和新版本 ID 的结果
    ```scheme
    '(("s-expression-id" . 被删除的S-ExpressionID)
      ("new-version-id" . 新版本ID))
    ```
  * 失败：返回错误列表
**示例：**
```scheme
(arb:delete 123)
;; => '(("s-expression-id" . 123)
;;       ("new-version-id" . 12349))

;; 删除不存在的S-Expression
(arb:delete 99999)
;; => '(("error" . "s-expression-not-found")
;;       ("message" . "指定的S-Expression不存在")
;;       ("s-expression-id" . 99999))
```

#### 执行接口

##### arb:eval-readonly [T2]
**功能：** 基于 Arbores 知识库执行 Scheme 代码，执行时对 Arbores 知识库做保护
**API 权限：** T2
**输入：** 待执行的 S-Expression
**输出：**
  * 成功：执行结果
  * 失败：错误信息
**示例：**
```scheme
(arb:eval-readonly '(+ 1 2 3))
;; => 6

(arb:eval-readonly '(let ((nums '(3 1 4 1 5)))
                          (quicksort nums)))  ;; 使用知识库中的quicksort
;; => '(1 1 3 4 5)
```

##### arb:eval [T1]
**功能：** 基于 Arbores 知识库执行 Scheme 代码，执行时允许创建或修改 Arbores 中的 S-Expression
**API 权限：** T1, T2
**输入：** 待执行的 S-Expression
**输出：**
  * 成功：返回包含执行结果和新版本 ID 的结果
    ```scheme
    '(("result" . 执行结果)
      ("new-version-id" . 新版本ID或#f))
    ```
  * 失败：错误信息
**示例：**
```scheme
(arb:eval '(begin
                 (arb:create "(define pi 3.14159)" '() "圆周率常量" "constant" '("pi"))
                 (* pi 2)))
;; => '(("result" . 6.28318)
;;       ("new-version-id" . 12349))

(arb:eval '(* 2 3))  ;; 不修改知识库的操作
;; => '(("result" . 6)
;;       ("new-version-id" . #f))
```


### Special Form
##### arb:ref [T2]
**功能：** 给定 ID 引用对应的 S-Expression
**语法：** `(arb:ref ((本地名 ID) ...) body)`
**输入：** 输入一组本地名到 ID 的绑定，和一个 body S-Expression
**输出：** 用 Arbores 中的 ID 替换 body 中的本地名得到的 S-Expression
**示例：**
```scheme
(arb:ref ((quicksort 123) (helper 456))
         (define (my-sort lst) (quicksort (helper lst))))
;; => '(define (my-sort lst) 
;;       (arb:get-code 123) (arb:get-code 456))
;; 实际执行时会替换为具体的代码
```

##### arb:transaction [T1]
**功能：** 在事务中执行一组修改操作，要么全部成功，要么全部回滚
**语法：** `(arb:transaction body...)`
**行为：** 
  * 在事务开始时创建版本快照
  * 顺序执行 body 中的所有表达式
  * 如果所有操作成功，提交事务并返回新版本ID
  * 如果任何操作失败，回滚到事务开始时的版本状态
**输出：**
  * 成功：返回事务结果和新版本 ID
    ```scheme
    '(("result" . 最后一个表达式的结果)
      ("new-version-id" . 新版本ID))
    ```
  * 失败：返回错误信息和回滚状态
    ```scheme
    '(("error" . 错误类型)
      ("message" . 错误描述)
      ("failed-at" . 失败的表达式索引)
      ("rollback-version" . 回滚后的版本ID))
    ```
**示例：**
```scheme
(arb:transaction
  (arb:create "(define (add x y) (+ x y))" '() "加法函数" "function" '("add"))
  (arb:create "(define (sub x y) (- x y))" '() "减法函数" "function" '("sub"))
  (arb:update 100 '(("dependencies" . (12350 12351)))))
;; => '(("result" . 更新结果)
;;       ("new-version-id" . 12352))

;; 失败示例（假设第二个操作失败）
(arb:transaction
  (arb:create "(define valid-func)" '() "有效函数" "function" '())
  (arb:create "invalid syntax(" '() "无效函数" "function" '())
  (arb:update 200 '(("code" . "new code"))))
;; => '(("error" . "syntax-error")
;;       ("message" . "第二个表达式语法错误")
;;       ("failed-at" . 1)
;;       ("rollback-version" . 12345)) ;; 回滚到事务开始前的版本
```