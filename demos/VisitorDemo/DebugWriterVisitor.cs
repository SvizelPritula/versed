namespace VisitorDemo;

public class DebugWriterVisitor<T>(T value, TextWriter textWriter, int indentLevel) : ITypeVisitor<T>
{
    public void VisitInt() => textWriter.WriteLine(value);
    public void VisitString() => textWriter.WriteLine(value);

    public void VisitStruct(IStructType<T> type)
    {
        textWriter.WriteLine("{");
        type.Accept(new DebugWriterStructVisitor<T>(value, textWriter, indentLevel + 1));
        textWriter.Write(new string(' ', indentLevel * 2));
        textWriter.WriteLine("}");
    }
}

public class DebugWriterStructVisitor<T>(T value, TextWriter textWriter, int indentLevel) : IStructVisitor<T>
{
    public void VisitField<F>(string name, Func<T, F> get, Action<T, F> set, IType<F> visitor)
    {
        textWriter.Write(new string(' ', indentLevel * 2));
        textWriter.Write(name);
        textWriter.Write(": ");
        visitor.Accept(new DebugWriterVisitor<F>(get(value), textWriter, indentLevel));
    }
}
