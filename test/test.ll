; ModuleID = 'test/test.c'
source_filename = "test/test.c"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

declare i32 @input()

declare void @output(i32)

define void @main() {
entry:
  %abc = alloca i32, align 4
  %def = alloca [10 x i32], align 4
  ret void
}
