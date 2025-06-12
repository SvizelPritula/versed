namespace VisitorDemo;

public class IntMetadata : IType<int>
{
    public static IntMetadata Instance { get; } = new();
    private IntMetadata() { }
    public void Accept(ITypeVisitor<int> visitor) => visitor.VisitInt(a => a);
}

public class StringMetadata : IType<string>
{
    public static StringMetadata Instance { get; } = new();
    private StringMetadata() { }
    public void Accept(ITypeVisitor<string> visitor) => visitor.VisitString(a => a);
}
