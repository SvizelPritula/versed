namespace VisitorDemo;

public ref struct TrademakerVisitor<T>(ref T value) : ITypeVisitor<T>
{
    readonly ref T storedValue = ref value;

    public void VisitInt(Getter<T, int> get) { }
    public void VisitString(Getter<T, string> get) => get(ref storedValue) += "™";

    public void VisitStruct<S, M>(Getter<T, S> get, M type) where M : IStructType<S>
        => type.Accept(new TrademakerStructVisitor<S>(ref get(ref storedValue)));

}

public ref struct TrademakerStructVisitor<T>(ref T value) : IStructVisitor<T>
{
    readonly ref T storedValue = ref value;

    public void VisitField<F, M>(string name, Getter<T, F> get, M visitor) where M : IType<F>
    {
        visitor.Accept(new TrademakerVisitor<F>(ref get(ref storedValue)));
    }
}
