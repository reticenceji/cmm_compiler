int gcd(int u, int v)
{
    if(v == 0)
        return u;
    else
        return gcd(v, u-u/v*v);
}
void main()
{
    int x;
    int y;
    x = 140;
    y = 49;
    output(gcd(x, y));
}