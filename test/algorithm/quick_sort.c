int part(int a[],int left,int right){
    int tmp;
    int i;
    int j;
    int t;
    tmp=a[left];
    i=left;
    j= left+1;
    while(j<=right){
        if(a[j]<tmp){
            i = i+1;
            t=a[i];
            a[i]=a[j];
            a[j]=t;
        }
        j=j+1;
    }
    t=a[i];
    a[i]=tmp;
    a[left]=t;
    return i;
}
void quicksort(int a[],int left,int right){
    int pivot;
    if(left<right){
        pivot=part(a,left,right);
        quicksort(a,left,pivot-1);
        quicksort(a,pivot+1,right);
    }
}
int main(){
    int a[10];
    int i;
    i=0;
    while(i<10){
        /*
        a[i]=10-i;
        */
        a[i]=input();
        i=i+1;
    }
    quicksort(a,0,9);
    i=0;
    while(i<10){
        output(a[i]);
        i=i+1;
    }
    return 0;
}