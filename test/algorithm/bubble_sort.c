void bubble_sort(int a[],int n){
    int i;
    int j;
    int temp;
    i=0;
    while(i<n)
    {  
        j=0;
        while(j<n-i-1)
        {  
            if(a[j]>a[j+1])  
            {  
                temp=a[j];  
                a[j]=a[j+1];  
                a[j+1]=temp;  
            } 
            j=j+1; 
        }  
        i=i+1;  
    }  
}


void main(){
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
    i=0;
    while(i<10){
        output(a[i]);
        i=i+1;
    }
    i=0;
    bubble_sort(a,10);
    while(i<10){
        output(a[i]);
        i=i+1;
    }
    return 0;
}