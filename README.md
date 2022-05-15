# C-

C-是《Compiler Construction: Principles and Practice》书后介绍的一个精简版的C语言。

构建：

```shell
cargo build --release
```

运行：

```shell
cmm <source file>
```

然后利用本地的编译器进行编译即可，如：

```
clang io.c test.s
```

> 本项目只实现到汇编代码的生成，从汇编到机器码是机械的翻译过程，并不予以实现。

## 词法分析&语法分析

使用 [PEST](https://pest.rs/) 提供的 [Parsing expression grammars](https://pest.rs/book/grammars/peg.html)，来对词法和语法进行描述。

## 语义分析

## 代码优化
使用 [PassManager](https://thedan64.github.io/inkwell/inkwell/passes/struct.PassManager.html)，基于函数进行优化。

优化包括：
- instruction combining
- reassociate
- GVN
- CFG simplification
- promote memory to register

如果要进行优化，使用 `--opt` 选项：
```shell
cmm test/opt/test1.c -opt
```

## 代码生成

使用 [LLVM](https://llvm.org/) 的Rust binding [inkwell](https://github.com/TheDan64/inkwell)。

## 测试

## AST 可视化
使用 [Graphviz](http://graphviz.org) 对 AST 进行可视化。

生成 dot 文件：
```shell
cmm <soruce file> --dotfile <dot file>
```

从 dot 文件生成 png 图片：
```shell
dot <dotfile> -T png -o dot.png
```

例如，生成 [test.c](test/ok/test.c) 的 ast 可视化文件：
```shell
cmm test/ok/test.c --dotfile ./dotfile
dot dotfile -T png -o ast.png
```
![](ast.png)

## TODO

- [x] `input`和 `output`函数。
- [x] 完整的测试。
- [x] 代码优化。
- [x] 现在不能有全局变量，需要修复。
- [x] 文档。
- [x] 语法树的可视化。
- [x] 执行环境。
