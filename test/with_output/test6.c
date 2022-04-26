int main(){
    int x;
    int y;
    int a;
    int b;
    x=0;
    y=0;

    /*
        && ||
        correctness
    */
    /*
        1
        1
        1
        1
    */
    if(x==0 && y==0){
        output(1);
    }
    else{
        output(0);
    }
    if(x==1 || y==1){
        output(0);
    }
    else{
        output(1);
    }
    if(x==0 || y==1){
        output(1);
    }
    else{
        output(0);
    }
    if(x==1 || y==0){
        output(1);
    }
    else{
        output(0);
    }
    /*
        0
        1
        2
        3
        4
    */
    a=0;
    b=0;
    while(x>=0 && y>=0 && a<5){
        output(a);
        a=a+1;
    }
    /*
        0
        1
        2
        3
        4
    */
    while(x<0 || y<0 || b<5){
        output(b);
        b=b+1;
    }
    /*
        && ||
        associate
    */
    /*
        114514
        0
    */
    if(0 && 0 || 1){
        output(114514);
    }
    else{
        output(0);
    }
    if(0 && (0 || 1)){
        output(114514);
    }
    else{
        output(0);
    }
    /*
        114514
        114514
    */
    if(1 || 0 && 0){
        output(114514);
    }
    else{
        output(0);
    }
    if(1 || (0 && 0)){
        output(114514);
    }
    else{
        output(0);
    }
    return 0;
}