namespace VisitorDemo;

public interface IStructVisitor<T>
{
    void VisitField<F>(string name, Func<T, F> get, Action<T, F> set, IType<F> visitor);
}
