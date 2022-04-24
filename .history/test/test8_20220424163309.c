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
void main(){
    int a[10];
    int b;
    a[0]=0;
    a[1]=0;
    a[2]=0;
    a[3]=0;
    a[4]=0;
    a[5]=0;
    a[6]=0;
    a[7]=0;
    a[8]=0;
    a[9]=0;
    a[0]=0;
    array(a,b);
}