namespace VisitorDemo;

public delegate ref F Getter<T, F>(ref T parent);

public static class Getters
{
    public static ref T Identity<T>(ref T parent) => ref parent;
    public static Getter<A, C> Compose<A, B, C>(Getter<A, B> f, Getter<B, C> g) => (ref A x) => ref g(ref f(ref x));
}
