#include <stdio.h>

void output(int number) {
    printf("%d\n", number);
    return;
}
int input() {
    int number;
    scanf("%d", &number);
    return number;
}