int a;
int gcd(int a, int b) {
    if (b == 0)
        return a;
    else
        return gcd(b, a - a / b * b);
    /*a-a/b*b == a mod b*/
}
int add(int a, int b) {
    return a + b;
    /*
        add
    */
}
int b[20];
int main() {
    int c;
    a = 10;
    c = 6;
}