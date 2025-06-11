namespace VisitorDemo;

public interface IStructType<T>
{
    void Accept(IStructVisitor<T> visitor);
}
