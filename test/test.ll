; ModuleID = 'test/test.c'
source_filename = "test/test.c"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

declare i32 @input()

declare void @output(i32)

define i32 @main() {
entry:
  %a = alloca i32, align 4
  %b = alloca [10 x i32], align 4
  store i32 1, i32* %a, align 4
  %0 = getelementptr [10 x i32], [10 x i32]* %b, i32 0, i32 0
  store i32 0, i32* %0, align 4
  %1 = getelementptr [10 x i32], [10 x i32]* %b, i32 0, i32 1
  %2 = load i32, i32* %a, align 4
  store i32 %2, i32* %1, align 4
  %3 = getelementptr [10 x i32], [10 x i32]* %b, i32 0, i32 2
  %4 = load i32, i32* %a, align 4
  %5 = add i32 %4, 1
  store i32 %5, i32* %3, align 4
  %6 = getelementptr [10 x i32], [10 x i32]* %b, i32 0, i32 3
  %7 = load i32, i32* %a, align 4
  %8 = mul i32 %7, 3
  store i32 %8, i32* %6, align 4
  %9 = getelementptr [10 x i32], [10 x i32]* %b, i32 0, i32 4
  %10 = getelementptr [10 x i32], [10 x i32]* %b, i32 0, i32 3
  %11 = load i32, i32* %10, align 4
  %12 = add i32 %11, 1
  store i32 %12, i32* %9, align 4
  br label %loop_head

loop_head:                                        ; preds = %loop_body, %entry
  %13 = load i32, i32* %a, align 4
  %14 = icmp sge i32 %13, 0
  br i1 %14, label %loop_body, label %loop_dest_block

loop_body:                                        ; preds = %loop_head
  %15 = load i32, i32* %a, align 4
  %16 = sub i32 6, %15
  %17 = getelementptr [10 x i32], [10 x i32]* %b, i32 0, i32 %16
  %18 = load i32, i32* %a, align 4
  %19 = sub i32 6, %18
  store i32 %19, i32* %17, align 4
  %20 = load i32, i32* %a, align 4
  %21 = sub i32 %20, 1
  store i32 %21, i32* %a, align 4
  br label %loop_head

loop_dest_block:                                  ; preds = %loop_head
  %22 = getelementptr [10 x i32], [10 x i32]* %b, i32 0, i32 6
  %23 = load i32, i32* %22, align 4
  %24 = icmp eq i32 %23, 6
  br i1 %24, label %then_block, label %if_dest_block

then_block:                                       ; preds = %loop_dest_block
  %25 = getelementptr [10 x i32], [10 x i32]* %b, i32 0, i32 7
  store i32 7, i32* %25, align 4
  br label %if_dest_block

if_dest_block:                                    ; preds = %then_block, %loop_dest_block
  ret i32 1
}
