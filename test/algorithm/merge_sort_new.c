void merge(int a[],int left,int mid,int right){
    int i;
    int j;
    int t;
    int flag;
    int k;
    int copy[10];
    i=left;
    j=mid+1;
    t=0;
    if(i<=mid){
        if(j<=right){
            flag=1;
        }
        else{
            flag=0;
        }
    }
    else{
        flag =0;
    }
    while(flag == 1){
        if(a[i] > a[j]){
            copy[t]=a[j];
            t=t+1;
            j=j+1;
        }
        else{
            copy[t]=a[i];
            t=t+1;
            i=i+1;
        }
        if(i<=mid){
            if(j<=right){
                flag=1;
            }
            else{
                flag=0;
            }
        }
        else{
            flag =0;
        }
    }
    while(i<=mid){
        copy[t]=a[i];
        t=t+1;
        i=i+1;
    }
    while(j<=right){
        copy[t]=a[j];
        t=t+1;
        j=j+1;
    }
    k=left;
    while(k<=right){
        a[k]=copy[k-left];
        k=k+1;
    }
}
void mergesort(int a[],int left,int right) {
    int mid;
    if(left>=right){
       return; 
    } 
    mid=left+(right-left)/2;
    mergesort(a,left,mid);
    mergesort(a,mid+1,right);
    merge(a,left,mid,right);
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
    mergesort(a,0,9);
    i=0;
    while(i<10){
        output(a[i]);
        i=i+1;
    }
    return 0;
}