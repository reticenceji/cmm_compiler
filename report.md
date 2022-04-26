# Cmm编译器设计报告

季高强 吴逸飞 高晨熙

## 序言

Cmm(C minus minus)编译器是《Compiler Construction: Principles and Practice》书附录介绍的一个精简版的C语言的实现，在书附录的基础上，添加了一些额外的特性。实现的功能包括：

1. 全局变量和局部变量的声明功能，函数定义功能。
2. 逻辑运算，位移运算，四则运算和取模运算，比较运算，赋值运算的支持。
3. 作用域的管理，允许内部作用域的同名变量临时覆盖外部作用域的同名变量。
4. 类型检查，并在特定条件下进行隐式类型转换。
5. 给出编译过程中，具体的报错产生原因。
6. 语法树的可视化。
7. 

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

季高强：词法分析，语法分析，语义分析和代码生成

吴逸飞：语法树的可视化，后端代码优化

高晨熙：编译器的综合测试

## 词法分析

词法分析使用正则表达式进行变量、字面量、注释和关键字和运算符的匹配。关键字和运算符是字符串的匹配，没有特别的地方。

注释和空白符的匹配规则

```
WHITESPACE = _{ " " | "\n" | "\r" }
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
pub enum AST {
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
fn gen_global_variable(&mut self, type_: &Type, name: &str) -> Result<()> {
    if self.global_variables.contains_key(name) || self.global_functions.contains_key(name) {
        Err(CodeGenErr::VariableRedefinition)?
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
                Err(CodeGenErr::ExpressionVoidType)?
            }
        }
        ...
    }
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

| 功能               | 对应的实现函数名      | 实现说明                                                     |
| ------------------ | --------------------- | ------------------------------------------------------------ |
| 生成函数           | `gen_function`        | 1. 调用LLVM的接口声明函数<br />2. 将函数参数放进作用域栈<br />3. 调用`gen_block_stmt`生成函数体<br />4. 将函数放入函数表 |
| 生成全局变量       | `gen_global_variable` | 1. 开辟空间存放变量<br />2. 设置初始值0<br />3. 将变量放入全局变量表 |
| 生成语句           | `gen_statement`       | 1. 语句可能是块语句，那么调用`gen_statement`<br />2. 语句可能是选择语句，我们构造三个BasicBlock和跳转语句，当条件满足的时候我们跳到IF_Block，条件不满足跳到ELSE_Block，最后都跳到DEST_Block。条件是表达式，如果表达式不为0代表条件满足。<br />3. 语句可能是循环语句，我们构造三个BasicBlock和跳转语句。第一个HEAD_Block检测条件，条件满足跳到LOOP_Block，不满足跳到DEST_Block。LOOP_Block执行循环体内代码，完成后调回HEAD_Block。<br />4. 语句可能是返回语句。构造return语句离开函数。<br />5. 语句可能是表达式，则调用对应的生成表达式的函数 |
| 生成块语句         | `gen_block_stmt`      | 1. 遍历变量声明，将变量放入作用域栈<br />2. 遍历子语句，调用`gen_statement`生成子语句 |
| 生成表达式         | `gen_expression`      | 根据表达式的类型调用下面生成表达式的函数                     |
| 生成运算表达式     | `gen_binary_expr`     | 1. 检查操作数的类型，如果不匹配则进行隐式转换<br />2. 根据运算符，生成对应的运算的代码 |
| 生成函数调用表达式 | `gen_function_call`   | 1. 准备参数，参数是表达式，调用`gen_expression`<br />2. 构造函数调用语句<br />3. 检查函数的返回值类型是否匹配 |
| 生成赋值表达式     | `gen_assignment_expr` | 1. 查变量表找到变量<br />2. 将右值赋值给变量，右值是表达式,调用`gen_expression` |

## 代码优化

## 测试案例

