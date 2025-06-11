namespace VisitorDemo;

public interface IType<T>
{
    void Accept(ITypeVisitor<T> visitor);
}
