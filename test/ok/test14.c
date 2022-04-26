int main(){
    /*
        if the first is true, the one after || will not be considered
        if the first is false, the one after && will not be considered
        if a-b == 0, a-b is false, else a-b is true 
    */
    int a,b;
    /*
        114514
        30
    */
    a=20;
    b=20;
    if(1 || a-b){
        output(114514);
    }
    else{
        output(30);
    }
    if(0 || a-b){
        output(114514);
    }
    else{
        output(30);
    }
    /*
        114514
        114514
    */
    a=25;
    b=20;
    if(1 || a-b){
        output(114514);
    }
    else{
        output(30);
    }
    if(0 || a-b){
        output(114514);
    }
    else{
        output(30);
    }
    /*
        30
        30
    */
    a=20;
    b=20;
    if(1 && a-b){
        output(114514);
    }
    else{
        output(30);
    }
    if(0 && a-b){
        output(114514);
    }
    else{
        output(30);
    }
    /*
        114514
        30
    */
    a=20;
    b=25;
    if(1 && a-b){
        output(114514);
    }
    else{
        output(30);
    }
    if(0 && a-b){
        output(114514);
    }
    else{
        output(30);
    }
    return 0;
}