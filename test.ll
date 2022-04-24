; ModuleID = 'test'
source_filename = "test"

declare i32 @input()

declare void @output(i32)

define i32 @gcd(i32 %u, i32 %v) {
entry:
  %u1 = alloca i32
  %v2 = alloca i32
  store i32 1, i32* %u1
  %0 = load i32, i32* %v2
  %1 = icmp eq i32 %0, 0
  br i1 %1, label %then_block, label %else_block

then_block:                                       ; preds = %entry
  br label %if_dest_block

else_block:                                       ; preds = %entry
  br label %if_dest_block

if_dest_block:                                    ; preds = %else_block, %then_block
  ret void
}

define void @main() {
entry:
  %x = alloca i32
  %y = alloca i32
  %input = call i32 @input()
  store i32 %input, i32* %x
  %input1 = call i32 @input()
  store i32 %input1, i32* %y
  %0 = load i32, i32* %x
  %1 = load i32, i32* %y
  %gcd = call i32 @gcd(i32 %0, i32 %1)
  call void @output(i32 %gcd)
  ret void
}
