namespace VisitorDemo;

public interface IStructVisitor<T>
{
    void VisitField<F, M>(string name, Getter<T, F> get, M visitor) where M : IType<F>;
}
