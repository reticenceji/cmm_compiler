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
    x = 140;
    y = 49;
    output(gcd(x, y));
}