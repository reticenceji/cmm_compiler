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

int mul(int a,int b){
    /*
        mul
    */
   return a*b;
}

int div(int a,int b){
    /*
        div
    */
   return a/b;
}

int mod(int a,int b){
    /*
        mod
    */
   return a % b;
}

int leftshift(int a,int b){
    /*
        <<
    */
   return a<<b;
}

int rightshift(int a,int b){
    /*
        >>
    */
   return a>>b;
}

int main(){
    int x;
    int y;
    x = 114514;
    y = 5;
    /*
        114519
        114509
        572570
        4
        22902
        3664448
        3578
    */
    output(add(x,y));
    output(sub(x,y));
    output(mul(x,y));
    output(mod(x,y));
    output(div(x,y));
    output(leftshift(x,y));
    output(rightshift(x,y));
    return 0;
}