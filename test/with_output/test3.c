int array(int a[], int n) {
    int sum;
    int i;
    sum = 0;
    i = 0;
    while (i < n) {
        sum = sum + a[i];
        i = i + 1;
    }
    return sum;
}
void main() {
    int a[10];
    int b;
    int c;
    a[0] = 0;
    a[1] = 1;
    a[2] = 2;
    a[3] = 3;
    a[4] = 4;
    a[5] = 5;
    a[6] = 6;
    a[7] = 7;
    a[8] = 8;
    a[9] = 9;
    c = array(a, b);
    output(c);
}