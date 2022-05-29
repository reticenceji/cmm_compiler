# Cmm 编译器设计报告

> 季高强 吴逸飞 高晨熙

[TOC]

## 序言

Cmm(C minus minus) 编译器是《Compiler Construction: Principles and Practice》书附录介绍的一个精简版的 C 语言的实现，在书附录的基础上，添加了一些额外的特性。实现的功能包括：

1. 全局变量和局部变量的声明功能，函数定义功能。
2. 逻辑运算，位移运算，四则运算和取模运算，比较运算，赋值运算的支持。
3. 作用域的管理，允许内部作用域的同名变量临时覆盖外部作用域的同名变量。
4. 类型检查，并在特定条件下进行隐式类型转换。
5. 给出编译过程中，具体的报错产生原因和代码位置。
6. 语法树的可视化。
7. 生成代码优化。

### 代码结构说明

主要包括下面的代码：

```
├── src
│   ├── ast_viz.rs    ; 语法树可视化
│   ├── codegen.rs    ; 语义分析，代码生成
│   ├── grammar.pest  ; 词法和语法的定义
│   ├── io.c          ; input和output函数
│   ├── main.rs       ; 主程序
│   └── parser.rs     ; 词法分析和语法分析
└── test              ; 集成测试
    ├── algorithm
    ├── ok
    ├── with_output
    └── wrong
```

### 分工情况

季高强：词法分析，语法分析，语义分析和代码生成 分数37%

吴逸飞：语法树的可视化，后端代码优化 分数33%

高晨熙：编译器的综合测试 分数30%

## 词法分析

词法分析使用正则表达式进行变量、字面量、注释和关键字和运算符的匹配。关键字和运算符是字符串的匹配，没有特别的地方。不过，我们使用Pest工具进行词法的分析，他让我们将词法分析和语法分析集成。

注释和空白符的匹配规则

```
WHITESPACE = _{ " " | "\n" | "\r" | "\t"}
COMMENT = _{
    ("/*" ~ (!"*/" ~ ANY)* ~ "*/") // Block comment
    | ("//" ~ (!"\n" ~ ANY)* ~ ("\n" | EOI)) // Line comment
}
```

变量名的匹配，即以`_`或字母开头的字符串，同时不能是关键字。

```
id = @{!keyword ~ ((ASCII_ALPHA | "_") ~ (ASCII_ALPHA | "_" | ASCII_DIGIT)*)}
```

字面量，其实就是数字的匹配。支持0b开头的二进制数，0o开头的八进制数，0x开头的十六进制数和普通的十进制数。

```
int_literal = { bin_literal | hex_literal | oct_literal | dec_literal }
dec_literal = @{ "0" | ASCII_NONZERO_DIGIT ~ ASCII_DIGIT* }
bin_literal = @{ ^"0b" ~ ASCII_BIN_DIGIT+ }
oct_literal = @{ ^"0o" ~ ASCII_OCT_DIGIT+ }
hex_literal = @{ ^"0x" ~ ASCII_HEX_DIGIT+ }
```

## 语法分析

