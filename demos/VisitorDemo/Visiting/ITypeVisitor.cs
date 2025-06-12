namespace VisitorDemo;

public interface ITypeVisitor<T>
{
    void VisitInt(Getter<T, int> get);
    void VisitString(Getter<T, string> get);

    void VisitStruct<S, M>(Getter<T, S> get, M type) where M : IStructType<S>;
}
