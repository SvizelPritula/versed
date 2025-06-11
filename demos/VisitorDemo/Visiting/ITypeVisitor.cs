namespace VisitorDemo;

public interface ITypeVisitor<T>
{
    void VisitInt();
    void VisitString();

    void VisitStruct(IStructType<T> type);
}
