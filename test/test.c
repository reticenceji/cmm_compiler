int a;

void func(int a) {}
int main() {
    int a;
    int b;
    // assignment
    a = 1;
    b = a + 114514;
    b = a * 1 + b / 2 - (3 - 5);
    if (b != 1) {
        b = 1;
    } else if (a > 1) {
        b = 2;
    }
    func(1);
    while (a > 0) {
        a = a - 1;
    }
    return b;
}