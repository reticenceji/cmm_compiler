# 测试代码

## 词法分析相关

**正确情况**

ID为字母的组合，num为数字的组合，注释为`/*...*/`的形式

```c
void main(void){
    int abc;
    int def[10];
    /*
    This is a test code
    
    And this is also a test code
    */
}
```

语句间的空白字符

```c
void main(void){
    int abc ;int def[10] ;
    /*
    This is a test code
    
    And this is also a test code
    */
    abc = 20;
    def[5]=7;
}
```

**错误情况**

1. ID中包含数字

   ```c
   void main(void){
       int a1;
       int 1b;
   }
   ```

   预期情况：

   

2. ID中包含非字母或数字的情况

   ```c
   void main(void){
       int a_;
       int b[;
   }
   ```

   预期情况：

   

3. 程序中出现未定义的字符

   ```c
   void main(void){
       int a;
       int a = b ^ 6;
       int c;
       int c = d \2;
   }
   ```

   预期情况：

   

4. 采用`\\`注释的形式

   ```c
   void main(void){
       int a=20;//I thought this is a comment
   }
   ```

   预期情况：

   

5. num中出现非数字的字符

   ```c
   void main(void){
       int a;
       a = -2!0;
   }
   ```

   预期情况：

   

6. 关键字写错

   ```c
   void main(void){
       intt a;
       return_ ;
   }
   ```

   预期情况：



## 语法分析相关

**正确情况**

函数返回类型为void

```c
void main(void){
    return ;
}
```

函数返回类型为int

```c
int main(void){
    return 0;
}
```

多个函数

```c
int add(int a,int b){
    return a+b;
    /*
    	add
    */
}
int sub(int a,int b){
    /*
    	sub
    */
    return a-b;
}
int main(void){
    int x;
    int y;
    x = 10;
    y = 53;
    output(add(x,y)+sub(x-y));
    return 0;
}
```

全局变量和局部变量的声明和赋值

```c
int a;
int gcd(int a,int b){
    if(b==0) return a;
    else return gcd(b,a-a/b*b);
    /*a-a/b*b == a mod b*/
}
int add(int a,int b){
    return a+b;
    /*
    	add
    */
}
int b[20];
int main(void){
    a=10;
    int c;
    c=6;
}
```

if表达式

```c
int gcd(int u, int v)
{
    u = 1;
    if(v == 0)
        return u;
    else
        return gcd(u, u-u/v*v);
}
void main(void)
{
    int x;
    int y;
    x = input();
    y = input();
    output(gcd(x, y));
}
```

array、while

```c
int array(int a[], int n)
{
    int sum;
    int i;
    sum = 0;
    i = 0;
    while(i < n)
    {
        sum = sum + a[i];
    }
    return sum;
}
void main(void){
    int a[10];
    int b;
    array(a,b);
}
```



**错误情况**

1. 函数定义参数列表为空

   ```c
   void main(){
       return ;
   }
   ```

   预期情况：

   

2. 给变量赋值负数

   ```c
   void main(void){
       int a;
       a= -10;
   }
   ```

   预期情况：

   

3. 预算符未按正常位置排放

   ```c
   void main(){
       int a;
       a= +2*(5/2-1-)
   }
   ```

   预期情况：

   

4. 缺少分号

   ```c
   void main(void)
   {
       int x
       int y
   }
   ```

   预期情况：

   

5. if语句格式不符

   ```c
   int gcd(int u, int v)
   {
       u = 1;
       if(v == 0)
           return u;
       else
           return gcd(u, u-u/v*v);
   }
   void main(void)
   {
       int x;
       int y;
       x = input();
       y = input();
       if(){
           y=y+1;
       }
       else{
        	y=y-1;   
       }
       if(x){
           
       }
       else{
           
       }
       else{
           
       }
   }
   ```

   预期情况：

   

6. while语句格式不符

   ```c
   int array(int a[], int n)
   {
       int sum;
       int i;
       sum = 0;
       i = 0;
       while()
       {
           sum = sum + a[i];
       }
       return sum;
   }
   void main(void){
       int a[10];
       int b;
       array(a,b);
   }
   ```

   预期情况：

   

7. 括号未匹配

   ```c
   void main(void){
       int a[10;
   }
   ```

   预期情况：

   

5. 

## 语义分析相关

变量未定义

```c
int inc(void)
{
    int i;
    i = i + 1;
    return j;
}
void main(void){
    
}
```



变量定义错误

```c
int inc(void)
{
    void i;
    i = i + 1;
    return i;
}
void main(void){
    
}
```





变量重定义

```c
int inc(void)
{
    int i;
    int i[5];
    i = i + 1;
    return i;
}
void main(void){
    
}
```





函数重定义

```c
int inc(void)
{
    int i;
    i = i + 1;
    return i;
}

int inc(int i)
{
    return i + 1;
}
void main(void){
    
}
```





return类型不匹配

```c
void inc(int i)
{
    i = i + 1;
    return i;
}
void main(void){
    
}
```



```c
int inc(int i)
{
    i = i + 1;
    return;
}
void main(void){
    
}
```





非法使用函数调用

```c
int inc(int i)
{
    int a[10];
    return a(5);
}
void main(void){
    
}
```





参数不匹配

```c
int inc(void)
{
    int i;
    i = i + 1;
    return i;
}

int main(void)
{
    int x;
    x = inc(x);
    return 0;
}
```





赋值号两边参数不匹配

```c
int inc(void)
{
    int i;
    i = i + 1;
    return i;
}

int main(void)
{
    int x[5];
    x = inc(x);
    return 0;
}
```





```c
void inc(void)
{
    int i;
    i = i + 1;
    return;
}

int main(void)
{
    int x[5];
    x[0] = 5;
    return 0;
}
```



```c
int main(void)
{
    int i;
    i = 5;
    return i;
}
```

