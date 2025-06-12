namespace VisitorDemo;

public interface IStructType<T>
{
    void Accept<V>(V visitor) where V: IStructVisitor<T>, allows ref struct;
}
