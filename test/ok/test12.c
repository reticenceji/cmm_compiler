int and(int a,int b){
    return a&b;
}
int or(int a,int b){
    return a|b;
}

int xor(int a,int b){
    return a^b;
}

int main(){
    int x;
    int y;
    /*
        3
        15
        12
    */
    x= 15;
    y= 3;
    output(and(x,y));
    output(or(x,y));
    output(xor(x,y));
    x= 9;
    y=2;
    /*
        0
        11
        11
    */
    output(and(x,y));
    output(or(x,y));
    output(xor(x,y));
    return 0;
}