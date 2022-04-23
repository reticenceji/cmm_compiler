; ModuleID = 'test/test_ast.c'
source_filename = "test/test_ast.c"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

declare i32 @input()

declare void @output(i32)

define i32 @main() {
entry:
  %a = alloca i32, align 4
  store i32 1, i32* %a, align 4
  ret i32 1
}
