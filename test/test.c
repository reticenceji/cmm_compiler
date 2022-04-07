int main() {
    int a;
    int b[10];
    a = 1;
    b[0] = 0;
    b[1] = a;
    b[2] = a + 1;
    b[3] = a * 3;
    b[4] = b[3] + 1;
    while (a >= 0) {
        b[6 - a] = 6 - a;
        a = a - 1;
    }
    if (b[6] == 6) {
        b[7] = 15 / 2;
    }
    return 1;
}