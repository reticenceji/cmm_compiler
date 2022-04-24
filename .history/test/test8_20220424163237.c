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
    array(a,b);
}