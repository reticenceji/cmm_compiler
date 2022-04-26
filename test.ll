; ModuleID = 'test'
source_filename = "test"

declare i32 @input()

declare void @output(i32)

define i32 @gcd(i32 %u, i32 %v) {
entry:
  %0 = alloca i32, align 4
  store i32 %u, i32* %0, align 4
  %1 = alloca i32, align 4
  store i32 %v, i32* %1, align 4
  %2 = load i32, i32* %1, align 4
  %3 = icmp eq i32 %2, 0
  %4 = zext i1 %3 to i32
  %condition = trunc i32 %4 to i1
  br i1 %condition, label %then_block, label %else_block

then_block:                                       ; preds = %entry
  %5 = load i32, i32* %0, align 4
  ret i32 %5

else_block:                                       ; preds = %entry
  %6 = load i32, i32* %1, align 4
  %7 = load i32, i32* %0, align 4
  %8 = load i32, i32* %0, align 4
  %9 = load i32, i32* %1, align 4
  %10 = sdiv i32 %8, %9
  %11 = load i32, i32* %1, align 4
  %12 = mul i32 %10, %11
  %13 = sub i32 %7, %12
  %gcd = call i32 @gcd(i32 %6, i32 %13)
  ret i32 %gcd

if_dest_block:                                    ; No predecessors!
  ret void
}

define void @main() {
entry:
  %x = alloca i32, align 4
  %y = alloca i32, align 4
  store i32 140, i32* %x, align 4
  store i32 49, i32* %y, align 4
  %0 = load i32, i32* %x, align 4
  %1 = load i32, i32* %y, align 4
  %gcd = call i32 @gcd(i32 %0, i32 %1)
  call void @output(i32 %gcd)
  ret void
}
