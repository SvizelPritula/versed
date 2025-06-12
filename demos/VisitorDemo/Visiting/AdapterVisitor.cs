namespace VisitorDemo;

public ref struct AdapterVisitor<I, O, V>(V inner, Getter<O, I> map) : ITypeVisitor<I> where V : ITypeVisitor<O>, allows ref struct
{
    readonly V storedInner = inner;

    public void VisitInt(Getter<I, int> get) => storedInner.VisitInt(Getters.Compose(map, get));
    public void VisitString(Getter<I, string> get) => storedInner.VisitString(Getters.Compose(map, get));
    public void VisitStruct<S, M>(Getter<I, S> get, M type) where M : IStructType<S> => storedInner.VisitStruct(Getters.Compose(map, get), type);
}
