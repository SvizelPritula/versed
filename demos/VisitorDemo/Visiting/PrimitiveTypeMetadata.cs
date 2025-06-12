namespace VisitorDemo;

public class IntMetadata : IType<int>
{
    public static IntMetadata Instance { get; } = new();
    private IntMetadata() { }
    public void Accept<V>(V visitor) where V : ITypeVisitor<int>, allows ref struct => visitor.VisitInt(Getters.Identity);
}

public class StringMetadata : IType<string>
{
    public static StringMetadata Instance { get; } = new();
    private StringMetadata() { }
    public void Accept<V>(V visitor) where V : ITypeVisitor<string>, allows ref struct => visitor.VisitString(Getters.Identity);
}
