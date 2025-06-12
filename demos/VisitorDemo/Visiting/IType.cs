namespace VisitorDemo;

public interface IType<T>
{
    void Accept<V>(V visitor) where V : ITypeVisitor<T>, allows ref struct;
}