语法分析使用[Parsing expression grammars](https://pest.rs/book/grammars/peg.html)，简称PEG，来对词法进行描述。PEG在2004年由Bryan Ford）推出，它与20世纪70年代初引入的自顶向下的语法分析语言家族密切相关。在语法上，PEG很接近上下文无关文法（CFG）。代码使用的PEG如下所示

```
program = {SOI ~ (var_declaration | func_declaration)* ~ EOI}

var_declaration = {(type_spec ~ id ~ ("[" ~ int_literal ~ "]")? ~ ";")} 

func_declaration = {type_spec ~ id ~ params ~ block_stmt}
    params = {"(" ~ param? ~ ("," ~ param)*  ~ ")"}
    param = {type_spec ~ id ~ pointer?}
    pointer = @{"[" ~ "]"}

block_stmt = {"{" ~ (var_declaration *) ~ (statement *) ~ "}"}        
statement = {expression_stmt | selection_stmt | iteration_stmt | return_stmt | block_stmt}
    expression_stmt = {(expression ~ ";") | ";"}
    selection_stmt = {("if" ~ "(" ~ expression ~ ")" ~ statement ~ ("else" ~ statement)?)}
    iteration_stmt = {"while" ~ "(" ~ expression ~ ")" ~ statement}
    return_stmt = {("return" ~ ";") | ("return" ~ expression ~ ";")}
        
expression = {assignment_expr | logic_or_expr}

assignment_expr = {var ~ assign_simple ~ expression}
unary_expr = {bracket_expr | call_expr | var | int_literal}
multiplicative_expr = {unary_expr ~ ((op_mul | op_div| op_mod) ~ unary_expr)*}
additive_expr = {multiplicative_expr ~ ((op_add | op_sub) ~ multiplicative_expr)*}
shift_expr = {additive_expr ~ ((op_lshift | op_rshift) ~ additive_expr)*}
cmp_expr = {shift_expr ~ ((op_ge | op_le | op_gt | op_lt)  ~ shift_expr)*}
equlity_expr = {cmp_expr ~ ((op_eq | op_ne) ~ cmp_expr)*}
bit_and_expr = {equlity_expr ~ ((op_bit_and) ~ equlity_expr)*}
bit_xor_expr = {bit_and_expr ~ ((op_bit_xor) ~ bit_and_expr)*}
bit_or_expr = {bit_xor_expr ~ ((op_bit_or) ~ bit_xor_expr)*}
logic_and_expr = {bit_or_expr ~ ((op_and) ~ bit_or_expr)*}
logic_or_expr = {logic_and_expr ~ ((op_or) ~ logic_and_expr)*}

var = {id ~ ("[" ~ expression ~ "]")?}
bracket_expr = {"(" ~ expression ~")"}
call_expr = {id ~ "(" ~ args ~ ")"}
args = {expression? ~("," ~ expression)*}
```

采用[Pest](https://pest.rs/)来完成语法分析。Pest 是一个用 Rust 编写的通用解析器，专注于可访问性、正确性和性能。通过Pest会根据PEG生成对应的解析代码，然后对C源代码进行解析。解析的结果就是一个parsing tree。

Pest用`Pair`表示parsing tree的节点。每个节点的类型可以通过`Pair::as_rule()`进行访问。通过`Pair::into_inner()`我们可以获得节点的所有子节点。

通过对parsing tree的进一步解析和精简，我们生成abstract syntax tree。我们将程序的语法树定义为`Vec<AST>`类型。AST的每个节点类型在下面列出。利用Rust语言特殊的`Enum`类型，我们可以将每个节点的属性及子节点和类型绑定在一起。例如，`FunctionDec`代表函数节点，函数的属性包括函数的返回值类型、函数名、函数的参数列表，子节点就是函数体。

```rust
pub enum ASTInfo {
    FunctionDec(Type, String, Vec<(Type, String)>, Box<AST>),
    VariableDec(Type, String),

    BlockStmt(Vec<AST>, Vec<AST>),
    SelectionStmt(Box<AST>, Box<AST>, Option<Box<AST>>),
    IterationStmt(Box<AST>, Box<AST>),
    ReturnStmt(Option<Box<AST>>),

    AssignmentExpr(Box<AST>, Box<AST>),
    BinaryExpr(Oprand, Box<AST>, Box<AST>),
    CallExpr(String, Vec<AST>),

    Variable(String, Option<Box<AST>>),
    IntLiteral(i32),
}
```

我们在解析二元表达式的时候，为了让文法体现出运算符的优先级避免二义性，我们的parsing tree会变得非常的深。在创建语法树的时候，就可以大大减小这个深度。parsing tree的一些节点信息，也可以直接作为AST的节点的属性进行存储。

## 语义分析

语义分析和代码生成的实现我们放在了一起。后端代码生成我们使用了[LLVM](https://llvm.org/)的Rust binding [inkwell](https://thedan64.github.io/inkwell/inkwell/index.html)。

语义分析的主要内容包括

1. 变量检查。检查变量在使用的时候是否已经被声明，检查是否在同一个作用域内声明了相同名字的变量，检查是否有全局变量和函数的名称相同。
2. 类型检查。由于C语言会进行隐式转换，所以我们的类型检查主要是检查函数返回值的类型是否和return的类型匹配，以及void函数是否被用在了应该使用表达式的地方。

实现的方式，就是在代码生成阶段，扫描语法树的时候先进行检查。例如，在往全局变量表添加变量的时候并生成相关代码的时候，检查是否已经存在相同的全局变量和函数。

```rust
fn gen_global_variable(...) -> Result<()> {
    if self.global_variables.contains_key(name) || self.global_functions.contains_key(name) {
        Err(Error::new(position, ErrorType::VariableRedefinition))?
    }
    ...
}
fn gen_function(...) -> Result<()> {
    if self.global_variables.contains_key(name) || self.global_functions.contains_key(name) {
        Err(Error::new(position, ErrorType::FunctionRedefinition))?
    }
    ...
}
```

再比如，检查我们需要一个expression的时候，我们的函数调用是否返回的是void。

```rust
fn gen_expression(&self, ast: &AST) -> Result<(Type, BasicValueEnum)> {
    match ast {
        AST::CallExpr(name, argments) => {
            let r = self.gen_function_call(name, argments);
            if r.is_ok() && r.as_ref().unwrap().0 == Type::Void {
                Err(Error::new(ast.position, ErrorType::ExpressionVoidType))?
            }
        }
        ...
    }
```

以及检测函数的返回值的类型是否匹配，赋值语句左右两边的类型是否匹配

```rust
match ret_value {
    Some(ast) => {
        let (type_, value) = self.gen_expression(ast)?;
        if type_ == func_return_type {
            self.builder.build_return(Some(&value));
        } else {
            Err(Error::new(ast.position, ErrorType::MismatchedTypeFunction))?
        }
    }
    None => {
        if func_return_type == Type::Void {
            self.builder.build_return(None);
        } else {
            Err(Error::new(stmt.position, ErrorType::MismatchedTypeFunction))?
        }
    }
}

if type_left == type_right {
    self.builder.build_store(ptr, value);
    Ok((type_left, value.as_basic_value_enum()))
} else {
    Err(Error::new(var.position, ErrorType::MismatchedType))?
}
```

编译器应该给出合适的报错信息，让用户可以修改源代码中的错误。我们将错误分成两种类型，一个是词法/语法分析时候发现的错误，一个是语义检查发现的错误。

对于词法/语法分析发现的错误，使用的Pest已经提供了较好的支持，我们可以直接利用他提供的错误信息进行一定的处理返回给用户。

对于语义检查发现的错误，我们分析错误的原因，利用保存在语法树中的位置信息，向用户报告错误的原因和位置。我们定义了以下错误类型

```rust
pub enum ErrorType {
    VariableRedefinition,    // 变量重定义
    IndexNotInt,             // 数组的索引不是整数类型
    VariableNotDefined,      // 变量未定义
    FunctionRedefinition,    // 函数重定义
    MismatchedType,          // 表达式的类型不匹配
    MismatchedTypeFunction,  // 函数的返回值不匹配
    FunctionNotDefined,      // 函数未定义
    ExpressionVoidType,      // 在需要表达式的地方使用了返回值为void的函数
    PestError(String),       // 词法分析错误和语法分析错误
}
```

通过在语法树中存储的行数信息，我们可以给出具体的错误发生位置，帮助用户精确定位错误。一个例子如下：

```
Error: 3:9:  --> 3:9
  |
3 |     int 1b;␊
  |         ^---
  |
  = expected id
```

## 代码生成

代码生成的过程是遍历语法树以产生LLVM-IR，然后再将LLVM-IR翻译成特定的机器代码。代码生成用到的数据如下，前三个是LLVM的一些数据结构，`global_variables`是全局变量表，`global_functions`是函数表，`variables_stack`可以认为是一个作用域栈，每个作用域都有自己的变量表，栈顶的作用域是优先的，代表嵌条的`{}`中内层是优先的。`current_function`记录当前正在生成代码的函数。

```rust
pub struct CodeBuilder<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,

        global_variables: HashMap<String, (Type, PointerValue<'ctx>)>,
        global_functions: HashMap<String, (Type, FunctionValue<'ctx>)>,
        variables_stack: Vec<HashMap<String, (Type, PointerValue<'ctx>)>>,
        current_function: Option<(Type, FunctionValue<'ctx>)>,
}
```

| 功能               | 对应的实现函数名      | 实现说明                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| ------------------ | --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 生成函数           | `gen_function`        | 1. 调用LLVM的接口声明函数<br />2. 将函数参数放进作用域栈<br />3. 调用`gen_block_stmt`生成函数体<br />4. 将函数放入函数表                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 生成全局变量       | `gen_global_variable` | 1. 开辟空间存放变量<br />2. 设置初始值0<br />3. 将变量放入全局变量表                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 生成语句           | `gen_statement`       | 1. 语句可能是块语句，那么调用`gen_statement`<br />2. 语句可能是选择语句，我们构造三个BasicBlock和跳转语句，当条件满足的时候我们跳到IF_Block，条件不满足跳到ELSE_Block，最后都跳到DEST_Block。条件是表达式，如果表达式不为0代表条件满足。<br />3. 语句可能是循环语句，我们构造三个BasicBlock和跳转语句。第一个HEAD_Block检测条件，条件满足跳到LOOP_Block，不满足跳到DEST_Block。LOOP_Block执行循环体内代码，完成后调回HEAD_Block。<br />4. 语句可能是返回语句。构造return语句离开函数。<br />5. 语句可能是表达式，则调用对应的生成表达式的函数 |
| 生成块语句         | `gen_block_stmt`      | 1. 遍历变量声明，将变量放入作用域栈<br />2. 遍历子语句，调用`gen_statement`生成子语句                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| 生成表达式         | `gen_expression`      | 根据表达式的类型调用下面生成表达式的函数                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 生成运算表达式     | `gen_binary_expr`     | 1. 检查操作数的类型，如果不匹配则进行隐式转换<br />2. 根据运算符，生成对应的运算的代码                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| 生成函数调用表达式 | `gen_function_call`   | 1. 准备参数，参数是表达式，调用`gen_expression`<br />2. 构造函数调用语句<br />3. 检查函数的返回值类型是否匹配                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| 生成赋值表达式     | `gen_assignment_expr` | 1. 查变量表找到变量<br />2. 将右值赋值给变量，右值是表达式,调用`gen_expression`                                                                                                                                                                                                                                                                                                                                                                                                                                                               |

## 代码优化

代码优化考虑使用 LLVM 的 [Pass](https://llvm.org/docs/Passes.html) 进行优化，这里采用的优化是基于函数的优化，即优化的单位是函数而不是整个文件程序。

优化采用的关键结构为：

```rust
pub struct CodeBuilder<'ctx> {
    ...
    /// For optimize
    fpm: Option<PassManager<FunctionValue<'ctx>>>,
}
```

当没有启用优化时，将 `fpm` 初始化为 `None` 即可，否则我们这样初始化：

```rust
// Create FPM
let mut fpm = None;
if opt {
    let temp = PassManager::create(&module);

    temp.add_instruction_combining_pass();
    temp.add_reassociate_pass();
    temp.add_gvn_pass();
    temp.add_cfg_simplification_pass();
    temp.add_promote_memory_to_register_pass();

    temp.initialize();
    fpm = Some(temp)
}
```

当完成一个函数的生成后，调用 `fpm` 对函数进行优化：

```rust
// Optimize on function level
if let Some(fpm) = &self.fpm {
  fpm.run_on(&function);
}
```

启用的优化及其描述如下表：

| Name                           | Description                                | Code Example                                                               |
| ------------------------------ | ------------------------------------------ | -------------------------------------------------------------------------- |
| Combine Redundant Instructions | 组合指令以形成更少、更简单的指令           | %Y = add i32 %X, 1<br />%Z = add i32 %Y, 1<br />=><br />%Z = add i32 %X, 2 |
| Reassociate Expressions        | 重新关联表达式的顺序，以得到更好的常数传播 | 4 + (x + 5) ⇒ x + (4 + 5)                                                  |
| Global Value Numbering         | 对全局值计算进行编号，消除部分冗余指令     |                                                                            |
| Simplify CFG                   | 执行死代码消除和基本的块合并               |                                                                            |
| Promote Memory to Register     | 即将内存的引用转换到寄存器中               |                                                                            |

一些具体的例子如下：

**Combine Redundant Instructions**

```c
int f(int x) {
    return (1+2+x)*(x+(1+2));
}
```

生成的 asm 如下：

```assembly
# No Optimization
_f:
    .cfi_startproc
    movl %edi, -4(%rsp)
    movl -4(%rsp), %eax
    addl $3, %eax
    movl -4(%rsp), %ecx
    addl $3, %ecx
    imull %ecx, %eax
    retq
    .cfi_endproc

# With Optimization
_f:
    .cfi_startproc
    movl %edi, %eax
    addl $3, %eax
    imull %eax, %eax
    retq
    .cfi_endproc
```

显然，两次对 `1+2+x` 的计算被优化为了一次。

**Reassociate Expressions**

```c
int f(int x) {
    return 4 + (x + 5) + 8;
}
```

生成的 asm 如下：

```assembly
# No Optimization
_f:
    .cfi_startproc
    movl %edi, -4(%rsp)
    movl -4(%rsp), %eax
    addl $5, %eax
    addl $4, %eax
    addl $8, %eax
    retq
    .cfi_endproc
# With Optimization
_f:
    .cfi_startproc
    movl %edi, %eax
    addl $17, %eax
    retq
    .cfi_endproc
```

多次对常数运算被优化为一次常数加法运算。

**Global Value Numbering**

```c
int f(int a, int b) {
    int c;
    int d;
    c = a + b;
    d = a + b;
}
```

这个优化的现象通过 lr 代码更加清晰：

```
# No Optimization
define i32 @f(i32 %a, i32 %b) {
entry:
  %0 = alloca i32, align 4
  store i32 %a, i32* %0, align 4
  %1 = alloca i32, align 4
  store i32 %b, i32* %1, align 4
  %c = alloca i32, align 4
  %d = alloca i32, align 4
  %2 = load i32, i32* %0, align 4
  %3 = load i32, i32* %1, align 4
  %4 = add i32 %2, %3
  store i32 %4, i32* %c, align 4
  %5 = load i32, i32* %0, align 4
  %6 = load i32, i32* %1, align 4
  %7 = add i32 %5, %6
  store i32 %7, i32* %d, align 4
  ret void
}

# With Optimization
define i32 @f(i32 %a, i32 %b) {
entry:
  %0 = alloca i32, align 4
  store i32 %a, i32* %0, align 4
  %1 = alloca i32, align 4
  store i32 %b, i32* %1, align 4
  %c = alloca i32, align 4
  %d = alloca i32, align 4
  %2 = add i32 %a, %b
  store i32 %2, i32* %c, align 4
  store i32 %2, i32* %d, align 4
  ret void
}
```

仔细观察，在未优化的代码中：

```
%4 = add i32 %2, %3
%7 = add i32 %5, %6
```

即 `c=a+b;d=a+b` 被计算了两次，而在优化后的代码中：

```
%2 = add i32 %a, %b
store i32 %2, i32* %c, align 4
store i32 %2, i32* %d, align 4
```

编译器发现 `a+b` 在 `c` 中已经计算过了，所以算 `d` 的时候就不会再次计算了。

**Simplify CFG**

```c
int f() {
    int a;
  
    if (0) {
        a = 1;
    } else {
        a = 2;
    }

    return a;
}
```

生成的 asm 如下：

```assembly
# No Optimization
_f:
    .cfi_startproc
    xorl %eax, %eax
    testb $1, %al
    jne LBB0_1
    jmp LBB0_2
LBB0_1:
    movl $1, -4(%rsp)
    jmp LBB0_3
LBB0_2:
    movl $2, -4(%rsp)
LBB0_3:
    movl -4(%rsp), %eax
    retq
    .cfi_endproc

# With Optimization
_f:
    .cfi_startproc
    movl $2, %eax
    retq
    .cfi_endproc
```

这个优化也比较明显，分析出代码不会执行 `if` 分支，所以直接删除了那一段，而直接执行了 `a=2` 并返回。（其他优化过程优化掉了 `a=2` 的计算）

**Promote Memory to Register**

```c
int f() {
    int a [3];

    a[0] = 0;
    a[1] = 1;
    a[2] = 3;
}
```

生成的 asm 如下：

```assembly
# No Optimization
_f:
    .cfi_startproc
    leaq -12(%rsp), %rax
    movq %rax, -24(%rsp)
    movq -24(%rsp), %rax
    movl $0, (%rax)
    movq -24(%rsp), %rax
    movl $1, 4(%rax)
    movq -24(%rsp), %rax
    movl $3, 8(%rax)
    retq
    .cfi_endproc
    
# With Optimization
_f:
    .cfi_startproc
    movl $0, -12(%rsp)
    movl $1, -8(%rsp)
    movl $3, -4(%rsp)
    retq
    .cfi_endproc
```

可以看到，重复的内存访问都被简化到了 `sp` 寄存器的操作上。

## AST 可视化
AST 可视化采用的是 [Graphviz](https://graphviz.org/) ，其接受 dot 文件并将其渲染为指定格式的图片。

可视化的关键结构有：
```rust

#[derive(Debug)]
pub struct DiGraph {
    name: Option<String>,
    id: usize,
    conts: Vec<Content>,
}

#[derive(Debug)]
enum Node {
    Symbol(usize, String),
    Subgraph(DiGraph),
}

#[derive(Debug)]
struct Edge {
    from: usize,
    to: usize,
}

#[derive(Debug)]
enum Content {
    Node(Node),
    Edge(Edge),
}

```

由于 dot 文件格式的要求，每个节点都需要一个唯一的 ID 进行标记，因此每分配一个节点，都需要分配一个 ID，这一点可以通过一个 `IDAllocator` 实现（不是重点，就不予以展示了）。

结构解析：
- `DiGraph`：在我们的设计中，每一个非终结符都是一个 `DiGraph`，这个图是一个树形结构，根节点即这个非终结符。
  - `name` 是这个非终结符的名称，用以在图中展示。
  - `id` 即前文提到的 dot 文件格式的要求。
  - `conts` 是这个非终结符下的一系列内容（可以理解为树中根节点下面所有的边和节点）。
- `Node`：即一个单独的节点，是一个 `Enum` 类型
  - 可以是 `Symbol`，即单个的节点。
  - `Subgraph`，如其名，子图，是一系列节点的集合，也可以认为是子数。
  - 这样设计的目的，是方面通过递归的方式进行图形绘制。
- `Edge`：记录节点之间边，`from` 和 `to` 即两个节点的 ID 。
- `Content`：即一个 `DiGraph` 的内容。一棵树包括节点和边，所以 `Content` 也是这两种类型。

这里面包含很多递归的定义，主要是方便递归的生产 dot 文件，可能造成理解上的一些困难。

生成 dot 文件的核心代码定义为 `DiGraph` 的 `parse_ast` 方法：
```rust
fn parse_ast(&mut self, ast: &AST) {
    match &ast.info {
        ASTInfo::FunctionDec(ftype, name, params, box ast) => {}
        ASTInfo::VariableDec(vtype, name) => {}
        ASTInfo::BlockStmt(ast1, ast2) => {
            self.name = Some("BlockStmt".to_string());
            for ast in ast1 {
                let subg = DiGraph::from_ast(ast);
                let node = Node::Subgraph(subg);

                self.add_cont(Content::Edge(Edge::new(&self, &node)));
                self.add_cont(Content::Node(node));
            }

            for ast in ast2 {
                let subg = DiGraph::from_ast(ast);
                let node = Node::Subgraph(subg);

                self.add_cont(Content::Edge(Edge::new(&self, &node)));
                self.add_cont(Content::Node(node));
            }
        }
        ASTInfo::SelectionStmt(box ast1, box ast2, ast3) => {}
        ASTInfo::IterationStmt(box ast1, box ast2) => {}
        ASTInfo::ReturnStmt(ast) => {}
        ASTInfo::AssignmentExpr(box ast1, box ast2) => {}
        ASTInfo::BinaryExpr(oprand, box ast1, box ast2) => {}
        ASTInfo::CallExpr(name, params) => {}
        ASTInfo::Variable(name, ast) => {}
        ASTInfo::IntLiteral(val) => {}
    }
```

解析思路是显然的，由于 AST 是个 Enum 的类型，我们单独对每个类型进行解析，如果又遇到 AST 结构（也即一个非终结符），就递归的调用自己即可。
解析过程以 `BlockStmt` 为例：
- 由于 `BlockStmt` 是一个非终结符（所以这必须是一个 `DiGraph` 的方法），我们先设置自己的名称，即在图中展示的 Label。
- `ast1` 是一个 AST 的 Vec，对其中每一个 AST 结构，其本质就是一个非终结符，也即一个 `DiGraph` 结构。
- 所以我们新建一个这样的结构，`let subg = DiGraph::from_ast(ast);`（`from_ast` 本质上就是 `parse_ast` 的 Wrapper，方便使用罢了），让其先解析好自己。
- 同是这个 `subg` 又是当前 `DiGraph` 的一个子节点（或者说子树），所以将其加入到当前的 `conts` 中，并建立一条 `self` 到 `subg` 的边。

如此就完成了对 `BlockStmt` 的解析，其他节点都是类似的，思想不变。

为了生成 dot 文件，观察以下代码：
```rust
impl Node {
    pub fn to_dot(&self) -> String {
        match self {
            Self::Symbol(id, name) => {
                format!("node{} [ label = \" {} \" ];", id, name)
            }
            Self::Subgraph(subg) => subg.to_dot(),
        }
    }
}

impl Edge {
    pub fn to_dot(&self) -> String {
        format!("node{} -> node{}", self.from, self.to)
    }
}

impl Content {
    pub fn to_dot(&self) -> String {
        match self {
            Self::Node(node) => node.to_dot(),
            Self::Edge(edge) => edge.to_dot(),
        }
    }
}

impl DiGraph{
    fn to_dot(&self) -> String {
        let mut buf = format!(
            "node{} [ label = \" {} \" ];",
            self.id,
            self.name.as_ref().expect("Unformed DiGraph!")
        );
        for cont in &self.conts {
            buf.push_str(&cont.to_dot());
            buf.push_str("\n");
        }

        buf
    }
}

```

同样是递归的生成，其含义是显然的。

如此 ast 可视化就完成了：生成 dot 文件后，使用 `dot` 命令就可以生成相应的图了。

例如 [test.c](test/ok/test.c) 的可视化如下：

![](ast.png)

## 测试案例

### 词法测试

| 词法规则                                      | 正确样例                             | 典型错误                |
| --------------------------------------------- | ------------------------------------ | ----------------------- |
| ID为以`_`或字母开头的字符串，同时不能是关键字 | `int _abc1;int add(int a_,int_ b){}` | `int 1a_;int return;` |
| num为数字的组合(不包括-等符号)                | `a=10;`                              | `a=-10;`                |
| 支持二进制、八进制、十六进制和十进制数字      | `a=0b1;b=0o7;c=0xf;C=0xF;d=10;`      | `a=0o9;`                |
| 注释为`/*...*/`或`//`的形式                   | `//a comment` `/*a comment*/`        | `/*wrong*`              |
| 仅支持部分运算符`+-*/\><>=<=&|^<<>>`                               | `a=a&1;`                | `a=!1;` |
| 空格、tab、换行等空白字符会被忽略             | `int a     =10;`                     | 一般不会有              |
| 匹配`int void return`等关键字                 | `int a;return 0;`                    | `int return;`           |

### 语法测试

| 语法规则                                           | 正确样例                     | 典型错误              |
| -------------------------------------------------- | ---------------------------- | --------------------- |
| 定义语句只能为`Type ID;`的形式，不能在定义同时赋值 | `int a;int b[20];`           | `int a=50;`           |
| 局部变量的定义语句结束后不能在该函数中再次定义变量 | `int a;int b;a=10;b=20;`     | `int a;a=10;int b;`   |
| 括号((,[,{,},],))需匹配完整                        | `int add(int a[],int b[]){}` | 括号不匹配            |
| 数组使用`Type ID[num]`的形式表示                   | `int def[10];`               | `int def[10;`         |
| 没有函数声明，只有函数定义                         | `int add(int a[],int b[]){}` | `int add();`          |
| 函数定义时若无参数则不写，不应该使用void           | `int test(){}`               | `int test(void){}`    |
| 函数定义时需要写完整的参数名，不能只写类型         | `int add(int a[],int b[]){}` | `int add(int,int){}`  |
| 运算符使用时要符合规则                             | `a=(a+1)^3/2*5;`             | `a=+a;`               |
| 语句末需要有`;`                                    | `int a;`                     | `int a`               |
| `if`和`else`要匹配，不允许出现额外的else           | `if(a){}else if(b){} else{}` | `if(a){}else{}else{}` |
| `while`语句和`if else`语句要符合使用规则           | `while(a){}`                 | `if(){}`和`while(){}` |

### 语义测试

#### 编译时出现的典型错误

1. 变量使用前需要先定义

   ```c
   int inc()
   {
       int i;
       return j; //未定义的变量j
   }
   void main(){
       inc();
   }
   ```

2. 变量使用时需要符合变量定义的类型

   ```c
   void main(){
       void i;
       i = 1;//给void类型变量i赋值int类型的数字
   }
   ```

3. 变量定义时不能使用已定义的变量名

   ```c
   void main(){
       int i;
       int i[5];//定义了已定义的变量名i
   }
   ```

4. 运算符两边的运算数类型需要符合运算符的规则

   ```c
   void main(){
       int i;
       void j;
       i = j+1; //+运算符两边的运算数都应为int，使用了void类型的j
   }
   ```

5. 函数定义时不能使用已定义的函数名

   ```c
   int inc()
   {
       int i;
       i = i + 1;
       return i;
   }
   int inc(int i) //定义了已定义的函数名inc
   {
       return i + 1;
   }
   void main(){
       inc();
   }
   ```

6. 函数定义时的返回类型需要与`return`语句相匹配

   ```c
   void inc(int i)
   {
       i = i + 1;
       return i;//函数返回类型应为void，但返回了int
   }
   int add(int i){
       i = i + 1;
       return; //函数返回类型应为int，但没有返回值（void）
   }
   void main(){
       inc();
       add(1);
   }
   ```

7. 必须先定义函数，然后才能调用函数

   ```c
   void main(){
       int i;
       inc(i);//函数仍未定义
       return ;
   }
   int inc(int i){
       return i+1;
   }
   ```

8. 调用函数时的参数需要与函数定义时的参数类型相匹配

   ```c
   int inc(int i){
       return i+1;
   }
   void swap(int a,int b){
       int tmp;
       tmp=b;
       b=a;
       a=tmp;
   }
   void main(){
       int i;
       int a[10];
       int b[10];
       inc();//函数定义要求参数为int，但调用时未传入参数
       swap(a,b);//函数定义要求参数为两个int，但调用时传入两个int类型的数组
       return ;
   }
   ```

9. 文件中需要有`main()`作为入口

   ```c
   int inc(int i){
       return i+1;
   }
   //没有main()作为入口
   ```

10. 访问数组时下标不能越界，需要在定义时限定的范围内

    ```c
    int main(){
        int a[10];
        int b;
        b=a[10];//下标超过定义时界限
        return 0;
    }
    ```

11. 可以使用变量作为`int main()`的返回值

    ```c
    int main(){
        int i;
        i=1;
        return i;//可以使用变量作为返回值，但main()的非0返回，编译器会报错
    }
    ```

12. 函数执行到`return`语句时需要退出该函数（在归并排序测试中体现）

#### 运行正确性测验

##### 排序算法测试

根据C--的语法编写了冒泡排序、快速排序和归并排序的代码，经检验均能正确对输入排序后输出。

冒泡排序中测试了嵌套的while语句，函数参数传递数组和数组的读写功能

快速排序中测试了递归函数，函数参数传递数组和`if`语句功能

归并排序中测试了递归函数，函数参数传递数组，多个并置的`if-else`语句，函数的`return`语句的功能

##### 变量作用域测试

局部变量若与已定义的全局变量同名，在函数中会屏蔽该全局变量

```c
int a;//定义全局变量a
int gcd(int a, int b) {//定义递归函数gcd，在参数中定义了局部变量a和b，屏蔽了全局变量a
    if (b == 0)
        return a;
    else
        return gcd(b, a - a / b * b);
}
int b[20];//在下方定义全局变量数组b
int add(int a, int b) {//定义函数add()，在参数中定义了局部变量a和b，屏蔽了全局变量a和b
    return a + b;
}
int main() {
    int c;
    a = 10;//给全局变量赋值
    c = 6;//给局部变量赋值
    return 0;
}
```
