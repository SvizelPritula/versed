using VisitorDemo;

public class AdapterVisitor<I, O>(ITypeVisitor<O> inner, Getter<O, I> map) : ITypeVisitor<I>
{
    public void VisitInt(Getter<I, int> get) => inner.VisitInt(v => get(map(v)));
    public void VisitString(Getter<I, string> get) => inner.VisitString(v => get(map(v)));
    public void VisitStruct<S, M>(Getter<I, S> get, M type) where M : IStructType<S> => inner.VisitStruct(v => get(map(v)), type);
}
