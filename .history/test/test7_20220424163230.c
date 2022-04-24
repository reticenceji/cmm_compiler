int gcd(int u, int v)
{
    u = 1;
    if(v == 0)
        return u;
    else
        return gcd(u, u-u/v*v);
}
void main()
{
    int x;
    int y;
    x = input();
    y = input();
    output(gcd(x, y));
}